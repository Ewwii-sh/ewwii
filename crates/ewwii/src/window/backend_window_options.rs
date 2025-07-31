use std::str::FromStr;

use anyhow::Result;

use super::window_definition::EnumParseError;
use crate::{
    cstm_enum_parse,
    // diag_error::{DiagResult, DiagError, DiagResultExt},
    // parser::{ast::Ast, ast_iterator::AstIterator, from_ast::FromAstElementContent},
    window::{coords, coords::NumWithUnit},
};
// use ewwii_shared_util::{Span, VarName};

// use crate::dynval::{DynVal, FromDynVal, ConversionError};
use rhai::Map;
// use crate::error::{DiagError, DiagResultExt};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
    #[error(transparent)]
    EnumParseError(#[from] EnumParseError),
    #[error("Enum parse error: {0}")]
    EnumParseErrorMessage(&'static str),
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
    pub fn eval(&self, properties: Map) -> Result<BackendWindowOptions, Error> {
        Ok(BackendWindowOptions { wayland: self.wayland.eval(properties.clone())?, x11: self.x11.eval(properties)? })
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
    pub fn from_map(map: &Map) -> Result<Self> {
        let get = |key: &str| map.get(key).cloned();

        let struts = Self::get_optional(map, "reserve")?;
        let window_type = Self::get_optional(map, "windowtype")?;
        let focusable = Self::get_optional(map, "focusable")?;

        let x11 = X11BackendWindowOptionsDef {
            sticky: Self::get_optional(map, "sticky")?,
            struts,
            window_type,
            wm_ignore: Self::get_optional(map, "wm-ignore")?,
        };

        let wayland = WlBackendWindowOptionsDef {
            exclusive: Self::get_optional(map, "exclusive")?,
            focusable,
            namespace: Self::get_optional(map, "namespace")?,
        };

        Ok(Self { wayland, x11 })
    }

    fn get_optional<T: Clone + 'static>(map: &Map, key: &str) -> Result<Option<T>> {
        Ok(map.get(key).cloned().and_then(|v| v.try_cast::<T>()))
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
    fn eval(&self, properties: Map) -> Result<X11BackendWindowOptions, Error> {
        Ok(X11BackendWindowOptions {
            sticky: properties.get("sticky").map(|d| d.clone_cast::<bool>()).unwrap_or(true),

            struts: match properties.get("reserve") {
                Some(dynval) => {
                    let obj_map = dynval.read_lock::<Map>().ok_or(Error::EnumParseErrorMessage("Expected map for reserve"))?;

                    let distance_str = obj_map.get("distance").ok_or(Error::MissingField("distance"))?.clone_cast::<String>();

                    let distance = NumWithUnit::from_str(&distance_str)?;

                    let side = obj_map.get("side").map(|s| s.clone_cast::<String>()).map(|s| Side::from_str(&s)).transpose()?;

                    X11StrutDefinition { distance, side: side.unwrap_or(Side::default()) }
                }
                None => X11StrutDefinition::default(),
            },

            window_type: match properties.get("windowtype") {
                Some(dynval) => {
                    let s = dynval.clone_cast::<String>();
                    X11WindowType::from_str(&s)?
                }
                None => X11WindowType::default(),
            },

            wm_ignore: {
                let wm_ignore = properties.get("wm_ignore").map(|d| d.clone_cast::<bool>());
                wm_ignore.unwrap_or_else(|| properties.get("windowtype").is_none() && properties.get("reserve").is_none())
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct WlBackendWindowOptions {
    pub exclusive: bool,
    pub focusable: WlWindowFocusable,
    pub namespace: Option<String>,
}

/// Unevaluated form of [`WlBackendWindowOptions`]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct WlBackendWindowOptionsDef {
    pub exclusive: Option<bool>,
    pub focusable: Option<String>,
    pub namespace: Option<String>,
}

impl WlBackendWindowOptionsDef {
    fn eval(&self, properties: Map) -> Result<WlBackendWindowOptions, Error> {
        Ok(WlBackendWindowOptions {
            exclusive: properties.get("exclusive").map(|d| d.clone_cast::<bool>()).unwrap_or(false),
            focusable: match properties.get("focusable") {
                Some(dynval) => {
                    let s = dynval.clone_cast::<String>().to_lowercase();
                    WlWindowFocusable::from_str(&s)?
                }
                None => WlWindowFocusable::default(),
            },
            namespace: properties.get("namespace").map(|d| d.clone_cast::<String>()),
        })
    }
}

// fn eval_opt_expr_as_bool(
//     opt_expr: &Option<NumWithUnit>,
//     default: bool,
//     local_variables: &HashMap<VarName, DynVal>,
// ) -> Result<bool, EvalError> {
//     Ok(match opt_expr {
//         Some(expr) => expr.eval(local_variables)?.as_bool()?,
//         None => default,
//     })
// }

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
        cstm_enum_parse! { "focusable", s,
            "none" => Self::None,
            "exclusive" => Self::Exclusive,
            "ondemand" => Self::OnDemand,
            // legacy support
            "true" => Self::Exclusive,
            "false" => Self::None,
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
        cstm_enum_parse! { "window type", s,
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
        cstm_enum_parse! { "side", s,
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

// impl X11StrutDefinitionExpr {
//     fn eval(&self, local_variables: &HashMap<VarName, DynVal>) -> Result<X11StrutDefinition, Error> {
//         Ok(X11StrutDefinition {
//             side: match &self.side {
//                 Some(expr) => Side::from_dynval(&expr.eval(local_variables)?)?,
//                 None => Side::default(),
//             },
//             distance: NumWithUnit::from_dynval(&self.distance.eval(local_variables)?)?,
//         })
//     }
// }

// impl FromAstElementContent for X11StrutDefinitionExpr {
//     const ELEMENT_NAME: &'static str = "struts";

//     fn from_tail<I: Iterator<Item = Ast>>(_span: Span, mut iter: AstIterator<I>) -> DiagResult<Self> {
//         let mut attrs = iter.expect_key_values()?;
//         iter.expect_done().map_err(DiagError::from).note("Check if you are missing a colon in front of a key")?;
//         Ok(X11StrutDefinitionExpr { side: attrs.ast_optional("side")?, distance: attrs.ast_required("distance")? })
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize)]
pub struct X11StrutDefinition {
    pub side: Side,
    pub distance: NumWithUnit,
}
