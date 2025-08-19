pub mod wifi;

use crate::module_resolver::{ChainedResolver, SimpleFileResolver};
use rhai::module_resolvers::StaticModuleResolver;
use rhai::{exported_module, Engine};

pub fn register_apilib(engine: &mut Engine) {
    use crate::providers::apilib::wifi::wifi;

    let mut resolver = StaticModuleResolver::new();

    // adding modules
    let wifi_mod = exported_module!(wifi);

    // inserting modules
    resolver.insert("api::wifi", wifi_mod);

    let chained = ChainedResolver { first: SimpleFileResolver, second: resolver.clone() };

    // Register the resolver
    engine.set_module_resolver(chained);
}
