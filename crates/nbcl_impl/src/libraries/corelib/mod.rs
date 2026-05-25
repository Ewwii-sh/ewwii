mod command;
mod env;
mod regex;

use nbcl::{NbclEngine, Type, library::{Library, LibraryItem}};

pub fn register_core_lib(engine: &mut NbclEngine) {
    let command = LibraryItem::define("command")
        .with_fn("run", vec![Type::Str], Type::Null, command::run)
        .with_fn("run_and_read", vec![Type::Str], Type::Str, command::run_and_read);

    let env = LibraryItem::define("env")
        .with_fn("set_env", vec![Type::Str, Type::Str], Type::Null, env::set_env)
        .with_fn("get_env", vec![Type::Str], Type::Str, env::get_env)
        .with_fn("get_current_dir", vec![], Type::Str, env::get_current_dir)
        .with_fn("get_username", vec![], Type::Str, env::get_username)
        .with_fn("get_homedir", vec![], Type::Str, env::get_home_dir);

    let regex = LibraryItem::define("regex")
        .with_fn("is_match", vec![Type::Str, Type::Str], Type::Bool, regex::is_match)
        .with_fn("find", vec![Type::Str, Type::Str], Type::Str, regex::find)
        .with_fn("find_all", vec![Type::Str, Type::Str], Type::List, regex::find_all)
        .with_fn("replace", vec![Type::Str, Type::Str, Type::Str], Type::Str, regex::replace);

    let core_lib = Library::new("core".into(), vec![command, env, regex]);
    engine.register_library(core_lib);
}
