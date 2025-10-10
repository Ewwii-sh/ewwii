/*
    Rhai providers.

    This directory contains the code of all non-widget related functions and modules
    made to make the configuration better for users.
    It is not related to widgets at all and are just there
    for providing data or doing certain actions.
*/

mod apilib;
mod stdlib;

use crate::module_resolver::{ChainedResolver, SimpleFileResolver};
use crate::updates::ReactiveVarStore;
use rhai::module_resolvers::StaticModuleResolver;

// expose the api's publically
pub use apilib::register_apilib;
pub use stdlib::register_stdlib;

pub fn register_all_providers(engine: &mut rhai::Engine, plhs: Option<ReactiveVarStore>) {
    let mut resolver = StaticModuleResolver::new();

    // modules
    register_stdlib(&mut resolver);
    register_apilib(&mut resolver);

    let chained = ChainedResolver {
        first: SimpleFileResolver { pl_handler_store: plhs.clone() },
        second: resolver.clone(),
    };
    engine.set_module_resolver(chained);
}
