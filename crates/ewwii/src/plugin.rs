use crate::app::{App, DaemonCommand};
use crate::config::{ConfigEngine, EWWII_CONFIG_PARSER};
use crate::daemon_response;
use crate::display_backend::DisplayBackend;
use crate::opts::WidgetControlAction;
use ewwii_plugin_api::proxy::{CallbackResponse, PluginRequest};
use ewwii_plugin_api::{IpcRequest, NbclType, PluginError, PluginValue, WidgetControlType, LibraryItemFFI};
use ewwii_shared_utils::ast::WidgetNode;
use ewwii_shared_utils::prop::Callback;
use nbcl::Type as ActualNbclType;
use nbcl::Value as NbclValue;
use nbcl::library::Library as NbclLibrary;
use nbcl::library::LibraryItem as NbclLibraryItem;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::RwLock;
use tokio::sync::mpsc;

pub static PLUGIN_TX: OnceLock<mpsc::Sender<PluginRequest>> = OnceLock::new();

pub fn is_compatible(plugin_ver: &str, host_ver: &str) -> bool {
    let p_str = plugin_ver.trim_matches('\0');
    let h_str = host_ver.trim_matches('\0');

    let p = semver::Version::parse(p_str);
    let h = semver::Version::parse(h_str);

    match (p, h) {
        (Ok(pv), Ok(hv)) => pv == hv,
        (Err(pe), Err(he)) => {
            log::error!("Both versions failed to parse: P_Err: {}, H_Err: {}", pe, he);
            false
        }
        _ => false,
    }
}

pub struct ActivePlugin {
    pub library: libloading::Library,
    pub id: String,
    pub version: String,
}

pub static ACTIVE_PLUGINS: Lazy<RwLock<Vec<ActivePlugin>>> = Lazy::new(|| RwLock::new(Vec::new()));

fn nbclvalue_to_plugin_value(any: NbclValue) -> PluginValue {
    let res = match any {
        NbclValue::Null => PluginValue::Null,
        NbclValue::Str(v) => PluginValue::String(v),
        NbclValue::Int(v) => PluginValue::Int(v),
        NbclValue::Float(v) => PluginValue::Float(v),
        NbclValue::Bool(v) => PluginValue::Bool(v),
        NbclValue::List(v) => {
            PluginValue::Array(v.into_iter().map(nbclvalue_to_plugin_value).collect())
        }
        _ => PluginValue::Null,
    };

    return res;
}

fn plugin_value_to_nbcl(val: PluginValue) -> NbclValue {
    match val {
        PluginValue::String(s) => NbclValue::Str(s),
        PluginValue::Int(i) => NbclValue::Int(i),
        PluginValue::Float(f) => NbclValue::Float(f),
        PluginValue::Bool(b) => NbclValue::Bool(b),
        PluginValue::Array(arr) => {
            let vec: Vec<NbclValue> = arr.into_iter().map(plugin_value_to_nbcl).collect();
            NbclValue::List(vec)
        }
        PluginValue::Null => NbclValue::Null,
    }
}

