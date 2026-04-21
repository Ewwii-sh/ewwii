use crate::config::ewwii_config::{ConfigEngine, EWWII_CONFIG_PARSER};
use crate::updates::api::VarWatcherAPI;
use crate::updates::SHUTDOWN_REGISTRY;
use ewwii_shared_utils::template::TemplateExpr;
use ewwii_shared_utils::variables::{GlobalCompare, GlobalVar};
use gtk4::glib;
use tokio::sync::watch;

pub fn handle_global_compare(compare: GlobalCompare) -> watch::Receiver<String> {
    let mut global_vars: Vec<(usize, GlobalVar)> = Vec::new();
    let mut idx = 0;

    for var in &compare.vars {
        if let Some(val) = var.as_global_var() {
            global_vars.push((idx, val.clone()));
        }
        idx += 1
    }

    let (tx, _) = watch::channel(String::new());

    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let _ = notify_tx.send(()); // Init

    // Shutdown registry
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    SHUTDOWN_REGISTRY.lock().unwrap().push(shutdown_tx);

    // Handle GlobalVar's
    for (_, var) in global_vars.clone() {
        let notify_tx = notify_tx.clone();
        let mut shutdown_rx = shutdown_rx.clone();

        tokio::spawn(async move {
            let mut rx = match VarWatcherAPI::subscribe(&var.name) {
                Some(rx) => rx,
                None => {
                    log::error!("Failed to subscribe to global var: {}", var.name);
                    return;
                }
            };

            loop {
                tokio::select! {
                    result = rx.changed() => {
                        if result.is_err() {
                            log::debug!("Watcher closed for global var: {}", var.name);
                            break;
                        }
                        let _ = notify_tx.send(());
                    }
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            log::trace!("Shutdown received, stopping watcher for: {}", var.name);
                            break;
                        }
                    }
                }
            }
        });
    }

    // aggregator task
    let tx_clone = tx.clone();
    let compare_closure = compare.closure.clone();
    let compare_array = compare.vars.clone();
    let global_vars = global_vars.clone();
    let mut shutdown_rx = shutdown_rx.clone();

    glib::MainContext::default().spawn_local(async move {
        loop {
            tokio::select! {
                msg = notify_rx.recv() => {
                    if msg.is_none() {
                        break;
                    }

                    let maybe_result: Option<String> = EWWII_CONFIG_PARSER.with(|parser_cell| {
                        let parser_ref = parser_cell.borrow();
                        let parser = match parser_ref.as_ref() {
                            Some(p) => p,
                            None => {
                                log::error!("Parser not initialized");
                                return None;
                            }
                        };
                        let mut args = compare_array.clone();
                        let state = VarWatcherAPI::state();
                        for (idx, gvar) in &global_vars {
                            let value = match state.get(&gvar.name) {
                                Some(v) => v,
                                None => {
                                    log::error!("Global var '{}' not found", gvar.name);
                                    return None;
                                }
                            };
                            if let Some(slot) = args.get_mut(*idx) {
                                *slot = value.clone().into();
                            } else {
                                log::error!("Index {} out of bounds", idx);
                            }
                        }
                        let callback_handle = match compare_closure.handle {
                            Some(h) => h,
                            None => {
                                log::error!("Unexpected callback handle received: None");
                                return None;
                            }
                        };
                        let args_dyn: rhai::Array = args
                            .into_iter()
                            .map(|a| a.into_dynamic())
                            .collect();
                        Some(
                            match parser {
                                ConfigEngine::Default(rhai) => {
                                    match rhai.call_callback::<String>(callback_handle, (args_dyn,)) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            log::error!("Closure execution failed: {:?}", e);
                                            return None;
                                        }
                                    }
                                }
                                ConfigEngine::Custom(_) => {
                                    log::error!("Callbacks are only supported with the Rhai config engine");
                                    return None;
                                }
                            }
                        )
                    });

                    if let Some(result) = maybe_result {
                        let _ = tx_clone.send(result);
                    }
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        log::trace!("Shutdown received, stopping aggregator");
                        break;
                    }
                }
            }
        }
    });

    tx.subscribe()
}

pub fn handle_template(template: TemplateExpr) -> watch::Receiver<String> {
    let watched_vars = template.collect_vars();
    let watched_vars: Vec<String> =
        watched_vars.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect();

    let (tx, _) = watch::channel(String::new());
    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let _ = notify_tx.send(()); // init

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    SHUTDOWN_REGISTRY.lock().unwrap().push(shutdown_tx);

    // subscribe to all vars in template
    for var_name in watched_vars.clone() {
        let notify_tx = notify_tx.clone();
        let mut shutdown_rx = shutdown_rx.clone();

        tokio::spawn(async move {
            let mut rx = match VarWatcherAPI::subscribe(&var_name) {
                Some(rx) => rx,
                None => {
                    log::error!("Failed to subscribe to var: {}", var_name);
                    return;
                }
            };

            loop {
                tokio::select! {
                    result = rx.changed() => {
                        if result.is_err() { break; }
                        let _ = notify_tx.send(());
                    }
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() { break; }
                    }
                }
            }
        });
    }

    // aggregator
    let tx_clone = tx.clone();
    let mut shutdown_rx = shutdown_rx.clone();

    glib::MainContext::default().spawn_local(async move {
        loop {
            tokio::select! {
                msg = notify_rx.recv() => {
                    if msg.is_none() { break; }

                    let state = VarWatcherAPI::state();
                    let var_map: std::collections::HashMap<String, String> = watched_vars
                        .iter()
                        .filter_map(|name| {
                            state.get(name).map(|v| (name.clone(), v.to_string()))
                        })
                        .collect();

                    match template.eval(&var_map) {
                        Ok(result) => { let _ = tx_clone.send(result); }
                        Err(e) => { log::error!("Template eval failed: {}", e); }
                    }
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() { break; }
                }
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
            PropValue::Bound { var_name, initial, parser, template } => {
                let var_value = crate::updates::api::VarWatcherAPI::state_of(&var_name);

                if let Some(tmpl) = template {
                    // quick template handling
                    // (template can only be passed by plugins as of rn)
                    let mut recv = crate::property::handle_template(tmpl);
                    glib::MainContext::default().spawn_local(async move {
                        while recv.changed().await.is_ok() {
                            let raw = recv.borrow().clone();
                            if let Some(v) = parser(&raw) {
                                setter(v);
                            }
                        }
                    });
                } else {
                    // no template (default)
                    if let Some(v) = (!var_value.is_empty()).then(|| parser(&var_value)).flatten() {
                        setter(v);
                    } else {
                        setter(initial);
                    }

                    if let Some(mut receiver) =
                        crate::updates::api::VarWatcherAPI::subscribe(&var_name)
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
            PropValue::Bound { var_name, initial, parser, template } => {
                let var_value = crate::updates::api::VarWatcherAPI::state_of(&var_name);

                if let Some(tmpl) = template {
                    let mut recv = crate::property::handle_template(tmpl);
                    glib::MainContext::default().spawn_local(async move {
                        while recv.changed().await.is_ok() {
                            let raw = recv.borrow().clone();
                            if let Some($v) = parser(&raw) {
                                $body
                            }
                        }
                    });
                } else {
                    if let Some($v) = (!var_value.is_empty()).then(|| parser(&var_value)).flatten() {
                        $body
                    } else {
                        let $v = initial;
                        $body
                    }

                    if let Some(mut receiver) = crate::updates::api::VarWatcherAPI::subscribe(&var_name) {
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
