use ewwii_plugin_api::EwwiiAPI;
use rhai::Engine;
use rhai_impl::parser::ParseConfig;
use std::sync::{Arc, RwLock};

pub struct EwwiiImpl;

impl EwwiiAPI for EwwiiImpl {
    // General
    fn log(&self, msg: &str) {
        println!("[HOST LOG] {}", msg);
    }
}
