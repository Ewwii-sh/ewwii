//! This module provides plugin requests and host proxy
//! that are used to redirect API calls to host after serialization

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use crate::{NativeFn, EwwiiAPI, PluginValue, PluginError};
use serde::{Serialize, Deserialize};

static CALLBACKS: OnceLock<Mutex<HashMap<u64, NativeFn>>> = OnceLock::new();

/// Helper to get or initialize the map
fn get_callbacks() -> &'static Mutex<HashMap<u64, NativeFn>> {
    CALLBACKS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Plugin Requests that needs to be send to host
#[derive(Serialize, Deserialize)]
pub enum PluginRequest {
    // (id, msg)
    Log((String, String)),
    Warn((String, String)),
    Error((String, String)),

    RegisterFn {
        id: String,
        name: String,
        callback_id: u64,
    },
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
    output_len: *mut usize
) -> *mut u8 {
    let bytes = unsafe { std::slice::from_raw_parts(arg_ptr, arg_len) };
    let args: Vec<PluginValue> = bincode::deserialize(bytes).unwrap_or_default();

    let callbacks = get_callbacks().lock().unwrap();
    if let Some(handler) = callbacks.get(&id) {
        let result = handler(args).unwrap_or(PluginValue::Null);

        let res_bytes = bincode::serialize(&result).unwrap_or_default();
        
        unsafe {
            let len = res_bytes.len();
            *output_len = len;
            
            let boxed_slice = res_bytes.into_boxed_slice();
            return Box::into_raw(boxed_slice) as *mut u8;
        }
    }
    
    std::ptr::null_mut()
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
        Self {
            id: id.to_string(),
        }
    }

    /// Internal helper to get the ID for every request.
    /// This now uses the instance ID instead of a global static.
    fn get_id(&self) -> &str {
        &self.id
    }

    fn call_host(&self, req: PluginRequest) -> Result<PluginValue, PluginError> {
        let bytes = bincode::serialize(&req)
            .map_err(|e| PluginError::BridgeError(e.to_string()))?;
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

    fn register_function(
        &self,
        name: &str,
        handler: NativeFn,
    ) -> Result<PluginValue, PluginError> {
        // Register id
        let id = rand::random::<u64>();
        get_callbacks().lock().unwrap().insert(id, handler);

        // Send request
        let req = PluginRequest::RegisterFn {
            id: self.get_id().to_string(),
            name: name.to_string(),
            callback_id: id,
        };

        self.call_host(req)
    }
}