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

    let result_value: Value = serde_json::from_str(&json)
        .map_err(|e| runtime_err!("Failed to parse JSON: {}", e))?;

    Ok(result_value)
}


