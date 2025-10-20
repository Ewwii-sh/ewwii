use ewwii_plugin_api::{widget_backend, EwwiiAPI};
use rhai::{Engine, Dynamic, Array};
use std::sync::mpsc::{channel as mpsc_channel, Receiver, Sender};

pub(crate) struct EwwiiImpl {
    pub(crate) requestor: Sender<PluginRequest>,
}

impl EwwiiAPI for EwwiiImpl {
    // General
    // "PCL = Plugin Controlled Log"
    fn print(&self, msg: &str) {
        println!("[PCL] {}", msg);
    }

    fn log(&self, msg: &str) {
        log::info!("[PCL] {}", msg);
    }

    fn warn(&self, msg: &str) {
        log::warn!("[PCL] {}", msg);
    }

    fn error(&self, msg: &str) {
        log::error!("[PCL] {}", msg);
    }

    // Rhai Manipulation Stuff
    fn rhai_engine_action(&self, f: Box<dyn FnOnce(&mut Engine) + Send>) -> Result<(), String> {
        self.requestor
            .send(PluginRequest::RhaiEngineAct(f))
            .map_err(|_| "Failed to send request to host".to_string())?;
        Ok(())
    }

    fn register_function(
        &self, 
        name: String,
        f: Box<dyn FnOnce(rhai::Array) -> Dynamic + Send>,
    ) -> Result<(), String> {
        let func_info = (name, f);
        
        self.requestor
            .send(PluginRequest::RegisterFunc(func_info))
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

    fn widget_reg_action(
        &self,
        f: Box<dyn FnOnce(&mut widget_backend::WidgetRegistryRepr) + Send>,
    ) -> Result<(), String> {
        self.requestor
            .send(PluginRequest::WidgetRegistryAct(f))
            .map_err(|_| "Failed to send request to host".to_string())?;
        Ok(())
    }
}

pub(crate) enum PluginRequest {
    RhaiEngineAct(Box<dyn FnOnce(&mut Engine) + Send>),
    RegisterFunc((String, Box<dyn FnOnce(Array) -> Dynamic + Send>)),
    ListWidgetIds(Sender<Vec<u64>>),
    WidgetRegistryAct(Box<dyn FnOnce(&mut widget_backend::WidgetRegistryRepr) + Send>),
}
