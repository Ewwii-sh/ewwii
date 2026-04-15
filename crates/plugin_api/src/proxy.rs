//! This module provides plugin requests and host proxy
//! that are used to redirect API calls to host after serialization

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use crate::{NativeFn, FnNamespace, EwwiiAPI};
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
        namespace: FnNamespace,
        callback_id: u64,
    },
}

// This is provided on the host side
extern "C" {
    fn ffi_gateway(ptr: *const u8, len: usize);
}

#[no_mangle]
pub extern "C" fn plugin_callback_handler(id: u64, arg_ptr: *const u8, arg_len: usize) {
    let bytes = unsafe { std::slice::from_raw_parts(arg_ptr, arg_len) };
    let args = bincode::deserialize(bytes).unwrap();

    let callbacks = get_callbacks().lock().unwrap();
    if let Some(handler) = callbacks.get(&id) {
        let _ = handler(args); 
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
}

impl EwwiiAPI for HostProxy {
    fn log(&self, msg: &str) {
        let plugid = &self.get_id();
        let req = PluginRequest::Log((plugid.to_string(), msg.to_string()));
        if let Ok(bytes) = bincode::serialize(&req) {
            unsafe { ffi_gateway(bytes.as_ptr(), bytes.len()); }
        }
    }

    fn warn(&self, msg: &str) {
        let plugid = &self.get_id();
        let req = PluginRequest::Warn((plugid.to_string(), msg.to_string()));
        if let Ok(bytes) = bincode::serialize(&req) {
            unsafe { ffi_gateway(bytes.as_ptr(), bytes.len()); }
        }
    }

    fn error(&self, msg: &str) {
        let plugid = &self.get_id();
        let req = PluginRequest::Error((plugid.to_string(), msg.to_string()));
        if let Ok(bytes) = bincode::serialize(&req) {
            unsafe { ffi_gateway(bytes.as_ptr(), bytes.len()); }
        }
    }

    fn register_function(
        &self,
        name: &str,
        namespace: FnNamespace,
        handler: NativeFn,
    ) -> Result<(), String> {
        // Register id
        let id = rand::random::<u64>();
        get_callbacks().lock().unwrap().insert(id, handler);

        // Send request
        let req = PluginRequest::RegisterFn {
            id: self.get_id().to_string(),
            name: name.to_string(),
            namespace,
            callback_id: id,
        };

        let bytes = bincode::serialize(&req).map_err(|e| e.to_string())?;
        unsafe {
            ffi_gateway(bytes.as_ptr(), bytes.len());
        }

        Ok(())
    }
}