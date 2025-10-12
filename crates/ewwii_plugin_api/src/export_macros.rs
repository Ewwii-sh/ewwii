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
/// ```rust,no_run
/// use ewwii_plugin_api::auto_plugin;
///
/// auto_plugin!(MyPluginName, {
///     // host variable is passed in automatically
///     host.log("Easy, huh?");
/// });
/// ```
///
/// ## When not to use it
///
/// This macro shall not be used if you prefer flexibility and safety.
/// The manual approach is verbose, but is way safer and flexible than using this macro.
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

        crate::export_plugin!($struct_name);
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
