pub mod command;
pub mod env;
pub mod monitor;
pub mod regex;
pub mod text;

use rhai::exported_module;
use rhai::module_resolvers::StaticModuleResolver;

pub fn register_stdlib(resolver: &mut StaticModuleResolver) {
    use crate::providers::stdlib::{
        command::command, env::env, monitor::monitor, regex::regex_lib, text::text,
    };

    // adding modules
    let text_mod = exported_module!(text);
    let env_mod = exported_module!(env);
    let monitor_mod = exported_module!(monitor);
    let command_mod = exported_module!(command);
    let regex_mod = exported_module!(regex_lib);

    // inserting modules
    resolver.insert("std::text", text_mod);
    resolver.insert("std::env", env_mod);
    resolver.insert("std::monitor", monitor_mod);
    resolver.insert("std::command", command_mod);
    resolver.insert("std::regex", regex_mod);
}
