// use anyhow::anyhow;
use derive_more::{Debug, Display};
// use once_cell::sync::Lazy;
// use regex::Regex;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::{fmt, str::FromStr};

use super::window_definition::EnumParseError;
use crate::enum_parse;
use crate::window::coords::{Error, NumWithUnit};

/// A pair of [NumWithUnit] values for x and y
#[derive(Clone, Copy, PartialEq, Deserialize, Serialize, Display, Debug, Default)]
#[display("{}x{}", x, y)]
pub struct Coords {
    pub x: NumWithUnit,
    pub y: NumWithUnit,
}

impl FromStr for Coords {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (sx, sy) =
            s.split_once(|c: char| c == 'x' || c == '*').ok_or(Error::MalformedCoords)?;
        Ok(Coords { x: sx.parse()?, y: sy.parse()? })
    }
}

impl From<crate::window::coords::Coords> for Coords {
    fn from(c: crate::window::coords::Coords) -> Self {
        Self { x: c.x, y: c.y }
    }
}

impl Coords {
    /// Create from absolute pixel values
    pub fn from_pixels((x, y): (i32, i32)) -> Self {
        Coords { x: NumWithUnit::Pixels(x), y: NumWithUnit::Pixels(y) }
    }
    /// Resolve relative or absolute coordinates against container size
    pub fn relative_to(&self, width: i32, height: i32) -> (i32, i32) {
        (self.x.to_pixels(width), self.y.to_pixels(height))
    }
}

impl NumWithUnit {
    pub fn to_pixels(&self, max: i32) -> i32 {
        match *self {
            NumWithUnit::Percent(n) => ((max as f64 / 100.0) * n as f64).round() as i32,
            NumWithUnit::Pixels(n) => n,
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
        enum_parse! { "x-alignment", s,
            "l" | "left" => AnchorAlignment::START,
            "c" | "center" => AnchorAlignment::CENTER,
            "r" | "right" => AnchorAlignment::END,
        }
    }
    pub fn from_y_alignment(s: &str) -> Result<Self, EnumParseError> {
        enum_parse! { "y-alignment", s,
            "t" | "top" => AnchorAlignment::START,
            "c" | "center" => AnchorAlignment::CENTER,
            "b" | "bottom" => AnchorAlignment::END,
        }
    }
    pub fn alignment_to_coordinate(&self, inner: i32, outer: i32) -> i32 {
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

impl std::str::FromStr for AnchorPoint {
    type Err = EnumParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x_str, y_str) = s.split_once(' ').ok_or_else(|| EnumParseError {
            input: s.to_string(),
            expected: vec!["<horizontal> <vertical>"],
        })?;

        let x = AnchorAlignment::from_x_alignment(x_str)?;
        let y = AnchorAlignment::from_y_alignment(y_str)?;

        Ok(AnchorPoint { x, y })
    }
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
        size: Option<Coords>,
    ) -> Self {
        WindowGeometry {
            anchor_point: anchor_point.unwrap_or(self.anchor_point),
            offset: offset.unwrap_or(self.offset),
            size: size.unwrap_or(self.size),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NumWithUnit::to_pixels
    #[test]
    fn to_pixels_percent_of_zero_container() {
        assert_eq!(NumWithUnit::Percent(50.0).to_pixels(0), 0);
    }

    #[test]
    fn to_pixels_percent_rounds() {
        // 50% of 101 = 50.5 (rounds to 51)
        assert_eq!(NumWithUnit::Percent(50.0).to_pixels(101), 51);
    }

    #[test]
    fn to_pixels_zero_percent() {
        assert_eq!(NumWithUnit::Percent(0.0).to_pixels(500), 0);
    }

    // AnchorAlignment::alignment_to_coordinate
    #[test]
    fn alignment_center_inner_larger_than_outer() {
        // if widget bigger than screen then it goes negative
        assert_eq!(AnchorAlignment::CENTER.alignment_to_coordinate(600, 500), -50);
    }

    #[test]
    fn alignment_center_odd_difference_truncates() {
        // outer - inner = 101, integer division truncates to 50
        assert_eq!(AnchorAlignment::CENTER.alignment_to_coordinate(100, 201), 50);
    }

    // Coords::relative_to
    #[test]
    fn relative_to_does_not_swap_axes() {
        let coords = Coords { x: NumWithUnit::Percent(10.0), y: NumWithUnit::Percent(50.0) };
        let (x, y) = coords.relative_to(1000, 2000);
        // x should be 10% of width (1000), y should be 50% of height (2000)
        assert_eq!(x, 100);
        assert_eq!(y, 1000);
    }

    // AnchorPoint::from_str
    #[test]
    fn anchor_point_missing_space_errors() {
        assert!("lefttop".parse::<AnchorPoint>().is_err());
    }

    #[test]
    fn anchor_point_invalid_token_errors() {
        assert!("diagonal up".parse::<AnchorPoint>().is_err());
    }

    // AnchorPoint::Dispay
    #[test]
    fn anchor_point_display_center_center() {
        let ap = AnchorPoint { x: AnchorAlignment::CENTER, y: AnchorAlignment::CENTER };
        assert_eq!(ap.to_string(), "center");
    }

    #[test]
    fn anchor_point_display_non_center() {
        let ap = AnchorPoint { x: AnchorAlignment::START, y: AnchorAlignment::END };
        assert_eq!(ap.to_string(), "start end");
    }

    // WindowGeometry::override_with
    #[test]
    fn override_with_partial_only_replaces_some() {
        let geom = WindowGeometry {
            anchor_point: AnchorPoint { x: AnchorAlignment::START, y: AnchorAlignment::START },
            offset: Coords::from_pixels((0, 0)),
            size: Coords::from_pixels((800, 600)),
        };
        let new_size = Coords::from_pixels((1920, 1080));
        let result = geom.override_with(None, None, Some(new_size));

        assert_eq!(result.anchor_point, geom.anchor_point);
        assert_eq!(result.offset, geom.offset);
        assert_eq!(result.size, new_size);
    }
}
