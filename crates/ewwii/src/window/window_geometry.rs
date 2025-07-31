use anyhow::anyhow;
use derive_more::{Debug, Display};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::{fmt, str::FromStr};

use super::window_definition::EnumParseError;
use crate::window::coords::{NumWithUnit};

#[macro_export]
macro_rules! cstm_enum_parse {
    ($name:literal, $input:expr, $($($s:literal)|* => $val:expr),* $(,)?) => {
        let input = $input.to_lowercase();
        match input.as_str() {
            $( $( $s )|* => Ok($val) ),*,
            _ => Err(EnumParseError {
                input,
                expected: vec![$($($s),*),*],
            })
        }
    };
}

/// Errors encountered when parsing numeric values or coordinates
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Failed to parse '{0}' as a length value")]
    NumParseFailed(String),
    #[error("Invalid unit '{0}', must be '%' or 'px'")]
    InvalidUnit(String),
    #[error("Invalid format. Coordinates must be formatted like '200x100' or '50%x50%' ")]
    MalformedCoords,
}

/// A pair of [NumWithUnit] values for x and y
#[derive(Clone, Copy, PartialEq, Deserialize, Serialize, Display, Debug, Default)]
#[display("{}x{}", x, y)]
pub struct Coords {
    pub x: NumWithUnit,
    pub y: NumWithUnit,
}

impl FromStr for Coords {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (sx, sy) = s.split_once(|c: char| c == 'x' || c == '*')
            .ok_or(ParseError::MalformedCoords)?;
        Ok(Coords { x: sx.parse()?, y: sy.parse()? })
    }
}

impl Coords {
    /// Create from absolute pixel values
    pub fn from_pixels((x, y): (i32, i32)) -> Self {
        Coords {
            x: NumWithUnit::Pixels(x),
            y: NumWithUnit::Pixels(y),
        }
    }
    /// Resolve relative or absolute coordinates against container size
    pub fn relative_to(&self, width: i32, height: i32) -> (i32, i32) {
        (self.x.to_pixels(width), self.y.to_pixels(height))
    }
}

impl NumWithUnit {
    pub fn to_pixels(&self, container_size: i32) -> i32 {
        match self {
            NumWithUnit::Pixels(px) => *px,
            NumWithUnit::Percent(p) => (p * container_size as f64).round() as i32,
        }
    }
}

/// Alignment options for anchoring
#[derive(Debug, Clone, Copy, Eq, PartialEq, SmartDefault, Serialize, Deserialize, Display)]
pub enum AnchorAlignment {
    #[display("start")]
    #[default]
    START,
    #[display("center")]
    CENTER,
    #[display("end")]
    END,
}

impl AnchorAlignment {
    pub fn from_x_alignment(s: &str) -> Result<Self, EnumParseError> {
        cstm_enum_parse! { "x-alignment", s,
            "l" | "left" => AnchorAlignment::START,
            "c" | "center" => AnchorAlignment::CENTER,
            "r" | "right" => AnchorAlignment::END,
        }
    }
    pub fn from_y_alignment(s: &str) -> Result<Self, EnumParseError> {
        cstm_enum_parse! { "y-alignment", s,
            "t" | "top" => AnchorAlignment::START,
            "c" | "center" => AnchorAlignment::CENTER,
            "b" | "bottom" => AnchorAlignment::END,
        }
    }
    pub fn align_coord(&self, inner: i32, outer: i32) -> i32 {
        match self {
            AnchorAlignment::START => 0,
            AnchorAlignment::CENTER => (outer - inner) / 2,
            AnchorAlignment::END => outer - inner,
        }
    }
}

/// A pair of horizontal and vertical anchor alignments
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AnchorPoint {
    pub x: AnchorAlignment,
    pub y: AnchorAlignment,
}

impl fmt::Display for AnchorPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.x, self.y) {
            (AnchorAlignment::CENTER, AnchorAlignment::CENTER) => write!(f, "center"),
            (x, y) => write!(f, "{} {}", x, y),
        }
    }
}

/// Final window geometry with anchor, offset, and size
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize)]
pub struct WindowGeometry {
    pub anchor_point: AnchorPoint,
    pub offset: Coords,
    pub size: Coords,
}

impl WindowGeometry {
    pub fn override_with(
        &self,
        anchor_point: Option<AnchorPoint>,
        offset: Option<Coords>,
        // if you are wondering why this is coords, ig that its because they watned to 
        // reuse coords? I dont know. It works so I am keeping it.
        size: Option<Coords>,
    ) -> Self {
        WindowGeometry {
            anchor_point: anchor_point.unwrap_or(self.anchor),
            offset: offset.unwrap_or(self.offset),
            size: size.unwrap_or(self.size),
        }
    }
}
