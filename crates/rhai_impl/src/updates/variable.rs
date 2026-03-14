use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc, sync::RwLock};
use tokio::sync::watch;

pub static GLOBAL_VAR_STORE: Lazy<Arc<RwLock<HashMap<String, String>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub static VAR_WATCHERS: Lazy<Arc<RwLock<HashMap<String, watch::Sender<String>>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub struct VarWatcherAPI;

impl VarWatcherAPI {
    /// Register new variable
    pub fn register(var_name: &str, initial_value: String) {
        let (tx, _) = watch::channel(initial_value.clone());
        VAR_WATCHERS.write().unwrap().insert(var_name.to_owned(), tx);
        GLOBAL_VAR_STORE.write().unwrap().insert(var_name.to_owned(), initial_value);
    }

    /// Subscribe to a variable
    pub fn subscribe(var_name: &str) -> Option<watch::Receiver<String>> {
        VAR_WATCHERS.read().unwrap().get(var_name).map(|tx| tx.subscribe())
    }

    /// Update a variable in the store
    pub fn update_store(var_name: &str, val: String) {
        GLOBAL_VAR_STORE.write().unwrap().insert(var_name.to_owned(), val);
    }

    /// Update the store and broadcast
    pub fn update_with_broadcast(var_name: &str, val: String) {
        GLOBAL_VAR_STORE.write().unwrap().insert(var_name.to_owned(), val.clone());
        Self::broadcast_value(var_name, &val);
    }

    /// Broadcast a variable
    pub fn broadcast(var_name: &str) {
        let value = {
            let store = GLOBAL_VAR_STORE.read().unwrap();
            store.get(var_name).cloned()
        };

        if let Some(value) = value {
            let watchers = VAR_WATCHERS.read().unwrap();
            if let Some(tx) = watchers.get(var_name) {
                let _ = tx.send(value);
            }
        }
    }

    /// Broadcast a variable with provided value
    pub fn broadcast_value(var_name: &str, value: &str) {
        let watchers = VAR_WATCHERS.read().unwrap();
        if let Some(tx) = watchers.get(var_name) {
            let _ = tx.send(value.to_owned());
        }
    }

    /// Get a snapshot of all current variable state
    pub fn state() -> HashMap<String, String> {
        GLOBAL_VAR_STORE.read().unwrap().clone()
    }
}
