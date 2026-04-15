//! This module provides examples of plugin implementation

/// An example plugin that can be exported directly
pub struct ExamplePlugin;

impl crate::Plugin for ExamplePlugin {
    /// Example metadata for the plugin
    fn metadata(&self) -> crate::PluginInfo {
        crate::PluginInfo::new("com.example.plugin", "1.0.0")
    }

    /// Example code that initalizes the plugin
    fn init(&self, host: &dyn crate::EwwiiAPI) {
        host.log("Example plugin says Hello!");
    }
}
