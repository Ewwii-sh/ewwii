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
//! pub struct DummyStructure;
//!
//! impl Plugin for DummyStructure {
//!     fn metadata(&self) -> PluginInfo {
//!         PluginInfo::new("com.app.example", "1.0.0", "Author27");
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
    /// **NOTE:***
    ///
    /// Due to TypeID mismatches, methods like `register_type`, `register_fn`,
    /// etc. won't work on the engine and may cause a crash. It is recommended
    /// to use the `register_function` API to register a funtion which `api::slib`
    /// can call to in rhai.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{EwwiiAPI, Plugin, rhai_backend::RhaiFnNamespace};
    /// use rhai::Dynamic;
    ///
    /// pub struct DummyStructure;
    ///
    /// impl Plugin for DummyStructure {
    ///     fn init(&self, host: &dyn EwwiiAPI) {
    ///         host.register_function(
    ///             "my_func".to_string(),
    ///             RhaiFnNamespace::Global,
    ///             Box::new(|args| {
    ///             // Do stuff
    ///             // - Perform things on the args (if needed)
    ///             // - And return a value
    ///             
    ///             Ok(Dynamic::default()) // return empty
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
        namespace: FnNamespace,
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
/// Returns the unique identity and version of the plugin.
    fn metadata(&self) -> PluginInfo;

    /// Function ran by host to startup plugin.
    fn init(&self, host: &dyn EwwiiAPI);
}
