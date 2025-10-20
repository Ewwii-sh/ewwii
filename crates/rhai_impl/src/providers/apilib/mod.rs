pub mod linux;
pub mod wifi;
pub mod slib;

use rhai::exported_module;
use rhai::module_resolvers::StaticModuleResolver;

pub fn register_apilib(resolver: &mut StaticModuleResolver) {
    use crate::providers::apilib::{linux::linux, wifi::wifi, slib::slib};

    // adding modules
    let wifi_mod = exported_module!(wifi);
    let linux_mod = exported_module!(linux);
    let slib_mod = exported_module!(slib);

    // inserting modules
    resolver.insert("api::wifi", wifi_mod);
    resolver.insert("api::linux", linux_mod);
    resolver.insert("api::slib", slib_mod);
}