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
//!
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

mod export_macros;

pub mod example;
pub mod widget_backend;

#[cfg(feature = "include-rhai")]
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
    /// _(include-rhai)_ Perform actions on the latest rhai engine.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{EwwiiAPI, Plugin};
    ///
    /// pub struct DummyStructure;
    ///
    /// impl Plugin for DummyStructure {
    ///     fn init(&self, host: &dyn EwwiiAPI) {
    ///         host.rhai_engine_action(Box::new(|eng| {
    ///             // eng = rhai::Engine
    ///             eng.set_max_expr_depths(128, 128);
    ///         }));
    ///     }
    /// }
    /// ```
    #[cfg(feature = "include-rhai")]
    fn rhai_engine_action(&self, f: Box<dyn FnOnce(&mut Engine) + Send>) -> Result<(), String>;

    // == Widget Rendering & Logic == //
    /// Get the list of all widget id's
    fn list_widget_ids(&self) -> Result<Vec<u64>, String>;

    /// _(include-gtk4)_ Perform actions on the latest widget registry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{EwwiiAPI, Plugin};
    ///
    /// pub struct DummyStructure;
    ///
    /// impl Plugin for DummyStructure {
    ///     fn init(&self, host: &dyn EwwiiAPI) {
    ///         host.widget_reg_action(Box::new(|wrg| {
    ///             // wrg = widget_backend::WidgetRegistryRepr
    ///             // The gtk4::Widget can be modified here.
    ///         }));
    ///     }
    /// }
    /// ```
    #[cfg(feature = "include-gtk4")]
    fn widget_reg_action(
        &self,
        f: Box<dyn FnOnce(&mut widget_backend::WidgetRegistryRepr) + Send>,
    ) -> Result<(), String>;
}

/// The API format that the plugin should follow.
/// This trait should be implemented for a structure and
/// that structure should be exported via FFI.
///
/// ## Example
///
/// ```rust
/// use ewwii_plugin_api::{Plugin, EwwiiAPI, export_plugin};
///
/// struct MyStruct;
///
/// impl Plugin for MyStruct {
///     fn init(&self, host: &dyn EwwiiAPI) {
///         /* Implementation Skipped */   
///      }
/// }
///
/// // Automatically does all the FFI related exports
/// export_plugin!(MyStruct);
/// ```
pub trait Plugin: Send + Sync {
    /// Function ran by host to startup plugin (and its a must-have for plugin loading)
    fn init(&self, host: &dyn EwwiiAPI);
}
