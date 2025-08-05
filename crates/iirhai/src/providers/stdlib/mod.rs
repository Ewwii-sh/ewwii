pub mod text;
pub mod env;

use rhai::{Engine, Module, exported_module};
use rhai::module_resolvers::StaticModuleResolver;
use std::rc::Rc;

pub fn register_stdlib(engine: &mut Engine) {
    use crate::providers::stdlib::{
        text::text,
        env::env,
    };

    let mut resolver = StaticModuleResolver::new();

    // adding modules
    let text_mod = exported_module!(text);
    let env_mod = exported_module!(env);

    // inserting modules
    resolver.insert("std::text", text_mod);
    resolver.insert("std::env", env_mod);

    // Register the resolver
    engine.set_module_resolver(resolver);
}
