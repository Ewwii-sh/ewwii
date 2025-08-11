pub mod env;
pub mod text;

use crate::module_resolver::{ChainedResolver, SimpleFileResolver};
use rhai::module_resolvers::StaticModuleResolver;
use rhai::{exported_module, Engine};

pub fn register_stdlib(engine: &mut Engine) {
    use crate::providers::stdlib::{env::env, text::text};

    let mut resolver = StaticModuleResolver::new();

    let chained = ChainedResolver { first: SimpleFileResolver, second: resolver.clone() };

    // adding modules
    let text_mod = exported_module!(text);
    let env_mod = exported_module!(env);

    // inserting modules
    resolver.insert("std::text", text_mod);
    resolver.insert("std::env", env_mod);

    // Register the resolver
    engine.set_module_resolver(chained);
}
