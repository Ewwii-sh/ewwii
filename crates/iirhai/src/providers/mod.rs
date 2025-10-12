/*
    Rhai providers.

    This directory contains the code of all non-widget related functions and modules
    made to make the configuration better for users.
    It is not related to widgets at all and are just there
    for providing data or doing certain actions.
*/

mod builtin_signals;
mod helper;
mod stdlib;

use builtin_signals::register_all_signals;
use stdlib::register_stdlib;

pub fn register_all_providers(engine: &mut rhai::Engine) {
    register_all_signals(engine);

    // modules
    register_stdlib(engine);
}
