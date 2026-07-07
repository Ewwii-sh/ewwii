use crate::updates::api::VarWatcherAPI;
use crate::updates::SHUTDOWN_REGISTRY;
use ewwii_shared_utils::prop::Callback;
use ewwii_shared_utils::template::TemplateExpr;
use gtk4::glib;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::watch;

thread_local! {
    static ACTIVE_TASKS: RefCell<Vec<glib::JoinHandle<()>>> = const { RefCell::new(Vec::new()) };
}

pub fn close_all_property_tasks() {
    ACTIVE_TASKS.with(|tasks| {
        let mut tasks_mut = tasks.borrow_mut();
        for handle in tasks_mut.drain(..) {
            handle.abort();
        }
    });
}

pub fn register_task(handle: glib::JoinHandle<()>) {
    ACTIVE_TASKS.with(|tasks| {
        tasks.borrow_mut().push(handle);
    });
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

pub fn mutate_raw(mutation: Option<Callback>, raw: String) -> String {
    if let Some(mut m) = mutation {
        let ret = Rc::new(RefCell::new(String::new()));
        let data = Rc::new(vec![raw]);

        m.ret = Some(ret.clone());
        m.data = Some(data);

        crate::config::ewwii_config::EWWII_CONFIG_PARSER.with(|p| {
            if let Some(parser_instance) = p.borrow().as_ref() {
                parser_instance.handle_callback(&m);
            }
        });

        let guard = ret.borrow();
        guard.clone()
    } else {
        raw
    }
}

#[macro_export]
macro_rules! apply_property {
    ($prop:expr, |$v:ident: $t:ty| $body:expr) => {{
        let setter = move |$v: $t| $body;
        match $prop {
            PropValue::Static(val) => {
                setter(val);
            }
            PropValue::Bound { var_name, initial, parser, template, mutation } => {
                let var_value = $crate::updates::api::VarWatcherAPI::state_of(&var_name);

                if let Some(tmpl) = template {
                    // quick template handling
                    // (template can only be passed by plugins as of rn)
                    let mut recv = $crate::property_macro::handle_template(tmpl);
                    let handle = glib::MainContext::default().spawn_local(async move {
                        while recv.changed().await.is_ok() {
                            let raw = recv.borrow().clone();
                            if let Some(v) = parser(&raw) {
                                setter(v);
                            }
                        }
                    });
                    $crate::property_macro::register_task(handle);
                } else {
                    // no template (default)
                    let resolved_init =
                        $crate::property_macro::mutate_raw(mutation.clone(), var_value.clone());
                    if let Some(v) =
                        (!var_value.is_empty()).then(|| parser(&resolved_init)).flatten()
                    {
                        setter(v);
                    } else {
                        setter(initial);
                    }

                    if let Some(mut receiver) =
                        $crate::updates::api::VarWatcherAPI::subscribe(&var_name)
                    {
                        let mutation_clone = mutation.clone();
                        let handle = glib::MainContext::default().spawn_local(async move {
                            while receiver.changed().await.is_ok() {
                                let raw = receiver.borrow().clone();
                                let resolved_raw =
                                    $crate::property_macro::mutate_raw(mutation_clone.clone(), raw);
                                if let Some(v) = parser(&resolved_raw) {
                                    setter(v);
                                }
                            }
                        });

                        $crate::property_macro::register_task(handle);
                    }
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! apply_property_watch {
    ($prop:expr, [$($clone:ident),*], |$v:ident: $t:ty| $body:expr) => {{
        $(let $clone = $clone.clone();)*

        match $prop {
            PropValue::Bound { var_name, initial, parser, template, mutation } => {
                let var_value = $crate::updates::api::VarWatcherAPI::state_of(&var_name);

                if let Some(tmpl) = template {
                    let mut recv = $crate::property_macro::handle_template(tmpl);
                    let handle = glib::MainContext::default().spawn_local(async move {
                        while recv.changed().await.is_ok() {
                            let raw = recv.borrow().clone();
                            if let Some(mut m) = mutation.clone() {
                                let ret = Rc::new(RefCell::new(String::new()));
                                let data = Rc::new(vec![raw]);

                                m.ret = Some(ret.clone());
                                m.data = Some(data);

                                $crate::config::ewwii_config::EWWII_CONFIG_PARSER.with(|p| {
                                    let parser_raw = p.borrow();
                                    let parser = parser_raw.as_ref().unwrap();

                                    parser.handle_callback(&m);
                                });

                                let new_raw = ret.borrow();
                                if let Some($v) = parser(&new_raw) {
                                    $body
                                }
                            } else {
                                if let Some($v) = parser(&raw) {
                                    $body
                                }
                            }
                        }
                    });
                    $crate::property_macro::register_task(handle);
                } else {
                    let resolved_init = $crate::property_macro::mutate_raw(mutation.clone(), var_value.clone());
                    if let Some($v) = (!var_value.is_empty()).then(|| parser(&resolved_init)).flatten() {
                        $body
                    } else {
                        let $v = initial;
                        $body
                    }

                    if let Some(mut receiver) = $crate::updates::api::VarWatcherAPI::subscribe(&var_name) {
                        let mutation_clone = mutation.clone();
                        let handle = glib::MainContext::default().spawn_local(async move {
                            while receiver.changed().await.is_ok() {
                                let raw = receiver.borrow().clone();
                                let resolved_raw = $crate::property_macro::mutate_raw(mutation_clone.clone(), raw);
                                if let Some($v) = parser(&resolved_raw) {
                                    $body
                                }
                            }
                        });
                        $crate::property_macro::register_task(handle);
                    }
                }
            }
            PropValue::Static(_) => {}
        }
    }};
}

#[macro_export]
macro_rules! bind_property {
    ($prop:expr, $key:expr, $getter:ident, [$($clone:ident),*], |$v:ident: $t:ty| $body:expr) => {
        if let Ok(prop) = $getter($prop, $key) {
            $(let $clone = $clone.clone();)*
            apply_property!(prop, |$v: $t| $body);
        }
    };
    ($prop:expr, $key:expr, $getter:ident, |$v:ident: $t:ty| $body:expr) => {
        if let Ok(prop) = $getter($prop, $key) {
            apply_property!(prop, |$v: $t| $body);
        }
    };
}
