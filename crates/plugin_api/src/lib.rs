//! # ewwii_plugin_api - A plugin interface for ewwii
//!
//! `ewwii_plguin_api` is a shared list of traits
//! that both ewwii and its plugins can use.
//! This crate simplifies and provides a safe way for building
//! plugins for ewwii.
//!
//! ## Example
//!
//! The following example shows how this crate shall be used to build ewwii plugins:
//!
//! ```rust
//! use ewwii_plugin_api::{
//!     EwwiiAPI, Plugin, PluginInfo, 
//!     export_plugin, proxy::HostProxy
//! };
//!
//! #[derive(Default)]
//! pub struct DummyStructure;
//!
//! impl Plugin for DummyStructure {
//!     fn metadata(&self) -> PluginInfo {
//!         PluginInfo::new("com.app.example", "1.0.0")
//!     }
//!
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
mod bridge;

pub mod example;
pub mod proxy;

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
    fn register_function(
        &self,
        name: &str,
        handler: NativeFn,
    ) -> Result<(), String>;
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
