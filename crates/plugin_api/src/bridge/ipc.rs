use serde::{Deserialize, Serialize};

// === ipc access implementation  === //
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcRequest {
    /// Control widgets
    WidgetControl(WidgetControlType),
    /// 1. Property
    /// 2. Value
    Update(String, String),
    Close(Vec<String>),
    /// 1. Name of window to open
    /// 2. Whether to toggle the window
    Open(String, bool),
    /// Reload config and css
    Reload,
    /// Close all windows
    CloseAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetControlType {
    /// Perform an action on a widget
    Action(WidgetActionType),
    /// Remove a widget
    Remove(String),
    /// Create a widget
    Create { parent: String, codes: Vec<String> },
    /// Get the property of a widget
    PropertyGet { widget: String, prop: String },
    /// Set a property of a widget
    PropertyUpdate { widget: String, prop: String, value: String },
    /// Add a class to a widget
    AddClass { widget: String, class: String },
    /// Remove the class of a widget
    RemoveClass { widget: String, class: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetActionType {
    /// Scroll a widget (must be ScrolledWindow)
    Scroll { widget: String, value: f64 },
    /// Focus a widget
    Focus(String),
}
