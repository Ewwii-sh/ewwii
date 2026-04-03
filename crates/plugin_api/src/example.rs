//! A module providing example implementations of plugins

/// An example plugin that can be exported directly
pub struct ExamplePlugin;

impl crate::Plugin for ExamplePlugin {
    /// Example code that initalizes the plugin
    fn init(&self, host: &dyn crate::EwwiiAPI) {
        host.log("Example plugin says Hello!");
    }
}
