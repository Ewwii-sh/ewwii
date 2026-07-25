use crate::runtime_err;
use nbcl::{error::Result, Value};
use regex::Regex;

pub fn is_match(args: Vec<Value>) -> Result<Value> {
    if let Value::Str(text) = &args[0] {
        if let Value::Str(pattern) = &args[1] {
            let re = Regex::new(pattern)
                .map_err(|e| runtime_err!("Failed to read regex pattern: {}", e))?;
            return Ok(Value::Bool(re.is_match(text)));
        }
    }
    Ok(Value::Bool(true))
}

pub fn find(args: Vec<Value>) -> Result<Value> {
    if let Value::Str(text) = &args[0] {
        if let Value::Str(pattern) = &args[1] {
            let re = Regex::new(pattern)
                .map_err(|e| runtime_err!("Failed to read regex pattern: {}", e))?;
            match re.find(text).map(|m| m.as_str().to_string()) {
                Some(s) => return Ok(Value::Str(s)),
                None => return Ok(Value::Str(String::new())),
            };
        }
    }
    Ok(Value::Str("".into()))
}

pub fn find_all(args: Vec<Value>) -> Result<Value> {
    if let Value::Str(text) = &args[0] {
        if let Value::Str(pattern) = &args[1] {
            let re = Regex::new(pattern)
                .map_err(|e| runtime_err!("Failed to read regex pattern: {}", e))?;
            let results = re.find_iter(text).map(|m| Value::Str(m.as_str().to_string())).collect();

            return Ok(Value::List(results));
        }
    }
    Ok(Value::List(Vec::new()))
}

pub fn replace(args: Vec<Value>) -> Result<Value> {
    if let Value::Str(text) = &args[0] {
        if let Value::Str(pattern) = &args[1] {
            if let Value::Str(replacement) = &args[2] {
                let re = Regex::new(pattern)
                    .map_err(|e| runtime_err!("Failed to read regex pattern: {}", e))?;
                return Ok(Value::Str(re.replace_all(text, replacement).to_string()));
            }
        }
    }

    Ok(Value::Str("".into()))
}
