use crate::config::{ConfigEngine, EWWII_CONFIG_PARSER};
use ewwii_plugin_api::proxy::{CallbackResponse, PluginRequest};
use ewwii_plugin_api::{PluginError, PluginValue};
use ewwii_shared_utils::ast::WidgetNode;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::RwLock;

pub fn is_compatible(plugin_ver: &str, host_ver: &str) -> bool {
    let p = semver::Version::parse(plugin_ver);
    let h = semver::Version::parse(host_ver);

    match (p, h) {
        (Ok(p_ver), Ok(h_ver)) => {
            p_ver == h_ver
        }
        _ => {
            false
        }
    }
}
pub struct ActivePlugin {
    pub library: libloading::Library,
    pub id: String,
    pub version: String,
}

pub static ACTIVE_PLUGINS: Lazy<RwLock<Vec<ActivePlugin>>> = Lazy::new(|| RwLock::new(Vec::new()));

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

fn call_plugin_handler(plugin_id: &str, callback_id: u64, arg_bytes: Vec<u8>) -> Option<Vec<u8>> {
    let plugins = ACTIVE_PLUGINS.read().unwrap();
    let plugin = plugins.iter().find(|p| p.id == plugin_id)?;

    unsafe {
        let func: libloading::Symbol<
            unsafe extern "C" fn(u64, *const u8, usize, *mut usize) -> *mut u8,
        > = plugin.library.get(b"plugin_callback_handler").ok()?;

        let mut res_len: usize = 0;
        let res_ptr = func(callback_id, arg_bytes.as_ptr(), arg_bytes.len(), &mut res_len);

        if res_ptr.is_null() {
            return None;
        }

        let res_slice = std::slice::from_raw_parts(res_ptr, res_len);
        let result = res_slice.to_vec();

        if let Ok(free_fn) =
            plugin.library.get::<unsafe extern "C" fn(*mut u8, usize)>(b"plugin_free_buffer")
        {
            free_fn(res_ptr, res_len);
        }

        Some(result)
    }
}

fn trigger_plugin_func_call(plugin_id: &str, callback_id: u64, args: rhai::Array) -> PluginValue {
    let arg_bytes =
        bincode::serialize(&args.into_iter().map(dynamic_to_plugin_value).collect::<Vec<_>>())
            .unwrap_or_default();
    let res = call_plugin_handler(plugin_id, callback_id, arg_bytes).unwrap_or_default();
    bincode::deserialize::<CallbackResponse>(&res)
        .ok()
        .and_then(|r| if let CallbackResponse::PluginValue(v) = r { Some(v) } else { None })
        .unwrap_or(PluginValue::Null)
}

fn trigger_plugin_config_parse(
    plugin_id: &str,
    callback_id: u64,
    source: &str,
    config_path: &str,
) -> Result<WidgetNode, PluginError> {
    let arg_bytes = bincode::serialize(&(source, config_path)).unwrap_or_default();
    let res = call_plugin_handler(plugin_id, callback_id, arg_bytes)
        .ok_or_else(|| "Plugin returned null".to_string())?;
    match bincode::deserialize::<CallbackResponse>(&res).map_err(|e| e.to_string())? {
        CallbackResponse::WidgetNode(node) => Ok(node),
        CallbackResponse::Error(e) => Err(e),
        _ => Err(PluginError::BridgeError("Unexpected response type".to_string())),
    }
}

pub struct CustomConfigEngine {
    id: String,
    extension: String,
    main_file: String,
    callback_id: u64,
}

impl CustomConfigEngine {
    pub fn extension(&self) -> String {
        self.extension.clone()
    }

    pub fn main_file(&self) -> String {
        self.main_file.clone()
    }

    pub fn parse_source(&self, source: String, config_path: PathBuf) -> Result<WidgetNode, String> {
        let path_str = config_path.to_str().unwrap_or("<unknown>");
        trigger_plugin_config_parse(&self.id, self.callback_id, &source, path_str)
            .map_err(|e| e.to_string())
    }
}

pub(crate) struct HostImpl;

impl HostImpl {
    pub fn handle_request(&self, request: PluginRequest) -> Result<PluginValue, PluginError> {
        match request {
            PluginRequest::Log((id, msg)) => {
                log::info!("[{}] {}", id, msg);
                Ok(PluginValue::Null)
            }
            PluginRequest::Warn((id, msg)) => {
                log::warn!("[{}] {}", id, msg);
                Ok(PluginValue::Null)
            }
            PluginRequest::Error((id, msg)) => {
                log::error!("[{}] {}", id, msg);
                Ok(PluginValue::Null)
            }
            PluginRequest::RegisterFn { id, name, callback_id } => {
                if name.trim().is_empty() {
                    return Err(PluginError::RegistrationError(
                        "Function name cannot be empty".into(),
                    ));
                }

                if name.contains(' ') {
                    return Err(PluginError::RegistrationError(
                        "Function names cannot contain spaces".into(),
                    ));
                }

                self.register_function_internal(id, name, callback_id)
            }
            PluginRequest::RegisterConfigEngine { id, extension, main_file, callback_id } => {
                if extension.trim().is_empty() || main_file.trim().is_empty() {
                    return Err(PluginError::RegistrationError(
                        "File extension or main file cannot be empty".into(),
                    ));
                }

                if extension.contains(' ') || main_file.contains(' ') {
                    return Err(PluginError::RegistrationError(
                        "File extension or main file cannot contain spaces".into(),
                    ));
                }

                let custom_engine = CustomConfigEngine { id, extension, main_file, callback_id };

                EWWII_CONFIG_PARSER.with(|p| {
                    *p.borrow_mut() = Some(ConfigEngine::Custom(custom_engine));
                });

                Ok(PluginValue::Null)
            }
        }
    }

    pub fn register_function_internal(
        &self,
        plugin_id: String,
        name: String,
        callback_id: u64,
    ) -> Result<PluginValue, PluginError> {
        EWWII_CONFIG_PARSER.with(|p| {
            let mut parser = p.borrow_mut();

            match parser.as_mut().unwrap() {
                ConfigEngine::Default(rhai) => {
                    rhai.engine.register_fn(
                        &name,
                        move |args: rhai::Array| -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
                            let result = trigger_plugin_func_call(&plugin_id, callback_id, args);

                            Ok(plugin_value_to_dynamic(result))
                        },
                    );

                    Ok(PluginValue::Null)
                },
                ConfigEngine::Custom(_) => Err(PluginError::RegistrationError(
                    "Registering rhai functions is only supported with the Rhai config engine"
                        .to_string()
                )),
            }
        })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_gateway(ptr: *const u8, len: usize, output_len: *mut usize) -> *mut u8 {
    // SAFETY: Convert the raw pointer/len into a Rust slice
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };

    let request: PluginRequest = match bincode::deserialize(bytes) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("[Host] Failed to deserialize plugin request: {}", e);
            return std::ptr::null_mut();
        }
    };

    let host = HostImpl;
    let response = host.handle_request(request);

    let res_bytes = bincode::serialize(&response).unwrap_or_default();
    unsafe {
        *output_len = res_bytes.len();
        let boxed = res_bytes.into_boxed_slice();
        Box::into_raw(boxed) as *mut u8
    }
}

// Way to free
#[unsafe(no_mangle)]
pub extern "C" fn host_free_buffer(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len));
        }
    }
}
