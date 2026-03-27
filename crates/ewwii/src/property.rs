#[macro_export]
macro_rules! apply_property {
    ($prop:expr, |$v:ident: $t:ty| $body:expr) => {{
        let setter = move |$v: $t| $body;
        match $prop {
            PropValue::Static(val) => {
                setter(val);
            }
            PropValue::Bound { var_name, initial, parser, additional: _ } => {
                setter(initial);
                if let Some(mut receiver) =
                    ewwii_rhai_impl::updates::variable::VarWatcherAPI::subscribe(&var_name)
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
    }};
}

#[macro_export]
macro_rules! apply_property_watch {
    ($prop:expr, [$($clone:ident),*], |$v:ident: $t:ty| $body:expr) => {{
        if let PropValue::Bound { var_name, initial: _, parser, additional: _ } = $prop {
            $(let $clone = $clone.clone();)*
            if let Some(mut receiver) = ewwii_rhai_impl::updates::variable::VarWatcherAPI::subscribe(&var_name) {
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
    }};

    ($prop:expr, |$v:ident: $t:ty| $body:expr) => {{
        if let PropValue::Bound { var_name, initial: _, parser } = $prop {
            if let Some(mut receiver) = ewwii_rhai_impl::updates::variable::VarWatcherAPI::subscribe(&var_name) {
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
