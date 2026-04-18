//! # ewwii_plugin_api
//!
//! `ewwii_plugin_api` provides the core traits, types, and abstractions
//! that bridge the **ewwii** host and its plugins.
//!
//! This crate provides a safe and easy plugin development API
//! through a simple and flexible interface for cross-boundary communication.
//!
//! ## Usage
//!
//! There are two ways to define a plugin: the **Recommended Macro** for
//! standard plugins, and the **Manual Implementation** for full control.
//!
//! ### 1. Recommended: Using `auto_plugin!`
//! For most use cases, the macro handles the boilerplate of exporting
//! symbols and implementing traits.
//!
//! ```rust
//! use ewwii_plugin_api::{auto_plugin, PluginInfo};
//!
//! auto_plugin!(
//!     DummyStructure,
//!     PluginInfo::new("com.app.example", "1.0.0"),
//!     host,
//!     {
//!         host.log("Plugin says Hello!");
//!     }
//! );
//! ```
//!
//! ### 2. Advanced: Manual Implementation
//! Use this approach if your plugin needs to maintain internal state,
//! implement additional traits, or manage complex lifetimes.
//!
//! ```rust
//! use ewwii_plugin_api::{EwwiiAPI, Plugin, PluginInfo, export_plugin};
//!
//! #[derive(Default)]
//! pub struct MyPlugin {
//!     count: std::sync::atomic::AtomicUsize,
//! }
//!
//! impl Plugin for MyPlugin {
//!     fn metadata(&self) -> PluginInfo {
//!         PluginInfo::new("com.app.example", "1.0.0")
//!     }
//!
//!     fn init(&self, host: &dyn EwwiiAPI) {
//!         host.log("Manual plugin initialized.");
//!     }
//! }
//!
//! // This macro exports the C-compatible symbols required by the host loader
//! export_plugin!(MyPlugin);
//! ```

mod bridge;
mod export_macros;

pub mod example;
pub mod proxy;

pub use ewwii_shared_utils as shared_utils;
pub use bridge::*;

/// The shared trait defining the Ewwii plugin API
pub trait EwwiiAPI: Send + Sync {
    // == General Stuff == //
    /// Log a message from the host
    fn log(&self, msg: &str);
    /// Log a warning from the host
    fn warn(&self, msg: &str);
    /// Log an error from the host
    fn error(&self, msg: &str);

    /// Expose a function that rhai configuration can call.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{
    ///     EwwiiAPI, Plugin,
    ///     PluginValue, PluginInfo,
    ///     NativeFn, NativeFnExt
    /// };
    ///
    /// pub struct DummyStructure;
    ///
    /// impl Plugin for DummyStructure {
    ///     fn metadata(&self) -> PluginInfo {
    ///         PluginInfo::new("test.example.register", "1.0")
    ///     }
    ///
    ///     fn init(&self, host: &dyn EwwiiAPI) {
    ///         host.register_function(
    ///             "my_func",
    ///             NativeFn::new(|args| {
    ///             // Do stuff
    ///             // - Perform things on the args (if needed)
    ///             // - And return a value
    ///             
    ///             Ok(PluginValue::Null) // return empty
    ///         }));
    ///     }
    /// }
    /// ```
    ///
    /// This example will register a function with signature "my_func(Array)" in rhai.
    ///
    /// ## Example use in rhai
    ///
    /// ```js
    /// print(my_func(["param1", "param2"]));
    /// ```
    fn register_function(&self, name: &str, handler: NativeFn) -> Result<PluginValue, PluginError>;

    // TODO: Add doc comment here :D
    fn register_config_engine(
        &self, 
        info: ConfigInfo, 
        parser: ParseFn
    ) -> Result<PluginValue, PluginError>;
}

/// The API format that the plugin should follow.
/// This trait should be implemented for a structure and
/// that structure should be exported via FFI.
///
/// ## Example
///
/// ```rust
/// use ewwii_plugin_api::{Plugin, PluginInfo, EwwiiAPI, export_plugin};
///
/// #[derive(Default)]
/// struct MyStruct;
///
/// impl Plugin for MyStruct {
///     fn metadata(&self) -> PluginInfo {
///         /* Dummy Implementation */
///         PluginInfo::new("", "")
///     }
///
///     fn init(&self, host: &dyn EwwiiAPI) {
///         /* Implementation Skipped */   
///      }
/// }
///
/// // Automatically does all the FFI related exports
/// export_plugin!(MyStruct);
/// ```
pub trait Plugin: Send + Sync {
    /// Returns the unique identity and version of the plugin.
    fn metadata(&self) -> PluginInfo;

    /// Function ran by host to startup plugin.
    fn init(&self, host: &dyn EwwiiAPI);
}
