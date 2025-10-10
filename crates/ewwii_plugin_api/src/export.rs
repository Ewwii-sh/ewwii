//! A module implementing macros and similar functions
//! for exporting a module. All the macros defined here
//! are acessable at the root of this crate.
//!
//! This module also provides an example structure called
//! [`ExamplePlugin`], which can be exported directly.

/// An example plugin that can be exported directly
pub struct ExamplePlugin;

impl crate::Plugin for ExamplePlugin {
    fn init(&self, host: &dyn crate::EwwiiAPI) {
        host.log("Example plugin says Hello!");
    }
}

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
