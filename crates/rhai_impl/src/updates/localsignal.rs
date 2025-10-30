use rhai::Map;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use gtk4::subclass::prelude::*;
use super::{get_prefered_shell, handle_poll, handle_listen};
use once_cell::sync::Lazy;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct LocalDataBinder {
        pub value: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LocalDataBinder {
        const NAME: &'static str = "LocalDataBinder";
        type Type = super::LocalDataBinder;
        type ParentType = glib::Object;
    }

	impl ObjectImpl for LocalDataBinder {
	    fn properties() -> &'static [glib::ParamSpec] {
	        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
	            once_cell::sync::Lazy::new(|| {
	                vec![glib::ParamSpecString::builder("value")
	                    .nick("Value")
	                    .blurb("The bound value")
	                    .default_value(None)
	                    .build()]
	            });
	        PROPERTIES.as_ref()
	    }

	    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
	        match pspec.name() {
	            "value" => self.value.borrow().to_value(),
	            _ => unimplemented!(),
	        }
	    }

	    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
	        match pspec.name() {
	            "value" => {
	                let val: Option<String> = value.get().unwrap();
	                self.set_value(&self.obj(), val.unwrap_or_default());
	            }
	            _ => unimplemented!(),
	        }
	    }
	}

    impl LocalDataBinder {
        pub fn set_value(&self, obj: &super::LocalDataBinder, val: String) {
            *self.value.borrow_mut() = val;
            obj.notify("value");
        }
    }
}

glib::wrapper! {
    pub struct LocalDataBinder(ObjectSubclass<imp::LocalDataBinder>);
}

impl LocalDataBinder {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    pub fn value(&self) -> String {
        self.imp().value.borrow().clone()
    }

    pub fn set_value(&self, val: &str) {
        self.set_property("value", val);
    }
}

#[derive(Debug, Clone)]
pub struct LocalSignal {
	pub id: u64,
	pub props: Map,
	pub data: Arc<LocalDataBinder>
}

thread_local! {
    pub static LOCAL_SIGNALS: Lazy<RefCell<HashMap<u64, Rc<LocalSignal>>>> =
        Lazy::new(|| RefCell::new(HashMap::new()));
}

pub fn register_signal(id: u64, signal: Rc<LocalSignal>) {
    LOCAL_SIGNALS.with(|registry| {
        registry.borrow_mut().insert(id, signal.clone());
    });
}

pub fn handle_localsignal_changes() {
	let shell = get_prefered_shell();
	let get_string_fn = shared_utils::extract_props::get_string_prop;
	let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
	let store = Arc::new(RwLock::new(HashMap::new()));

    LOCAL_SIGNALS.with(|registry| {
        let registry_ref = registry.borrow();

        for (id, signal) in registry_ref.iter() {
            let props = &signal.props;

            match get_string_fn(&props, "type", None) {
                Ok(signal_type) => {
                	match signal_type.to_ascii_lowercase().as_str() {
                		"poll" => handle_poll(id.to_string(), &props, shell.clone(), store.clone(), tx.clone()),
                		"listen" => handle_listen(id.to_string(), &props, shell.clone(), store.clone(), tx.clone()),
                		o => log::error!("Invalid type: '{}'", o),
                	}
                }
                Err(_) => {
                    log::error!(
                        "Unable to handle localsignal {}: 'type' property missing or invalid.",
                        id
                    );
                }
            }
        }
    });

	glib::MainContext::default().spawn_local(async move {
	    while let Some(id_str) = rx.recv().await {
	        let value_opt = {
	            let guard = store.read().unwrap();
	            guard.get(&id_str).cloned()
	        };

	        if let Some(value) = value_opt {
	            if let Ok(id) = id_str.parse::<u64>() {
	                LOCAL_SIGNALS.with(|registry| {
	                    let mut registry_ref = registry.borrow_mut();

	                    if let Some(signal) = registry_ref.get_mut(&id) {
	                        signal.data.set_value(&value);
	                    } else {
	                        log::warn!("No LocalSignal found for id {}", id);
	                    }
	                });
	            } else {
	                log::error!("Invalid id_str '{}': cannot parse to u64", id_str);
	            }
	        } else {
	            log::warn!("No value found in store for id '{}'", id_str);
	        }
	    }
	});
}