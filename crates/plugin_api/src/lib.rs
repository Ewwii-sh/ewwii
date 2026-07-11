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
//! Either way, you will be working with the [`EwwiiAPI`] trait.
//!
//! ### 1. Recommended: Using `auto_plugin!`
//!
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
//!         // host is 'EwwiiAPI'
//!         host.log("Plugin says Hello!");
//!     }
//! );
//! ```
//!
//! ### 2. Advanced: Manual Implementation
//!
//! You generally would never need to use the manual approach unless you **have** to store someting
//! inside the plugin structure.
//!
//! ```rust
//! use ewwii_plugin_api::{EwwiiAPI, Plugin, PluginInfo, export_plugin};
//! use std::sync::Arc;
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
//!     fn init(&self, host: Arc<dyn EwwiiAPI>) {
//!         host.log("Manual plugin initialized.");
//!     }
//! }
//!
//! // This macro exports the C-compatible symbols required by the host loader
//! export_plugin!(MyPlugin);
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

mod bridge;
mod export_macros;

pub mod proxy;

pub use bridge::*;
pub use ewwii_shared_utils as shared_utils;

pub const API_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");

/// The shared trait defining the Ewwii plugin API
pub trait EwwiiAPI: Send + Sync {
    // == General Stuff == //
    fn metadata_id(&self) -> String;

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
    /// **Example 1:**
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
    ///
    /// **Example 2:**
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
    ///         let future_result = host.ipc_request(IpcRequest::WidgetControl(WidgetControlType::PropertyGet {
    ///             widget: "my_widget".to_string(),
    ///             prop: "label".to_string(),
    ///         }));
    ///
    ///         future_result.resolve_async(|res| {
    ///             if res.is_ok() {
    ///                 // do stuff with result...
    ///             }
    ///         });
    ///     }
    /// );
    /// ```
    ///
    /// WARNING: Trying to resolve a future_result if the ipc request is not the type that returns
    /// something may result in a deadlock.
    fn ipc_request(&self, req: IpcRequest) -> FutureResult<String>;

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

    /// Register a library that can be imported in Nbcl.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{
    ///     auto_plugin, PluginValue,
    ///     PluginInfo, NativeFn,
    ///     NativeFnExt, NbclType,
    ///     LibraryItem
    /// };
    ///
    /// auto_plugin!(
    ///     DummyStructure,
    ///     PluginInfo::new("test.example.library", "1.0"),
    ///     host,
    ///     {
    ///         host.register_library(
    ///             "example",
    ///             vec![
    ///                 LibraryItem::define("foo")
    ///                     .with_fn(
    ///                         "greet",
    ///                         vec![NbclType::String],
    ///                         NbclType::Null,
    ///                         NativeFn::new(|args| {
    ///                             let PluginValue::String(ref name) = args[0] else {
    ///                                 // guranteed to be string
    ///                                 unreachable!();
    ///                             };
    ///                             println!("Hello, {}!", name);
    ///                             Ok(PluginValue::Null) // return empty (must match provided return type)
    ///                         })
    ///                     )
    ///             ]
    ///         );
    ///     }
    /// );
    ///
    /// ```
    ///
    /// This example will register a library called 'example' with item 'foo'
    /// which can be imported like so in nbcl:
    ///
    /// ```js
    /// import example.foo
    /// foo.greet("Bob")
    /// ```
    fn register_library(&self, name: &str, items: Vec<LibraryItem>);

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

    /// Register a static widget into ewwii.
    ///
    /// A static widget is a simple widget that does not accept any
    /// properties or children provided by ewwii and runs entirely on its own terms. It can be used
    /// for widgets thats only purpose is showing data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{auto_plugin, PluginInfo, gtk4};
    ///
    /// auto_plugin!(
    ///     DummyStructure,
    ///     PluginInfo::new("test.example.static-widget", "1.0.0"),
    ///     host,
    ///     {
    ///         let my_box = gtk4::Box::default();
    ///         let box_general = my_box.upcast::<gtk4::Widget>();
    ///         host.register_static_widget("awesome-widget", box_general);
    ///     }
    /// );
    /// ```
    #[cfg(feature = "widgets")]
    #[cfg_attr(docsrs, doc(cfg(feature = "widgets")))]
    fn register_static_widget(&self, name: &str, widget: gtk4::Widget);

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
    ///     PluginInfo::new("test.example.inject", "1.0.0"),
    ///     host,
    ///     {
    ///         host.inject_css("* { all: unset }");
    ///     }
    /// );
    /// ```
    fn inject_css(&self, css: &str) -> FutureResult<u64>;

