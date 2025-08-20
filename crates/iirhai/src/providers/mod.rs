/*
    Rhai providers.

    This directory contains the code of all non-widget related functions and modules
    made to make the configuration better for users.
    It is not related to widgets at all and are just there
    for providing data or doing certain actions.
*/

mod apilib;
mod builtin_signals;
mod helper;
mod stdlib;

use crate::module_resolver::{ChainedResolver, SimpleFileResolver};
use rhai::module_resolvers::StaticModuleResolver;

use apilib::register_apilib;
use builtin_signals::register_all_signals;
use stdlib::register_stdlib;

pub fn register_all_providers(engine: &mut rhai::Engine) {
    let mut resolver = StaticModuleResolver::new();

    register_all_signals(&mut resolver);

    // modules
    register_stdlib(&mut resolver);
    register_apilib(&mut resolver);

    let chained = ChainedResolver { first: SimpleFileResolver, second: resolver.clone() };
    engine.set_module_resolver(chained);
}
