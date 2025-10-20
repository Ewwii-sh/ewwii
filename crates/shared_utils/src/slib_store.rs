use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use rhai::{Array, Dynamic};

static FUNC_REGISTRY: Lazy<Mutex<HashMap<String, Box<dyn FnOnce(Array) -> Dynamic + Send>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn register_functions(
    name: String,
    func: Box<dyn FnOnce(Array) -> Dynamic + Send>
) -> Result<(), String> {
    let mut registry = FUNC_REGISTRY
        .lock()
        .map_err(|e| e.to_string())?; // Propagate the error
    
    registry.insert(name, func);
    Ok(())
}



pub fn call_registered(name: &str, args: Array) -> Option<Dynamic> {
    let mut registry = FUNC_REGISTRY.lock().unwrap();
    if let Some(func) = registry.remove(name) {
        Some(func(args))
    } else {
        None
    }
}
