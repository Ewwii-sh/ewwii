use std::str::FromStr;

use anyhow::Result;

use super::window_definition::EnumParseError;
use crate::{
    enum_parse,
    // diag_error::{DiagResult, DiagError, DiagResultExt},
    // parser::{ast::Ast, ast_iterator::AstIterator, from_ast::FromAstElementContent},
    window::{coords, coords::NumWithUnit},
};
// use ewwii_shared_utils::{Span, VarName};

// use crate::dynval::{DynVal, FromDynVal, ConversionError};
use ewwii_shared_utils::prop::{Property, PropertyMap};
// use crate::error::{DiagError, DiagResultExt};

pub trait TryFromProperty: Sized {
    fn try_from_prop(p: &Property) -> Option<Self>;
}

// Basic type implementations
impl TryFromProperty for bool {
    fn try_from_prop(p: &Property) -> Option<Self> {
        p.as_bool()
    }
}

impl TryFromProperty for String {
    fn try_from_prop(p: &Property) -> Option<Self> {
        p.as_str().map(|s| s.to_string())
    }
}

impl TryFromProperty for i64 {
    fn try_from_prop(p: &Property) -> Option<Self> {
        p.as_int()
    }
}

impl TryFromProperty for NumWithUnit {
    fn try_from_prop(p: &Property) -> Option<Self> {
        match p {
            Property::String(s) => Self::from_str(s).ok(),
            // Map raw numbers to Pixels(i32)
            Property::Int(i) => Some(Self::Pixels(*i as i32)),
            Property::Float(f) => Some(Self::Pixels(*f as i32)),
            _ => None,
        }
    }
}

