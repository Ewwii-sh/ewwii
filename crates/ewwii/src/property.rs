use crate::config::ewwii_config::{EWWII_CONFIG_AST, EWWII_CONFIG_PARSER};
use ewwii_rhai_impl::updates::api::VarWatcherAPI;
use shared_utils::variables::{GlobalCompare, GlobalVar};
use tokio::sync::watch;
use gtk4::glib;

pub fn handle_global_compare(compare: GlobalCompare) -> watch::Receiver<String> {
    let mut global_vars: Vec<(usize, GlobalVar)> = Vec::new();
    let mut idx = 0;

    for var in &compare.vars {
        if let Some(val) = shared_utils::prop_utils::try_get_global_var(var) {
            global_vars.push((idx, val));
        }
        idx += 1
    }

    let (tx, _) = watch::channel(String::new());

    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let _ = notify_tx.send(()); // Init

    // Handle GlobalVar's
    for (_, var) in global_vars.clone() {
        let notify_tx = notify_tx.clone();

        tokio::spawn(async move {
            let mut rx = match VarWatcherAPI::subscribe(&var.name) {
                Some(rx) => rx,
                None => {
                    log::error!("Failed to subscribe to global var: {}", var.name);
                    return;
                }
            };

            loop {
                if rx.changed().await.is_err() {
                    log::debug!("Watcher closed for global var: {}", var.name);
                    break;
                }

                let _ = notify_tx.send(());
            }
        });
    }

    // aggregator task
    let tx_clone = tx.clone();
    let compare_closure = compare.closure.clone();
    let compare_array = compare.vars.clone();
    let global_vars = global_vars.clone();

    glib::MainContext::default().spawn_local(async move {
        while notify_rx.recv().await.is_some() {
            let maybe_result: Option<String> = EWWII_CONFIG_AST.with(|ast_cell| {
                EWWII_CONFIG_PARSER.with(|parser_cell| {
                    let ast_ref = ast_cell.borrow();
                    let parser_ref = parser_cell.borrow();

                    let ast = match ast_ref.as_ref() {
                        Some(a) => a,
                        None => {
                            log::error!("AST not initialized");
                            return None;
                        }
                    };

                    let parser = match parser_ref.as_ref() {
                        Some(p) => p,
                        None => {
                            log::error!("Parser not initialized");
                            return None;
                        }
                    };

                    let engine = &parser.engine;

                    // Building args
                    let mut args = compare_array.clone();
                    let state = VarWatcherAPI::state();

                    for (idx, gvar) in &global_vars {
                        let value = match state.get(&gvar.name) {
                            Some(v) => v,
                            None => {
                                log::error!("Global var '{}' not found", gvar.name);
                                continue;
                            }
                        };

                        if let Some(slot) = args.get_mut(*idx) {
                            *slot = rhai::Dynamic::from(value.clone());
                        } else {
                            log::error!("Index {} out of bounds", idx);
                        }
                    }

                    Some(
                        match compare_closure.call(engine, ast, (args,)) {
                            Ok(v) => v,
                            Err(e) => {
                                log::error!("Closure execution failed: {:?}", e);
                                return None;
                            }
                        }
                    )
                })
            });

            if let Some(result) = maybe_result {
                let _ = tx_clone.send(result);
            }
        }
    });

    tx.subscribe()
}

#[macro_export]
macro_rules! apply_property {
    ($prop:expr, |$v:ident: $t:ty| $body:expr) => {{
        let setter = move |$v: $t| $body;
        match $prop {
            PropValue::Static(val) => {
                setter(val);
            }
            PropValue::Bound { var_name, initial, parser } => {
                let var_value = ewwii_rhai_impl::updates::api::VarWatcherAPI::state_of(&var_name);

                // Set initial only if variable value is empty
                if let Some(v) = (!var_value.is_empty()).then(|| parser(&var_value)).flatten() {
                    setter(v);
                } else {
                    setter(initial);
                }

                if let Some(mut receiver) =
                    ewwii_rhai_impl::updates::api::VarWatcherAPI::subscribe(&var_name)
                {
                    glib::MainContext::default().spawn_local(async move {
                        while receiver.changed().await.is_ok() {
                            let raw = receiver.borrow().clone();
                            if let Some(v) = parser(&raw) {
                                setter(v);
                            }
                        }
                    });
                }
            }
            PropValue::Compare { comp, parser } => {
                let mut recv = crate::property::handle_global_compare(comp);
                glib::MainContext::default().spawn_local(async move {
                    while recv.changed().await.is_ok() {
                        let raw = recv.borrow().clone();
                        if let Some(v) = parser(&raw) {
                            setter(v);
                        }
                    }
                });
            }
        }
    }};
}

#[macro_export]
macro_rules! apply_property_watch {
    ($prop:expr, [$($clone:ident),*], |$v:ident: $t:ty| $body:expr) => {{
        $(let $clone = $clone.clone();)*

        match $prop {
            PropValue::Bound { var_name, initial, parser } => {
                let var_value = ewwii_rhai_impl::updates::api::VarWatcherAPI::state_of(&var_name);

                // Set initial only if variable value is empty
                if let Some($v) = (!var_value.is_empty()).then(|| parser(&var_value)).flatten() {
                    $body
                } else {
                    let $v = initial;
                    $body
                }

                if let Some(mut receiver) = ewwii_rhai_impl::updates::api::VarWatcherAPI::subscribe(&var_name) {
                    gtk4::glib::MainContext::default().spawn_local(async move {
                        while receiver.changed().await.is_ok() {
                            let raw = receiver.borrow().clone();
                            if let Some($v) = parser(&raw) {
                                $body
                            }
                        }
                    });
                }
            }
            PropValue::Compare { comp, parser } => {
                let mut recv = crate::property::handle_global_compare(comp);
                glib::MainContext::default().spawn_local(async move {
                    while recv.changed().await.is_ok() {
                        let raw = recv.borrow().clone();
                        if let Some($v) = parser(&raw) {
                            $body
                        }
                    }
                });
            }
            PropValue::Static(_) => {}
        }
    }};
}

#[macro_export]
macro_rules! bind_property {
    ($props:expr, $key:expr, $getter:ident, $default:expr, [$($clone:ident),*], |$v:ident: $t:ty| $body:expr) => {
        if let Ok(prop) = $getter($props, $key, $default) {
            $(let $clone = $clone.clone();)*
            apply_property!(prop, |$v: $t| $body);
        }
    };
    ($props:expr, $key:expr, $getter:ident, $default:expr, |$v:ident: $t:ty| $body:expr) => {
        if let Ok(prop) = $getter($props, $key, $default) {
            apply_property!(prop, |$v: $t| $body);
        }
    };
}