fn plugin_nbcltype_to_nbcltype(ty: NbclType) -> ActualNbclType {
    match ty {
        NbclType::String => ActualNbclType::Str,
        NbclType::Int => ActualNbclType::Int,
        NbclType::Float => ActualNbclType::Float,
        NbclType::Bool => ActualNbclType::Bool,
        NbclType::Array => ActualNbclType::List,
        NbclType::Any => ActualNbclType::Any,
        NbclType::Null => ActualNbclType::Null,
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

fn trigger_plugin_func_call(
    plugin_id: &str,
    callback_id: u64,
    args: Vec<NbclValue>,
) -> PluginValue {
    let arg_bytes =
        bincode::serialize(&args.into_iter().map(nbclvalue_to_plugin_value).collect::<Vec<_>>())
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

#[derive(Clone)]
pub struct CustomConfigEngine {
    id: String,
    extension: String,
    main_file: String,
    callback_id: u64,
    cfg_callback_id: Option<u64>,
}

impl CustomConfigEngine {
    pub fn extension(&self) -> String {
        self.extension.clone()
    }

    pub fn main_file(&self) -> String {
        self.main_file.clone()
    }

    pub fn handle_callback(&self, callback: &Callback) {
        if let Some(cfg_cb) = self.cfg_callback_id {
            let arg_bytes =
                bincode::serialize(&(callback.name.clone(), callback.handle.unwrap_or_default()))
                    .unwrap_or_default();
            if let None = call_plugin_handler(&self.id, cfg_cb, arg_bytes) {
                log::error!("Failed calling callback handler.");
            }
        } else {
            log::error!(
                "Failed to handle callback: plugin did not register config callback handler."
            )
        }
    }

    pub fn parse_source(&self, source: String, config_path: PathBuf) -> Result<WidgetNode, String> {
        let path_str = config_path.to_str().unwrap_or("<unknown>");
        trigger_plugin_config_parse(&self.id, self.callback_id, &source, path_str)
            .map_err(|e| e.to_string())
    }
}

impl<B: DisplayBackend> App<B> {
    pub fn handle_plugin_request(&mut self, request: PluginRequest) {
        match request {
            PluginRequest::Log((id, msg)) => {
                log::info!("[{}] {}", id, msg);
            }
            PluginRequest::Warn((id, msg)) => {
                log::warn!("[{}] {}", id, msg);
            }
            PluginRequest::Error((id, msg)) => {
                log::error!("[{}] {}", id, msg);
            }
            PluginRequest::Ipc((_, req)) => {
                self.handle_plugin_ipc(req);
            }
            PluginRequest::RegisterFn { id, name, types, return_type, callback_id } => {
                if name.trim().is_empty() {
                    log::error!("Function name cannot be empty");
                    return;
                }

                if name.contains(' ') {
                    log::error!("Function names cannot contain spaces");
                    return;
                }

                self.register_function_internal(id, name, types, return_type, callback_id);
            }
            PluginRequest::RegisterLib { id, name, items } => {
                if name.trim().is_empty() {
                    log::error!("Function name cannot be empty");
                    return;
                }

                if name.contains(' ') {
                    log::error!("Function names cannot contain spaces");
                    return;
                }

                self.register_lib_internal(id, name, items);
            }
            PluginRequest::RegisterConfigEngine { id, extension, main_file, callback_id } => {
                if extension.trim().is_empty() || main_file.trim().is_empty() {
                    log::error!("File extension or main file cannot be empty");
                    return;
                }

                if extension.contains(' ') || main_file.contains(' ') {
                    log::error!("File extension or main file cannot contain spaces");
                    return;
                }

                let custom_engine = CustomConfigEngine {
                    id,
                    extension,
                    main_file,
                    callback_id,
                    cfg_callback_id: None,
                };

                EWWII_CONFIG_PARSER.with(|p| {
                    *p.borrow_mut() = Some(ConfigEngine::Custom(custom_engine));
                });
            }
            PluginRequest::InjectCss(css) => {
                self.custom_css_provider.load_from_string(&css);
            }
            PluginRequest::ConfigCallbackHandle(id) => {
                EWWII_CONFIG_PARSER.with(|p| {
                    let mut parser = p.borrow_mut();
                    if let Some(ConfigEngine::Custom(ref mut c)) = *parser {
                        c.cfg_callback_id = Some(id);
                    }
                });
            }
        }
    }

    pub fn register_function_internal(
        &self,
        plugin_id: String,
        name: String,
        types: Vec<NbclType>,
        return_type: NbclType,
        callback_id: u64,
    ) {
        EWWII_CONFIG_PARSER.with(|p| {
            let mut parser = p.borrow_mut();
            let types = types.into_iter().map(plugin_nbcltype_to_nbcltype).collect();
            let return_type = plugin_nbcltype_to_nbcltype(return_type);

            match parser.as_mut().unwrap() {
                ConfigEngine::Default(nbcl) => {
                    nbcl.engine.register_native_fn(&name, types, return_type, move |args| {
                        let result = trigger_plugin_func_call(&plugin_id, callback_id, args);

                        Ok(plugin_value_to_nbcl(result))
                    });
                }
                ConfigEngine::Custom(_) => {
                    log::error!(
                        "Registering nbcl functions is only supported with the Nbcl config engine"
                    );
                    return;
                }
            }
        })
    }

    pub fn register_lib_internal(
        &self,
        plugin_id: String,
        name: String,
        items: Vec<LibraryItemFFI>,
    ) {
        EWWII_CONFIG_PARSER.with(|p| {
            let mut parser = p.borrow_mut();

            match parser.as_mut().unwrap() {
                ConfigEngine::Default(nbcl) => {
                    let mut lib_items = Vec::new();
                    for item in items {
                        let mut lib_item   = NbclLibraryItem::define(item.name);

                        for (name, func) in item.functions {
                            let ret    = plugin_nbcltype_to_nbcltype(func.ret);
                            let params = func.params.into_iter().map(plugin_nbcltype_to_nbcltype).collect();

                            let callback_id = func.callback_id.clone();
                            let plugin_id = plugin_id.clone();
                            lib_item = lib_item.with_fn(&name, params, ret, move |args| {
                                let result = trigger_plugin_func_call(&plugin_id, callback_id, args);

                                Ok(plugin_value_to_nbcl(result))
                            });
                        }

                        lib_items.push(lib_item);
                    }

                    nbcl.engine.register_library(NbclLibrary::new(name, lib_items));
                }
                ConfigEngine::Custom(_) => {
                    log::error!(
                        "Registering rhai functions is only supported with the Nbcl config engine"
                    );
                    return;
                }
            }
        })
    }

    pub fn handle_plugin_ipc(&mut self, req: IpcRequest) {
        let handle = tokio::runtime::Handle::current();
        match req {
            IpcRequest::WidgetControl(wc_type) => match wc_type {
                WidgetControlType::Remove(w) => {
                    let (sender, _recv) = daemon_response::create_pair();
                    let command = DaemonCommand::WidgetControl {
                        action: WidgetControlAction::Remove { names: vec![w] },
                        sender,
                    };
                    handle.block_on(async {
                        self.handle_command(command).await;
                    });
                }
                WidgetControlType::Create { parent, codes } => {
                    let (sender, _recv) = daemon_response::create_pair();
                    let command = DaemonCommand::WidgetControl {
                        action: WidgetControlAction::Create {
                            nbcl_codes: codes,
                            parent_name: parent,
                        },
                        sender,
                    };
                    handle.block_on(async {
                        self.handle_command(command).await;
                    });
                }
                WidgetControlType::PropertyGet { prop, widget } => {
                    let (sender, _recv) = daemon_response::create_pair();
                    let command = DaemonCommand::WidgetControl {
                        action: WidgetControlAction::PropertyGet {
                            property: prop,
                            widget_name: widget,
                        },
                        sender,
                    };
                    handle.block_on(async {
                        self.handle_command(command).await;
                    });
                }
                WidgetControlType::PropertyUpdate { prop, widget, value } => {
                    let p2v = HashMap::from([(prop, value)]);

                    let (sender, _recv) = daemon_response::create_pair();
                    let command = DaemonCommand::WidgetControl {
                        action: WidgetControlAction::PropertyUpdate {
                            property_and_value: p2v,
                            widget_name: widget,
                        },
                        sender,
                    };
                    handle.block_on(async {
                        self.handle_command(command).await;
                    });
                }
                WidgetControlType::AddClass { class, widget } => {
                    let (sender, _recv) = daemon_response::create_pair();
                    let command = DaemonCommand::WidgetControl {
                        action: WidgetControlAction::AddClass { class, widget_name: widget },
                        sender,
                    };
                    handle.block_on(async {
                        self.handle_command(command).await;
                    });
                }
                WidgetControlType::RemoveClass { class, widget } => {
                    let (sender, _recv) = daemon_response::create_pair();
                    let command = DaemonCommand::WidgetControl {
                        action: WidgetControlAction::RemoveClass { class, widget_name: widget },
                        sender,
                    };
                    handle.block_on(async {
                        self.handle_command(command).await;
                    });
                }
            },
            IpcRequest::Update(var, val) => {
                let (sender, _recv) = daemon_response::create_pair();
                let command =
                    DaemonCommand::Update { mappings: HashMap::from([(var, val)]), sender };
                handle.block_on(async {
                    self.handle_command(command).await;
                });
            }
            IpcRequest::Close(windows) => {
                let (sender, _recv) = daemon_response::create_pair();
                let command = DaemonCommand::CloseWindows { windows, auto_reopen: false, sender };
                handle.block_on(async {
                    self.handle_command(command).await;
                });
            }
            IpcRequest::Open(window, toggle) => {
                let (sender, _recv) = daemon_response::create_pair();
                let command = DaemonCommand::OpenWindow {
                    window_name: window,
                    instance_id: None,
                    pos: None,
                    size: None,
                    anchor: None,
                    screen: None,
                    should_toggle: toggle,
                    duration: None,
                    sender,
                };
                handle.block_on(async {
                    self.handle_command(command).await;
                });
            }
            IpcRequest::CloseAll => {
                let command = DaemonCommand::CloseAll;
                handle.block_on(async {
                    self.handle_command(command).await;
                });
            }
            IpcRequest::Reload => {
                let (sender, _recv) = daemon_response::create_pair();
                let command = DaemonCommand::ReloadConfigAndCss(sender);
                handle.block_on(async {
                    self.handle_command(command).await;
                });
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_gateway(ptr: *const u8, len: usize) {
    // SAFETY: Convert the raw pointer/len into a Rust slice
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };

    let request: PluginRequest = match bincode::deserialize(bytes) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("[Host] Failed to deserialize plugin request: {}", e);
            return;
        }
    };

    PLUGIN_TX.get().unwrap().blocking_send(request).unwrap();
}