    /// Remove a CSS injected into the engine.
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
    ///     PluginInfo::new("test.example.remove-inject", "1.0.0"),
    ///     host,
    ///     {
    ///         let future_idx = host.inject_css("* { all: unset }");
    ///
    ///         let host_clone = host.clone();
    ///         future_idx.resolve_async(move |res| {
    ///             if let Ok(idx) = res {
    ///                 host_clone.remove_css(idx);
    ///             }
    ///         });
    ///     }
    /// );
    /// ```
    fn remove_css(&self, idx: u64);

    /// Inject to bootstrap before every source.
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
    ///     PluginInfo::new("test.example.bootstrap", "1.0.0"),
    ///     host,
    ///     {
    ///         host.inject_nbcl_bootstrap("print('Hi!')");
    ///     }
    /// );
    /// ```
    fn inject_nbcl_bootstrap(&self, source: &str);

    /// Emit a message to a buffer which other plugins can see.
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
    ///     PluginInfo::new("test.example.emit", "1.0.0"),
    ///     host,
    ///     {
    ///         let data = "secert-data".to_string(); // or can be json
    ///         host.emit("emit-loaded", data);
    ///     }
    /// );
    /// ```
    fn emit(&self, signal: &str, data: String);

    /// Listen to a message emitted by a plugin or ewwii in the buffer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{
    ///     auto_plugin, PluginInfo,
    ///     ListenHandleFn, ListenHandleFnExt
    /// };
    ///
    /// auto_plugin!(
    ///     DummyStructure,
    ///     PluginInfo::new("test.example.emitlisten", "1.0.0"),
    ///     host,
    ///     {
    ///         let host_clone = host.clone();
    ///         host.listen("emit-loaded", ListenHandleFn::new(move |info| {
    ///             println!("{}", info.pid); // Plugin ID (Verify its right plugin)
    ///             println!("{}", info.data); // Data it emitted
    ///
    ///             // example of doing host calls:
    ///             host_clone.emit("emit-loaded-received", "received!".to_string());
    ///         }));
    ///     }
    /// );
    /// ```
    ///
    /// # Ewwii Emissions
    ///
    /// Ewwii too emits messages to the plugin buffer to let the plugins know when a event has
    /// occourred. These are all the messages ewwii can emit:
    ///
    /// - ewwii-config-loaded
    /// - ewwii-applied-styles
    /// - ewwii-started-signals
    /// - ewwii-init-window
    /// - ewwii-reloaded-windows
    ///
    fn listen(&self, signal: &str, handle: ListenHandleFn);

    /// Register a signal into ewwii.
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
    ///     PluginInfo::new("test.example.signal", "1.0.0"),
    ///     host,
    ///     {
    ///         host.register_signal("example", "initial value".to_string());
    ///     }
    /// );
    /// ```
    fn register_signal(&self, name: &str, initial: String);

    /// Update the value of a signal.
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
    ///     PluginInfo::new("test.example.signal", "1.0.0"),
    ///     host,
    ///     {
    ///         host.update_signal("example", "new val".to_string());
    ///     }
    /// );
    /// ```
    fn update_signal(&self, name: &str, value: String);

    /// Run a function when a signal updates.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ewwii_plugin_api::{
    ///     auto_plugin, PluginInfo,
    ///     SignalUpdateFn, SignalUpdateFnExt
    /// };
    ///
    /// auto_plugin!(
    ///     DummyStructure,
    ///     PluginInfo::new("test.example.signal", "1.0.0"),
    ///     host,
    ///     {
    ///         host.on_signal_update("example", SignalUpdateFn::new(|val| {
    ///             // 'val' is the value of the signal now
    ///             // stuff can be done here...
    ///         }));
    ///     }
    /// );
    /// ```
    fn on_signal_update(&self, name: &str, handle: SignalUpdateFn);

    /// Get the value of a signal
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
    ///     PluginInfo::new("test.example.signal", "1.0.0"),
    ///     host,
    ///     {
    ///         let future_result = host.signal_value("example");
    ///         future_result.resolve_async(move |res| {
    ///             if res.is_ok() {
    ///                 // ...
    ///             }
    ///         });
    ///     }
    /// );
    /// ```
    fn signal_value(&self, name: &str) -> FutureResult<String>;

    /// Get the runtime paths like configuration directories and ipc socket file.
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
    ///     PluginInfo::new("test.example.paths", "1.0.0"),
    ///     host,
    ///     {
    ///         let frt_paths = host.get_runtime_paths();
    ///         frt_paths.resolve_async(move |res| {
    ///             if res.is_ok() {
    ///                 // ...
    ///             }
    ///         });
    ///     }
    /// );
    /// ```
    fn get_runtime_paths(&self) -> FutureResult<RuntimePaths>;

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
/// use std::sync::Arc;
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
///     fn init(&self, host: Arc<dyn EwwiiAPI>) {
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
    fn init(&self, host: std::sync::Arc<dyn EwwiiAPI>);
}
