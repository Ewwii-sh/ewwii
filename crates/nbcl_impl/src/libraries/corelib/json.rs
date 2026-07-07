use crate::runtime_err;
use nbcl::{error::Result, Value};
use std::collections::BTreeMap;

pub fn to_json(mut args: Vec<Value>) -> Result<Value> {
    let Value::Map(map) = args.remove(0) else {
        return Err(runtime_err!("to_json() expects a map"));
    };

    let collect_map: BTreeMap<String, Value> = map.into_iter().collect();
    let json_str = serde_json::to_string(&collect_map)
        .map_err(|e| runtime_err!("Failed to serialize JSON: {}", e))?;

    Ok(Value::Str(json_str))
}

pub fn parse_json(mut args: Vec<Value>) -> Result<Value> {
    let Value::Str(json) = args.remove(0) else {
        return Err(runtime_err!("parse_json() expects a json string"));
    };

    let raw_json: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| runtime_err!("Failed to parse JSON: {}", e))?;

    Ok(convert_json_to_value(raw_json))
}

fn convert_json_to_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => Value::Str(s),
        serde_json::Value::Array(arr) => {
            Value::List(arr.into_iter().map(convert_json_to_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let pairs = obj.into_iter().map(|(k, v)| (k, convert_json_to_value(v))).collect();
            Value::Map(pairs)
        }
    }
}
