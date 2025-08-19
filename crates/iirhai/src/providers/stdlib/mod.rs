pub mod env;
pub mod monitor;
pub mod text;
pub mod json;

use crate::module_resolver::{ChainedResolver, SimpleFileResolver};
use rhai::module_resolvers::StaticModuleResolver;
use rhai::{exported_module, Engine};

pub fn register_stdlib(engine: &mut Engine) {
    use crate::providers::stdlib::{
        env::env, 
        monitor::monitor, 
        text::text,
        json::json,
    };

    let mut resolver = StaticModuleResolver::new();

    // adding modules
    let text_mod = exported_module!(text);
    let env_mod = exported_module!(env);
    let monitor_mod = exported_module!(monitor);
    let json_mod = exported_module!(json);

    // inserting modules
    resolver.insert("std::text", text_mod);
    resolver.insert("std::env", env_mod);
    resolver.insert("std::monitor", monitor_mod);
    resolver.insert("std::json", json_mod);

    let chained = ChainedResolver { first: SimpleFileResolver, second: resolver.clone() };

    // Register the resolver
    engine.set_module_resolver(chained);
}
