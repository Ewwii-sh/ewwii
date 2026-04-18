//! This module provides plugin requests and host proxy
//! that are used to redirect API calls to host after serialization

use crate::{EwwiiAPI, NativeFn, ParseFn, PluginError, PluginValue, ConfigInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use ewwii_shared_utils::ast::WidgetNode;

/// Represents the different types of callbacks that can be registered by a plugin.
pub enum CallbackHandler {
    NativeFn(NativeFn),
    ParseFn(ParseFn),
}

/// Represents the possible response types returned by a plugin callback.
#[derive(Serialize, Deserialize)]
pub enum CallbackResponse {
    PluginValue(PluginValue),
    WidgetNode(WidgetNode),
    Error(PluginError),
}

static CALLBACKS: OnceLock<Mutex<HashMap<u64, CallbackHandler>>> = OnceLock::new();

/// Helper to get or initialize the map
fn get_callbacks() -> &'static Mutex<HashMap<u64, CallbackHandler>> {
    CALLBACKS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Plugin Requests that needs to be send to host
#[derive(Serialize, Deserialize)]
pub enum PluginRequest {
    // (id, msg)
    Log((String, String)),
    Warn((String, String)),
    Error((String, String)),

    // Complex API
    RegisterFn { id: String, name: String, callback_id: u64 },
    RegisterConfigEngine { id: String, extension: String, main_file: String, callback_id: u64 }
}

// This is provided on the host side
extern "C" {
    fn ffi_gateway(ptr: *const u8, len: usize, out_len: *mut usize) -> *mut u8;
    fn host_free_buffer(ptr: *mut u8, len: usize);
}

#[no_mangle]
pub extern "C" fn plugin_callback_handler(
    id: u64,
    arg_ptr: *const u8,
    arg_len: usize,
    output_len: *mut usize,
) -> *mut u8 {
    let bytes = unsafe { std::slice::from_raw_parts(arg_ptr, arg_len) };
    let callbacks = get_callbacks().lock().unwrap();

    let res_bytes = match callbacks.get(&id) {
        Some(CallbackHandler::NativeFn(f)) => {
            let args: Vec<PluginValue> = bincode::deserialize(bytes).unwrap_or_default();
            let result = f(args).unwrap_or(PluginValue::Null);
            bincode::serialize(&CallbackResponse::PluginValue(result)).unwrap_or_default()
        }
        Some(CallbackHandler::ParseFn(f)) => {
            let (source, path): (String, String) = 
                bincode::deserialize(bytes).unwrap_or_default();
            match f(&source, &path) {
                Ok(node) => bincode::serialize(&CallbackResponse::WidgetNode(node)).unwrap_or_default(),
                Err(e) => bincode::serialize(
                    &CallbackResponse::Error(PluginError::ParseError(e))
                ).unwrap_or_default(),
            }
        }
        None => return std::ptr::null_mut(),
    };

    unsafe {
        let len = res_bytes.len();
        *output_len = len;
        let boxed_slice = res_bytes.into_boxed_slice();
        Box::into_raw(boxed_slice) as *mut u8
    }
}

#[no_mangle]
pub extern "C" fn plugin_free_buffer(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len));
        }
    }
}

/// Proxy that redirects requests to host
pub struct HostProxy {
    id: String,
}

impl HostProxy {
    /// The constructor the macro calls.
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }

    /// Internal helper to get the ID for every request.
    /// This now uses the instance ID instead of a global static.
    fn get_id(&self) -> &str {
        &self.id
    }

    fn call_host(&self, req: PluginRequest) -> Result<PluginValue, PluginError> {
        let bytes =
            bincode::serialize(&req).map_err(|e| PluginError::BridgeError(e.to_string()))?;
        let mut out_len: usize = 0;

        unsafe {
            let res_ptr = ffi_gateway(bytes.as_ptr(), bytes.len(), &mut out_len);
            if res_ptr.is_null() {
                return Err(PluginError::BridgeError("Host gateway returned null".to_string()));
            }

            let res_slice = std::slice::from_raw_parts(res_ptr, out_len);
            let result: Result<PluginValue, PluginError> = bincode::deserialize(res_slice)
                .map_err(|e| PluginError::BridgeError(format!("Deserialization error: {}", e)))?;

            host_free_buffer(res_ptr, out_len);
            result
        }
    }
}

impl EwwiiAPI for HostProxy {
    fn log(&self, msg: &str) {
        let plugid = &self.get_id();
        let req = PluginRequest::Log((plugid.to_string(), msg.to_string()));

        let _ = self.call_host(req);
    }

    fn warn(&self, msg: &str) {
        let plugid = &self.get_id();
        let req = PluginRequest::Warn((plugid.to_string(), msg.to_string()));

        let _ = self.call_host(req);
    }

    fn error(&self, msg: &str) {
        let plugid = &self.get_id();
        let req = PluginRequest::Error((plugid.to_string(), msg.to_string()));

        let _ = self.call_host(req);
    }

    fn register_function(&self, name: &str, handler: NativeFn) -> Result<PluginValue, PluginError> {
        // Register id
        let id = rand::random::<u64>();
        get_callbacks().lock().unwrap().insert(id, CallbackHandler::NativeFn(handler));

        // Send request
        let req = PluginRequest::RegisterFn {
            id: self.get_id().to_string(),
            name: name.to_string(),
            callback_id: id,
        };

        self.call_host(req)
    }

    fn register_config_engine(
        &self, 
        info: ConfigInfo, 
        parser: ParseFn
    ) -> Result<PluginValue, PluginError> {
        // Register id
        let id = rand::random::<u64>();
        get_callbacks().lock().unwrap().insert(id, CallbackHandler::ParseFn(parser));

        // Send request
        let req = PluginRequest::RegisterConfigEngine {
            id: self.get_id().to_string(),
            extension: info.extension.to_string(),
            main_file: info.main_file.to_string(),
            callback_id: id,
        };

        self.call_host(req)
    }
}
