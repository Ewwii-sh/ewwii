use rhai::plugin::*;
use rhai::Dynamic;
use rhai::EvalAltResult;
use serde_json::Value;

#[export_module]
pub mod json {
    // parse a JSON string into a Dynamic representing serde_json::Value
    #[rhai_fn(return_raw)]
    pub fn parse_json(json_str: &str) -> Result<Dynamic, Box<EvalAltResult>> {
        serde_json::from_str::<Value>(json_str).map(Dynamic::from).map_err(|e| format!("Failed to parse JSON: {e}").into())
    }

    // Turn a dyn JSON val back to a string
    #[rhai_fn(return_raw)]
    pub fn to_string(json: Dynamic) -> Result<String, Box<EvalAltResult>> {
        let value: Value = json.try_cast::<Value>().ok_or("Expected a JSON value")?;
        serde_json::to_string(&value).map_err(|e| format!("Failed to serialize JSON: {e}").into())
    }

    // get a key in a JSON object
    #[rhai_fn(return_raw)]
    pub fn get(json: Dynamic, key: &str) -> Result<Dynamic, Box<EvalAltResult>> {
        let value: Value = json.try_cast::<Value>().ok_or("Expected a JSON object")?;
        match value.get(key) {
            Some(v) => Ok(Dynamic::from(v.clone())),
            None => Ok(Dynamic::UNIT),
        }
    }

    // Set a key in a JSON object
    #[rhai_fn(return_raw)]
    pub fn set(json: Dynamic, key: &str, value: Dynamic) -> Result<(), Box<EvalAltResult>> {
        let mut map: Value = json.try_cast::<Value>().ok_or("Expected a JSON object")?;
        if let Value::Object(ref mut obj) = map {
            let v: Value = value.try_cast::<Value>().ok_or("Expected JSON value")?;
            obj.insert(key.to_string(), v);
            Ok(())
        } else {
            Err("JSON value is not an object".into())
        }
    }
}
