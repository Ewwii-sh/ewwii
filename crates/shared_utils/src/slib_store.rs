use once_cell::sync::Lazy;
use rhai::{Array, Dynamic};
use std::collections::HashMap;
use std::sync::Mutex;

static FUNC_REGISTRY: Lazy<Mutex<HashMap<String, Box<dyn Fn(Array) -> Dynamic + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn register_functions(
    name: String,
    func: Box<dyn Fn(Array) -> Dynamic + Send + Sync>,
) -> Result<(), String> {
    let mut registry = FUNC_REGISTRY.lock().map_err(|e| e.to_string())?;
    registry.insert(name, func);
    Ok(())
}

pub fn call_registered(name: &str, args: Array) -> Result<Option<Dynamic>, String> {
    let registry = FUNC_REGISTRY.lock().map_err(|e| e.to_string())?;

    if let Some(func) = registry.get(name) {
        Ok(Some(func(args)))
    } else {
        Ok(None)
    }
}