impl TryFromProperty for X11StrutDefinitionExpr {
    fn try_from_prop(p: &Property) -> Option<Self> {
        let map = p.as_map()?;

        let distance = map.get("distance").and_then(|v| NumWithUnit::try_from_prop(v))?;

        let side = map.get("side").and_then(|v| NumWithUnit::try_from_prop(v));

        Some(Self { side, distance })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
    #[error(transparent)]
    EnumParseError(#[from] EnumParseError),
    #[error(transparent)]
    CoordsError(#[from] coords::Error),
}

/// Backend-specific options of a window
/// Unevaluated form of [`BackendWindowOptions`]
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct BackendWindowOptionsDef {
    pub wayland: WlBackendWindowOptionsDef,
    pub x11: X11BackendWindowOptionsDef,
}

impl BackendWindowOptionsDef {
    pub fn eval(&self, properties: PropertyMap) -> Result<BackendWindowOptions, Error> {
        Ok(BackendWindowOptions {
            wayland: self.wayland.eval(properties.clone())?,
            x11: self.x11.eval(properties)?,
        })
    }

    // pub fn from_attrs(attrs: &mut Attributes) -> DiagResult<Self> {
    //     let struts = attrs.ast_optional("reserve")?;
    //     let window_type = attrs.ast_optional("windowtype")?;
    //     let focusable = attrs.ast_optional("focusable")?;
    //     let x11 = X11BackendWindowOptionsDef {
    //         sticky: attrs.ast_optional("sticky")?,
    //         struts,
    //         window_type,
    //         wm_ignore: attrs.ast_optional("wm-ignore")?,
    //     };
    //     let wayland = WlBackendWindowOptionsDef {
    //         exclusive: attrs.ast_optional("exclusive")?,
    //         focusable,
    //         namespace: attrs.ast_optional("namespace")?,
    //     };

    //     Ok(Self { wayland, x11 })
    // }

    // pass rhai map from WindowDefinition here
    pub fn from_map(map: &PropertyMap) -> Result<Self> {
        // let get = |key: &str| map.get(key).cloned();

        let struts = Self::get_optional(map, "reserve");
        let window_type = Self::get_optional(map, "windowtype");
        let focusable = Self::get_optional(map, "focusable");

        let x11 = X11BackendWindowOptionsDef {
            sticky: Self::get_optional(map, "sticky"),
            struts,
            window_type,
            wm_ignore: Self::get_optional(map, "wm_ignore"),
        };

        let wayland = WlBackendWindowOptionsDef {
            exclusive: Self::get_optional(map, "exclusive"),
            focusable,
            namespace: Self::get_optional(map, "namespace"),
            force_normal: Self::get_optional(map, "force_normal"),
        };

        Ok(Self { wayland, x11 })
    }

    fn get_optional<T: TryFromProperty>(map: &PropertyMap, key: &str) -> Option<T> {
        map.get(key).and_then(|v| T::try_from_prop(v))
    }
}

/// Backend-specific options of a window that are backend
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct BackendWindowOptions {
    pub x11: X11BackendWindowOptions,
    pub wayland: WlBackendWindowOptions,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct X11BackendWindowOptions {
    pub wm_ignore: bool,
    pub sticky: bool,
    pub window_type: X11WindowType,
    pub struts: X11StrutDefinition,
}

/// Unevaluated form of [`X11BackendWindowOptions`]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct X11BackendWindowOptionsDef {
    pub sticky: Option<NumWithUnit>,
    pub struts: Option<X11StrutDefinitionExpr>,
    pub window_type: Option<String>,
    pub wm_ignore: Option<NumWithUnit>,
}

impl X11BackendWindowOptionsDef {
    fn eval(&self, properties: PropertyMap) -> Result<X11BackendWindowOptions, Error> {
        Ok(X11BackendWindowOptions {
            sticky: properties.get("sticky").and_then(|d| d.as_bool()).unwrap_or(true),

            struts: match properties.get("reserve").and_then(|v| v.as_map()) {
                Some(obj_map) => {
                    let distance = obj_map
                        .get("distance")
                        .and_then(|v| v.as_str())
                        .ok_or(Error::MissingField("distance"))
                        .and_then(|s| NumWithUnit::from_str(s).map_err(Into::into))?;

                    let side = obj_map
                        .get("side")
                        .and_then(|s| s.as_str())
                        .map(Side::from_str)
                        .transpose()?
                        .unwrap_or_default();

                    X11StrutDefinition { distance, side }
                }
                None => X11StrutDefinition::default(),
            },

            window_type: match properties.get("windowtype") {
                Some(dynval) => {
                    let s = dynval.as_str().unwrap();
                    X11WindowType::from_str(&s)?
                }
                None => X11WindowType::default(),
            },

            wm_ignore: {
                let wm_ignore = properties.get("wm_ignore").and_then(|d| d.as_bool());
                wm_ignore.unwrap_or_else(|| {
                    properties.get("windowtype").is_none() && properties.get("reserve").is_none()
                })
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct WlBackendWindowOptions {
    pub exclusive: bool,
    pub focusable: WlWindowFocusable,
    pub namespace: Option<String>,
    pub force_normal: bool,
}

/// Unevaluated form of [`WlBackendWindowOptions`]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct WlBackendWindowOptionsDef {
    pub exclusive: Option<bool>,
    pub focusable: Option<String>,
    pub namespace: Option<String>,
    pub force_normal: Option<bool>,
}

impl WlBackendWindowOptionsDef {
    fn eval(&self, properties: PropertyMap) -> Result<WlBackendWindowOptions, Error> {
        Ok(WlBackendWindowOptions {
            exclusive: properties.get("exclusive").and_then(|d| d.as_bool()).unwrap_or(false),
            focusable: match properties.get("focusable") {
                Some(dynval) => {
                    let s = dynval.as_str().unwrap_or_default().to_lowercase();
                    WlWindowFocusable::from_str(&s)?
                }
                None => WlWindowFocusable::default(),
            },
            namespace: properties.get("namespace").and_then(|d| d.as_str()).map(String::from),
            force_normal: properties.get("force_normal").and_then(|d| d.as_bool()).unwrap_or(false),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, smart_default::SmartDefault, serde::Serialize)]
pub enum WlWindowFocusable {
    #[default]
    None,
    Exclusive,
    OnDemand,
}
impl FromStr for WlWindowFocusable {
    type Err = EnumParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        enum_parse! { "focusable", s,
            "none" => Self::None,
            "exclusive" => Self::Exclusive,
            "ondemand" => Self::OnDemand,
        }
    }
}

/// Window type of an x11 window
#[derive(Debug, Clone, PartialEq, Eq, smart_default::SmartDefault, serde::Serialize)]
pub enum X11WindowType {
    #[default]
    Dock,
    Dialog,
    Toolbar,
    Normal,
    Utility,
    Desktop,
    Notification,
}
impl FromStr for X11WindowType {
    type Err = EnumParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        enum_parse! { "window type", s,
            "dock" => Self::Dock,
            "toolbar" => Self::Toolbar,
            "dialog" => Self::Dialog,
            "normal" => Self::Normal,
            "utility" => Self::Utility,
            "desktop" => Self::Desktop,
            "notification" => Self::Notification,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, smart_default::SmartDefault, serde::Serialize)]
pub enum Side {
    #[default]
    Top,
    Left,
    Right,
    Bottom,
}

impl FromStr for Side {
    type Err = EnumParseError;

    fn from_str(s: &str) -> Result<Side, Self::Err> {
        enum_parse! { "side", s,
            "l" | "left" => Side::Left,
            "r" | "right" => Side::Right,
            "t" | "top" => Side::Top,
            "b" | "bottom" => Side::Bottom,
        }
    }
}

/// Unevaluated form of [`X11StrutDefinition`]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct X11StrutDefinitionExpr {
    pub side: Option<NumWithUnit>,
    pub distance: NumWithUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize)]
pub struct X11StrutDefinition {
    pub side: Side,
    pub distance: NumWithUnit,
}
