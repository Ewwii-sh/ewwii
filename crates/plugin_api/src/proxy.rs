//! This module provides plugin requests and host proxy
//! that are used to redirect API calls to host after serialization

use crate::{
    ConfigCallbackFn, ConfigInfo, EwwiiAPI, FutureResult, IpcRequest, LibraryFnFFI, LibraryItem,
    LibraryItemFFI, ListenHandleFn, NativeFn, NbclType, ParseFn, PluginError, PluginValue,
    RuntimePaths, SignalUpdateFn, EmitInfo,
};
use ewwii_shared_utils::ast::WidgetNode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{Mutex, OnceLock};

pub type ManualHandle<T> = Arc<dyn Fn(T) + Send + Sync>;

pub trait ManualHandleExt<T> {
    fn new<F>(f: F) -> Self
    where
        F: Fn(T) + Send + Sync + 'static;
}

impl<T> ManualHandleExt<T> for ManualHandle<T> {
    fn new<F>(f: F) -> Self
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        Arc::new(f)
    }
}

/// Represents the different types of callbacks that can be registered by a plugin.
pub enum CallbackHandler {
    NativeFn(NativeFn),
    ParseFn(ParseFn),
    ListenHandleFn(ListenHandleFn),
    SignalUpdateFn(SignalUpdateFn),
    ConfigCallbackFn(ConfigCallbackFn),

    ManualHandleStr(ManualHandle<String>),
    ManualHandleU64(ManualHandle<u64>),
    ManualHandleRtPaths(ManualHandle<RuntimePaths>),
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
#[derive(Debug, Serialize, Deserialize)]
pub enum PluginRequest {
    // (id, msg)
    Log(String, String),
    Warn(String, String),
    Error(String, String),

    // IPC API (Data transfer)
    Ipc(String, IpcRequest, u64),

    // Registration API (Complex)
    RegisterFn {
        id: String,
        name: String,
        types: Vec<NbclType>,
        return_type: NbclType,
        callback_id: u64,
    },
    RegisterLib {
        id: String,
        name: String,
        items: Vec<LibraryItemFFI>,
    },
    RegisterConfigEngine {
        id: String,
        extension: String,
        main_file: String,
        callback_id: u64,
    },

    // Dynamic Runtime
    InjectCss(String, String, u64),
    RemoveCss(u64),
    InjectNbclBootstrap(String),
    Emit(String, String),
    Listen(String, String, u64),
    RegisterSignal(String, String),
    UpdateSignal(String, String),
    OnSignalUpdate(String, String, u64),
    SignalValue(String, String, u64),
    GetRuntimePaths(String, u64),

