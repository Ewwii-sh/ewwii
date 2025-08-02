/*
    Rhai providers.

    This directory contains the code of all non-widget related functions and modules
    made to make the configuration better for users.
    It is not related to widgets at all and are just there
    for providing data or doing certain actions.

    The providers which start with the word `builtin` are built in functions
    which are equivalent to eww's `magic variables`.

    The providers which end in `module` are rhai modules which contain submodules (fn)
    which can be used for certain tasks.
*/

mod builtin_signals;
mod helper;

use builtin_signals::register_all_signals;

pub fn register_all_providers(engine: &mut rhai::Engine) {
    register_all_signals(engine);
}
