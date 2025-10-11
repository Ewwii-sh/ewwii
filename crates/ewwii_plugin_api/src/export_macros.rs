//! Module implementing macros

/// Macro to implement and export a plugin in a single step.
/// With this macro, users can write their plugin code directly
/// without having to manually implement each trait.
///
/// ## Example
///
/// The following example shows how you can use this macro to
/// easily make plugins in a single step.
///
/// ```rust
/// auto_plugin!(MyPluginName, {
///     // host variable is passed in automatically
///     host.log("Easy, huh?");
/// })
///
/// ```
///
/// That's it! The plugin is ready.
///
/// ## When not to use it
///
/// This macro shall not be used if you want to have
/// fields in your plugin.
///
/// ```rust
/// struct MyPluginName {
///     awesome_field: String
/// }
/// ```
///
/// For a structure like the above, you should do this instead:
///
/// ```rust
/// use ewwii_plugin_api::{EwwiiAPI, Plugin, export_plugin};
///
/// pub struct DummyStructure;
///
/// impl Plugin for DummyStructure {
///     // critical for ewwii to launch the plugin
///     fn init(&self, host: &dyn EwwiiAPI) {
///         // will be printed by the host
///         host.log("Plugin says Hello!");
///     }
/// }
///
/// // Critical for ewwii to load the plugin
/// export_plugin!(DummyStructure);
/// ```
#[macro_export]
macro_rules! auto_plugin {
    ($struct_name:ident, $init_block:block) => {
        pub struct $struct_name;

        // Implement the Plugin trait
        impl crate::Plugin for $struct_name {
            fn init(&self, host: &dyn crate::EwwiiAPI) {
                $init_block
            }
        }

        export_plugin!($struct_name);
    };
}

/// Automatically implements `create_plugin` for a given fieldless structure
#[macro_export]
macro_rules! export_plugin {
    ($plugin_struct:path) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn create_plugin() -> Box<dyn $crate::Plugin> {
            Box::new($plugin_struct)
        }
    };
}

/// Automatically implements `create_plugin` for a given structure that has fields.
///
/// This macro expects the structure to have fields and also implement a `default()` method.
#[macro_export]
macro_rules! export_stateful_plugin {
    ($plugin_struct:path) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn create_plugin() -> Box<dyn $crate::Plugin> {
            Box::new(<$plugin_struct>::default())
        }
    };
}
