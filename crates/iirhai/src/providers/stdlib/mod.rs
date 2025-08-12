pub mod env;
pub mod text;
pub mod monitor;

use crate::module_resolver::{ChainedResolver, SimpleFileResolver};
use rhai::module_resolvers::StaticModuleResolver;
use rhai::{exported_module, Engine};

pub fn register_stdlib(engine: &mut Engine) {
    use crate::providers::stdlib::{env::env, text::text, monitor::monitor};

    let mut resolver = StaticModuleResolver::new();

    // adding modules
    let text_mod = exported_module!(text);
    let env_mod = exported_module!(env);
    let monitor_mod = exported_module!(monitor);

    // inserting modules
    resolver.insert("std::text", text_mod);
    resolver.insert("std::env", env_mod);
    resolver.insert("std::monitor", monitor_mod);

    let chained = ChainedResolver { first: SimpleFileResolver, second: resolver.clone() };

    // Register the resolver
    engine.set_module_resolver(chained);
}
