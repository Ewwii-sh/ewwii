use ewwii_plugin_api::{EwwiiAPI, FnNamespace, NativeFn};
use ewwii_plugin_api::proxy::{PluginRequest};
use crate::config::EWWII_CONFIG_PARSER;

pub(crate) struct EwwiiImpl;

impl EwwiiAPI for EwwiiImpl {
    // General
    // "PCL = Plugin Controlled Log"
    fn log(&self, msg: &str) {
        log::info!("[PCL] {}", msg);
    }

    fn warn(&self, msg: &str) {
        log::warn!("[PCL] {}", msg);
    }

    fn error(&self, msg: &str) {
        log::error!("[PCL] {}", msg);
    }

    fn register_function(
        &self,
        name: &str,
        namespace: FnNamespace,
        handler: NativeFn,
    ) -> Result<(), String> {
        match namespace {
            // TODO
            FnNamespace::Custom(_) => {},
            FnNamespace::Global => {},
        }

        Ok(())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_gateway(ptr: *const u8, len: usize) {
    // SAFETY: Convert the raw pointer/len into a Rust slice
    let bytes = unsafe { 
        std::slice::from_raw_parts(ptr, len) 
    };

    let request: PluginRequest = match bincode::deserialize(bytes) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("[Host] Failed to deserialize plugin request: {}", e);
            return;
        }
    };

    let host = crate::plugin::EwwiiImpl;

    match request {
        PluginRequest::Log((id, msg)) => host.log(&format!("[{}] {}", id, msg)),
        PluginRequest::Warn((id, msg)) => host.warn(&format!("[{}] {}", id, msg)),
        PluginRequest::Error((id, msg)) => host.error(&format!("[{}] {}", id, msg)),
        PluginRequest::RegisterFn { id, name, namespace, callback_id } => {
            // Here you'd call your internal registration logic
            // that handles the Rhai setup using the callback_id.
            // let _ = host.register_function(&name, namespace, callback_id);
        }
    }
}
