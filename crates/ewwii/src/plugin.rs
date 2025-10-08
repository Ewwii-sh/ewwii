use ewwii_plugin_api::EwwiiAPI;
use rhai::Engine;
use std::sync::mpsc::Sender;

pub(crate) struct EwwiiImpl {
    pub(crate) requestor: Sender<PluginRequest>,
}

impl EwwiiAPI for EwwiiImpl {
    // General
    fn log(&self, msg: &str) {
        println!("[HOST LOG] {}", msg);
    }

    // Rhai Manipulation Stuff
    fn rhai_engine_action(&self, f: Box<dyn FnOnce(&mut Engine) + Send>) -> Result<(), String> {
        self.requestor
            .send(PluginRequest::RhaiEngineAct(f))
            .map_err(|_| "Failed to send request to host".to_string())?;
        Ok(())
    }
}

pub(crate) enum PluginRequest {
    RhaiEngineAct(Box<dyn FnOnce(&mut Engine) + Send>),
}
