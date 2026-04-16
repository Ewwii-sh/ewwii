use crate::config::EWWII_CONFIG_PARSER;
use ewwii_plugin_api::{EwwiiAPI, PluginValue, NativeFn};
use ewwii_plugin_api::proxy::PluginRequest;
use once_cell::sync::Lazy;
use std::sync::RwLock;

pub struct ActivePlugin {
    pub library: libloading::Library,
    pub id: String,
    pub version: String,
}

pub static ACTIVE_PLUGINS: Lazy<RwLock<Vec<ActivePlugin>>> = Lazy::new(|| {
    RwLock::new(Vec::new())
});

fn dynamic_to_plugin_value(any: rhai::Dynamic) -> PluginValue {
    if any.is_unit() {
        return PluginValue::Null;
    }
    if let Some(v) = any.clone().try_cast::<String>() {
        return PluginValue::String(v);
    }
    if let Some(v) = any.clone().try_cast::<i64>() {
        return PluginValue::Int(v);
    }
    if let Some(v) = any.clone().try_cast::<f64>() {
        return PluginValue::Float(v);
    }
    if let Some(v) = any.clone().try_cast::<bool>() {
        return PluginValue::Bool(v);
    }
    if let Some(v) = any.clone().try_cast::<rhai::Array>() {
        return PluginValue::Array(v.into_iter().map(dynamic_to_plugin_value).collect());
    }
    
    PluginValue::Null
}

fn plugin_value_to_dynamic(val: PluginValue) -> rhai::Dynamic {
    match val {
        PluginValue::String(s) => s.into(),
        PluginValue::Int(i) => i.into(),
        PluginValue::Float(f) => f.into(),
        PluginValue::Bool(b) => b.into(),
        PluginValue::Array(arr) => {
            let vec: Vec<rhai::Dynamic> = arr.into_iter().map(plugin_value_to_dynamic).collect();
            vec.into()
        }
        PluginValue::Null => rhai::Dynamic::UNIT,
    }
}

fn trigger_plugin_callback(
    plugin_id: &str, 
    callback_id: u64, 
    args: rhai::Array,
) -> PluginValue {
    let plugins = ACTIVE_PLUGINS.read().unwrap();
    
    if let Some(plugin) = plugins.iter().find(|p| p.id == plugin_id) {
        let plugin_args: Vec<PluginValue> = args.into_iter().map(dynamic_to_plugin_value).collect();
        let arg_bytes = bincode::serialize(&plugin_args).unwrap_or_default();

        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(u64, *const u8, usize, *mut usize) -> *mut u8> = 
                plugin.library.get(b"plugin_callback_handler").unwrap();
            
            let mut res_len: usize = 0;
            let res_ptr = func(callback_id, arg_bytes.as_ptr(), arg_bytes.len(), &mut res_len);

            if res_ptr.is_null() {
                return PluginValue::Null;
            }

            let res_slice = std::slice::from_raw_parts(res_ptr, res_len);
            let result: PluginValue = bincode::deserialize(res_slice).unwrap_or(PluginValue::Null);

            // cleanup
            if let Ok(free_fn) = plugin.library.get::<unsafe extern "C" fn(*mut u8, usize)>(b"ewwii_free_buffer") {
                free_fn(res_ptr, res_len);
            }

            return result;
        }
    }
    log::error!("[Ewwii Plugin Handler] Could not find plugin {} in ACTIVE_PLUGINS", plugin_id);
    PluginValue::Null
}

pub(crate) struct EwwiiImpl;

impl EwwiiImpl {
    pub fn register_function_internal(
        &self, 
        plugin_id: String, 
        name: String, 
        callback_id: u64
    ) {
        EWWII_CONFIG_PARSER.with(|p| {
            let mut parser = p.borrow_mut();
            let parser_ref = parser.as_mut().unwrap();

            parser_ref.engine.register_fn(&name, move |args: rhai::Array| 
                -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> 
            {
                let result = trigger_plugin_callback(&plugin_id, callback_id, args);

                Ok(plugin_value_to_dynamic(result))
            });
        });
    }
}

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
        _name: &str,
        _handler: NativeFn,
    ) -> Result<(), String> {
        // NoOp
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
        PluginRequest::RegisterFn { id, name, callback_id } => {
            let exists = {
                let plugins = ACTIVE_PLUGINS.read().unwrap();
                plugins.iter().any(|p| p.id == id)
            };

            if !exists {
                log::error!("Plugin {} tried to register a function but isn't loaded!", id);
                return;
            }

            host.register_function_internal(id, name, callback_id);
        }
    }
}
