use ewwii_plugin_api::EwwiiAPI;
use rhai::Engine;
use std::sync::mpsc::{channel as mpsc_channel, Receiver, Sender};

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

    // Widget Rendering & Logic
    fn list_widget_ids(&self) -> Result<Vec<u64>, String> {
        let (tx, rx): (Sender<Vec<u64>>, Receiver<Vec<u64>>) = mpsc_channel();

        self.requestor
            .send(PluginRequest::ListWidgetIds(tx))
            .map_err(|_| "Failed to send request to host".to_string())?;

        match rx.recv() {
            Ok(r) => Ok(r),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub(crate) enum PluginRequest {
    RhaiEngineAct(Box<dyn FnOnce(&mut Engine) + Send>),
    ListWidgetIds(Sender<Vec<u64>>),
}
