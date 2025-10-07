use ewwii_plugin_api::{EwwiiAPI, Plugin};

pub struct EwwiiImpl;

impl HostAPI for EwwiiImpl {
	// General
    fn log(&self, msg: &str) {
        println!("[HOST LOG] {}", msg);
    }

    // Rhai Engine Stuff
    fn get_rhai_engine(&self) {

    }

    fn set_rhai_engine(&self, engine: &Engine) {

    }
}
