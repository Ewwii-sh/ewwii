use rhai::Map;

#[derive(Debug, Clone)]
pub struct WindowDefinition {
    pub name: String,
    pub geometry: Option<WindowGeometry>,
    pub stacking: Option<String>, // stringified Rhai expressions
    pub monitor: Option<String>,
    pub resizable: Option<bool>,
    pub backend_options: BackendOptions,
}

#[derive(Debug, Clone)]
pub struct WindowGeometry {
    pub anchor_point: Option<String>, // e.g., "top-left"
    pub offset: Coords,
    pub size: Coords,
}

#[derive(Debug, Clone)]
pub struct Coords {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone)]
pub struct BackendOptions {
    pub shadow: Option<bool>,
    pub rounded: Option<bool>,
    pub focusable: Option<bool>,
}