    // Handlers
    ConfigCallbackHandle(u64),
}

// This is provided on the host side
extern "C" {
    fn ffi_gateway(ptr: *const u8, len: usize);
}

/// # SAFETY
///
/// Unsafe is necessary across FFI
#[no_mangle]
pub unsafe extern "C" fn plugin_callback_handler(
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
            let (source, path): (String, String) = bincode::deserialize(bytes).unwrap_or_default();
            match f(&source, &path) {
                Ok(node) => {
                    bincode::serialize(&CallbackResponse::WidgetNode(node)).unwrap_or_default()
                }
                Err(e) => bincode::serialize(&CallbackResponse::Error(PluginError::ParseError(e)))
                    .unwrap_or_default(),
            }
        }
        Some(CallbackHandler::ListenHandleFn(f)) => {
            let value: EmitInfo = bincode::deserialize(bytes).unwrap_or_default();
            f(value);
            return std::ptr::null_mut();
        }
        Some(CallbackHandler::SignalUpdateFn(f)) => {
            let value: String = bincode::deserialize(bytes).unwrap_or_default();
            f(&value);
            return std::ptr::null_mut();
        }
        Some(CallbackHandler::ConfigCallbackFn(f)) => {
            let (name, id): (String, String) = bincode::deserialize(bytes).unwrap_or_default();
            f(&name, &id);
            return std::ptr::null_mut();
        }
        Some(CallbackHandler::ManualHandleStr(f)) => {
            let value: String = bincode::deserialize(bytes).unwrap_or_default();
            f(value);
            return std::ptr::null_mut();
        }
        Some(CallbackHandler::ManualHandleU64(f)) => {
            let value: u64 = bincode::deserialize(bytes).unwrap_or_default();
            f(value);
            return std::ptr::null_mut();
        }
        Some(CallbackHandler::ManualHandleRtPaths(f)) => {
            let value: RuntimePaths = bincode::deserialize(bytes).unwrap_or_default();
            f(value);
            return std::ptr::null_mut();
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

/// # SAFETY
///
/// Unsafe is necessary across FFI
#[no_mangle]
pub unsafe extern "C" fn plugin_free_buffer(ptr: *mut u8, len: usize) {
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

    fn call_host(&self, req: PluginRequest) {
        let bytes = match bincode::serialize(&req) {
            Ok(o) => o,
            Err(e) => {
                println!("Failed to serialize request: {}", e);
                return;
            }
        };

        unsafe {
            ffi_gateway(bytes.as_ptr(), bytes.len());
        }
    }
}

impl EwwiiAPI for HostProxy {
    fn metadata_id(&self) -> String {
        self.get_id().to_string()
    }

    fn log(&self, msg: &str) {
        let plugid = &self.get_id();
        let req = PluginRequest::Log(plugid.to_string(), msg.to_string());

        self.call_host(req);
    }

    fn warn(&self, msg: &str) {
        let plugid = &self.get_id();
        let req = PluginRequest::Warn(plugid.to_string(), msg.to_string());

        self.call_host(req);
    }

    fn error(&self, msg: &str) {
        let plugid = &self.get_id();
        let req = PluginRequest::Error(plugid.to_string(), msg.to_string());

        self.call_host(req);
    }

    fn ipc_request(&self, req: IpcRequest) -> FutureResult<String> {
        let (tx, rx) = std::sync::mpsc::channel();
        let handle = ManualHandle::new(move |value: String| {
            let _ = tx.send(value);
        });

        let id = rand::random::<u64>();
        get_callbacks().lock().unwrap().insert(id, CallbackHandler::ManualHandleStr(handle));

        let req = PluginRequest::Ipc(self.get_id().to_string(), req, id);
        self.call_host(req);

        FutureResult { channel: rx }
    }

    // === Registration === //

    fn register_function(
        &self,
        name: &str,
        types: Vec<NbclType>,
        return_type: NbclType,
        handler: NativeFn,
    ) {
        // Register id
        let id = rand::random::<u64>();
        get_callbacks().lock().unwrap().insert(id, CallbackHandler::NativeFn(handler));

        // Send request
        let req = PluginRequest::RegisterFn {
            id: self.get_id().to_string(),
            name: name.to_string(),
            types,
            return_type,
            callback_id: id,
        };

        self.call_host(req)
    }

    fn register_library(&self, name: &str, items: Vec<LibraryItem>) {
        let mut ffi_items = Vec::new();
        for item in items {
            let mut functions = HashMap::new();

            for (name, func) in item.functions {
                let id = rand::random::<u64>();
                get_callbacks().lock().unwrap().insert(id, CallbackHandler::NativeFn(func.handler));

                let fn_ffi = LibraryFnFFI { params: func.params, ret: func.ret, callback_id: id };
                functions.insert(name, fn_ffi);
            }

            let ffi_item = LibraryItemFFI { name: item.name, functions };
            ffi_items.push(ffi_item);
        }

        let req = PluginRequest::RegisterLib {
            id: self.get_id().to_string(),
            name: name.to_string(),
            items: ffi_items,
        };

        self.call_host(req)
    }

    fn register_config_engine(&self, info: ConfigInfo, parser: ParseFn) {
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

    // === Dynamic Runtime === //

    fn inject_css(&self, css: &str) -> FutureResult<u64> {
        let (tx, rx) = std::sync::mpsc::channel();
        let handle = ManualHandle::new(move |value: u64| {
            let _ = tx.send(value);
        });

        let id = rand::random::<u64>();
        get_callbacks().lock().unwrap().insert(id, CallbackHandler::ManualHandleU64(handle));

        let req = PluginRequest::InjectCss(css.to_string(), self.get_id().to_string(), id);
        self.call_host(req);

        FutureResult { channel: rx }
    }

    fn remove_css(&self, idx: u64) {
        let req = PluginRequest::RemoveCss(idx);
        self.call_host(req);
    }

    fn inject_nbcl_bootstrap(&self, source: &str) {
        let req = PluginRequest::InjectNbclBootstrap(source.to_string());
        self.call_host(req);
    }

    fn emit(&self, signal: &str, data: String) {
        let req = PluginRequest::Emit(signal.to_string(), data);
        self.call_host(req);
    }

    fn listen(&self, signal: &str, handle: ListenHandleFn) {
        let id = rand::random::<u64>();
        get_callbacks().lock().unwrap().insert(id, CallbackHandler::ListenHandleFn(handle));

        let req = PluginRequest::Listen(self.get_id().to_string(), signal.to_string(), id);
        self.call_host(req);
    }

    fn register_signal(&self, name: &str, initial: String) {
        let req = PluginRequest::RegisterSignal(name.to_string(), initial);
        self.call_host(req);
    }

    fn update_signal(&self, name: &str, value: String) {
        let req = PluginRequest::UpdateSignal(name.to_string(), value);
        self.call_host(req);
    }

    fn on_signal_update(&self, name: &str, handle: SignalUpdateFn) {
        let id = rand::random::<u64>();
        get_callbacks().lock().unwrap().insert(id, CallbackHandler::SignalUpdateFn(handle));

        let req = PluginRequest::OnSignalUpdate(self.get_id().to_string(), name.to_string(), id);
        self.call_host(req);
    }

    fn signal_value(&self, name: &str) -> FutureResult<String> {
        let (tx, rx) = std::sync::mpsc::channel();
        let handle = ManualHandle::new(move |value: String| {
            let _ = tx.send(value);
        });

        let id = rand::random::<u64>();
        get_callbacks().lock().unwrap().insert(id, CallbackHandler::ManualHandleStr(handle));

        let req = PluginRequest::SignalValue(self.get_id().to_string(), name.to_string(), id);
        self.call_host(req);

        FutureResult { channel: rx }
    }

    fn get_runtime_paths(&self) -> FutureResult<RuntimePaths> {
        let (tx, rx) = std::sync::mpsc::channel();
        let handle = ManualHandle::new(move |value: RuntimePaths| {
            let _ = tx.send(value);
        });

        let id = rand::random::<u64>();
        get_callbacks().lock().unwrap().insert(id, CallbackHandler::ManualHandleRtPaths(handle));

        let req = PluginRequest::GetRuntimePaths(self.get_id().to_string(), id);
        self.call_host(req);

        FutureResult { channel: rx }
    }

    // === Handlers === //

    fn handle_config_callbacks(&self, handle: ConfigCallbackFn) {
        let id = rand::random::<u64>();
        get_callbacks().lock().unwrap().insert(id, CallbackHandler::ConfigCallbackFn(handle));

        let req = PluginRequest::ConfigCallbackHandle(id);
        self.call_host(req);
    }
}
