//! `ewwii_plguin_api` is a shared list of traits
//! that both ewwii and its plugins can use.
//! This is a **must-have** for building plugins for ewwii
//! as this explicit layout is what ewwii requires a plugin to have.
//!
//! ## Example
//!
//! The following example shows how this crate shall be used to build ewwii plugins:
//!
//! ```rust
//! use ewwii_plugin_api::{EwwiiAPI, Plugin, export_plugin};
//! pub struct DummyStructure;
//!
//! impl Plugin for DummyStructure {
//!     // critical for ewwii to launch the plugin
//!     fn init(&self, host: &dyn EwwiiAPI) {
//!         // will be printed by the host
//!         host.log("Plugin says Hello!");
//!     }
//! }
//!
//! // Critical for ewwii to load the plugin
//! export_plugin!(DummyStructure);
//! ```

pub mod export;
pub mod widget_backend;

use rhai::Engine;

/// The shared trait defining the Ewwii plugin API
pub trait EwwiiAPI: Send + Sync {
    // == General Stuff == //
    /// Print a message from the host
    fn print(&self, msg: &str);
    /// Log a message from the host
    fn log(&self, msg: &str);
    /// Log a warning from the host
    fn warn(&self, msg: &str);
    /// Log an error from the host
    fn error(&self, msg: &str);

    // == Rhai Manipulation Stuff == //
    /// Perform actions on the latest rhai engine
    fn rhai_engine_action(&self, f: Box<dyn FnOnce(&mut Engine) + Send>) -> Result<(), String>;

    // == Widget Rendering & Logic == //
    /// Get the list of all widget id's
    fn list_widget_ids(&self) -> Result<Vec<u64>, String>;

    /// Perform actions on the latest widget registry
    fn widget_reg_action(
        &self,
        f: Box<dyn FnOnce(&mut widget_backend::WidgetRegistryRepr) + Send>,
    ) -> Result<(), String>;
}

/// The API format that the plugin should follow
pub trait Plugin: Send + Sync {
    /// Function ran by host to startup plugin (and its a must-have for plugin loading)
    fn init(&self, host: &dyn EwwiiAPI);
}
