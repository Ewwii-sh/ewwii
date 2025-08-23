use crate::cstm_enum_parse;
use std::fmt::Display;

#[derive(Debug, thiserror::Error)]
pub struct EnumParseError {
    pub input: String,
    pub expected: Vec<&'static str>,
}
impl Display for EnumParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse `{}`, must be one of {}", self.input, self.expected.join(", "))
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    derive_more::Display,
    smart_default::SmartDefault,
    serde::Serialize,
)]
pub enum WindowStacking {
    #[default]
    Foreground,
    Background,
    Bottom,
    Overlay,
}

impl std::str::FromStr for WindowStacking {
    type Err = EnumParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        cstm_enum_parse! { "WindowStacking", s,
            "foreground" | "fg" => WindowStacking::Foreground,
            "background" | "bg" => WindowStacking::Background,
            "bottom" | "bt" => WindowStacking::Bottom,
            "overlay" | "ov" => WindowStacking::Overlay,
        }
    }
}
