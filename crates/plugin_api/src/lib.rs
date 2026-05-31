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

pub use bridge::*;
pub use ewwii_shared_utils as shared_utils;

pub const API_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");

/// The shared trait defining the Ewwii plugin API
pub trait EwwiiAPI: Send + Sync {
    // == General Stuff == //
    /// Log a message from the host
    fn log(&self, msg: &str);
    /// Log a warning from the host
    fn warn(&self, msg: &str);
    /// Log an error from the host
    fn error(&self, msg: &str);

    // === IPC, I guess === //

    /// Send an IPC request to ewwii.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{
    ///     auto_plugin, PluginInfo,
    ///     IpcRequest, WidgetControlType
    /// };
    ///
    /// auto_plugin!(
    ///     DummyStructure,
    ///     PluginInfo::new("test.example.ipc", "1.0.0"),
    ///     host,
    ///     {
    ///         host.ipc_request(IpcRequest::WidgetControl(WidgetControlType::PropertyUpdate {
    ///             widget: "my_widget".to_string(),
    ///             prop: "label".to_string(),
    ///             value: "Hello, World".to_string()
    ///         }));
    ///     }
    /// );
    /// ```
    fn ipc_request(&self, req: IpcRequest);

    // === Registration Stuff === //

    /// Expose a function that nbcl configuration can call.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{
    ///     auto_plugin, PluginInfo,
    ///     NativeFn, NativeFnExt,
    ///     NbclType, PluginValue
    /// };
    ///
    /// auto_plugin!(
    ///     DummyStructure,
    ///     PluginInfo::new("test.example.ipc", "1.0.0"),
    ///     host,
    ///     {
    ///         host.register_function(
    ///             "my_func",              // function name
    ///             vec![NbclType::String], // args we receive
    ///             NbclType::Null,         // type we return
    ///             NativeFn::new(|args| {
    ///                 // Do stuff
    ///                 // - Perform things on the args (if needed)
    ///                 // - And return a value
    ///
    ///                 Ok(PluginValue::Null) // return empty (must match provided return type)
    ///         }));
    ///     }
    /// );
    /// ```
    ///
    /// This example will register a function with signature "my_func(List)" in nbcl.
    ///
    /// ## Example use in nbcl
    ///
    /// ```js
    /// print(my_func("param"));
    /// ```
    fn register_function(
        &self,
        name: &str,
        types: Vec<NbclType>,
        return_type: NbclType,
        handler: NativeFn,
    );

    /// Replace nbcl with a custom configuration engine.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{
    ///     auto_plugin, PluginInfo,
    ///     ConfigInfo, ParseFn,
    ///     ParseFnExt, shared_utils::ast::WidgetNode
    /// };
    ///
    /// auto_plugin!(
    ///     DummyStructure,
    ///     PluginInfo::new("test.example.ipc", "1.0.0"),
    ///     host,
    ///     {
    ///         // example language: Lua
    ///         host.register_config_engine(
    ///             ConfigInfo {
    ///                 extension: "lua",
    ///                 main_file: "main.lua",
    ///
    ///             },
    ///             ParseFn::new(|source, path| {
    ///             // source (&str) - source code of main.lua
    ///             // path (&str) - path to main.lua
    ///
    ///             // Parse Lua and construct WidgetNode
    ///
    ///             Ok(WidgetNode::Tree(vec![])) // returning Dummy for now
    ///         }));
    ///     }
    /// );
    /// ```
    fn register_config_engine(&self, info: ConfigInfo, parser: ParseFn);

    // === Dynamic Runtime === //

    /// Inject custom CSS into the engine.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{
    ///     auto_plugin, PluginInfo,
    /// };
    ///
    /// auto_plugin!(
    ///     DummyStructure,
    ///     PluginInfo::new("test.example.ipc", "1.0.0"),
    ///     host,
    ///     {
    ///         host.inject_css("* { all: unset }".to_string());
    ///     }
    /// );
    /// ```
    fn inject_css(&self, css: String);

    // === Handlers === //

    /// Handle callbacks registered by the config engine.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{
    ///     auto_plugin, PluginInfo,
    ///     ConfigCallbackFn,
    ///     ConfigCallbackFnExt
    /// };
    ///
    /// auto_plugin!(
    ///     DummyStructure,
    ///     PluginInfo::new("test.example.ipc", "1.0.0"),
    ///     host,
    ///     {
    ///         host.handle_config_callbacks(ConfigCallbackFn::new(|name, id| {
    ///             // handle here
    ///         }));
    ///     }
    /// );
    /// ```
    fn handle_config_callbacks(&self, handle: ConfigCallbackFn);
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
