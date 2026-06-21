use crate::updates::api::VarWatcherAPI;
use crate::updates::SHUTDOWN_REGISTRY;
use ewwii_shared_utils::template::TemplateExpr;
use gtk4::glib;
use tokio::sync::watch;

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
                let var_value = $crate::updates::api::VarWatcherAPI::state_of(&var_name);

                if let Some(tmpl) = template {
                    // quick template handling
                    // (template can only be passed by plugins as of rn)
                    let mut recv = $crate::property::handle_template(tmpl);
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
                        $crate::updates::api::VarWatcherAPI::subscribe(&var_name)
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
        }
    }};
}

#[macro_export]
macro_rules! apply_property_watch {
    ($prop:expr, [$($clone:ident),*], |$v:ident: $t:ty| $body:expr) => {{
        $(let $clone = $clone.clone();)*

        match $prop {
            PropValue::Bound { var_name, initial, parser, template } => {
                let var_value = $crate::updates::api::VarWatcherAPI::state_of(&var_name);

                if let Some(tmpl) = template {
                    let mut recv = $crate::property::handle_template(tmpl);
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

                    if let Some(mut receiver) = $crate::updates::api::VarWatcherAPI::subscribe(&var_name) {
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
