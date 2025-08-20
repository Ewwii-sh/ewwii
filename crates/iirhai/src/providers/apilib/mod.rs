pub mod wifi;

use rhai::exported_module;
use rhai::module_resolvers::StaticModuleResolver;

pub fn register_apilib(resolver: &mut StaticModuleResolver) {
    use crate::providers::apilib::wifi::wifi;

    // adding modules
    let wifi_mod = exported_module!(wifi);

    // inserting modules
    resolver.insert("api::wifi", wifi_mod);
}
