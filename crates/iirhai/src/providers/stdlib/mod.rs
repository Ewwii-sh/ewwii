pub mod env;
pub mod json;
pub mod monitor;
pub mod text;

use rhai::exported_module;
use rhai::module_resolvers::StaticModuleResolver;

pub fn register_stdlib(resolver: &mut StaticModuleResolver) {
    use crate::providers::stdlib::{env::env, json::json, monitor::monitor, text::text};

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
}
