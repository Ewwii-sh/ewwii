//! A module implementing macros for exporting a plugin.
//! It allows fast and easy export of plugins by abstracting
//! the FFI (Foreign Function Interface) related code needed.

/// Automatically implements `create_plugin` for a given fieldless structure
#[macro_export]
macro_rules! export_plugin {
    ($plugin_struct:ty) => {
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
    ($plugin_struct:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn create_plugin() -> Box<dyn $crate::Plugin> {
            Box::new(<$plugin_struct>::default())
        }
    };
}
