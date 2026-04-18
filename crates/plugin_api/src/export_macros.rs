//! This module implements macros that makes developing plugins easier

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
/// use ewwii_plugin_api::{auto_plugin, PluginInfo};
///
/// auto_plugin!(
///     MyPluginName,
///     PluginInfo::new("com.auto.plugin", "0.1.0"),
///     host, // this host contains the API's
///     {
///         host.log("It's very easy!");
///     }
/// );
/// ```
#[macro_export]
macro_rules! auto_plugin {
    ($struct_name:ident, $metadata:expr, $host_name:ident, $init_block:block) => {
        #[derive(::std::default::Default)]
        pub struct $struct_name;

        impl $crate::Plugin for $struct_name {
            fn metadata(&self) -> $crate::PluginInfo {
                $metadata
            }

            fn init(&self, $host_name: &dyn $crate::EwwiiAPI) {
                $init_block
            }
        }

        $crate::export_plugin!($struct_name);
    };
}

/// Exports the required FFI symbols for the plugin to load
#[macro_export]
macro_rules! export_plugin {
    ($plugin_struct:ty) => {
        #[no_mangle]
        pub extern "C" fn ewwii_plugin_create() -> $crate::PluginInfo {
            let p = <$plugin_struct as ::std::default::Default>::default();
            $crate::Plugin::metadata(&p)
        }

        #[no_mangle]
        pub extern "C" fn ewwii_plugin_init(id_ptr: *const u8, id_len: usize) {
            let id_bytes = unsafe { ::std::slice::from_raw_parts(id_ptr, id_len) };

            let id_cow = ::std::string::String::from_utf8_lossy(id_bytes);
            let id_str: &str = &id_cow;

            let proxy = $crate::proxy::HostProxy::new(id_str);

            let p = <$plugin_struct as ::std::default::Default>::default();
            $crate::Plugin::init(&p, &proxy);
        }
    };
}
