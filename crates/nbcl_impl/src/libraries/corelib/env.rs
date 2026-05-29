use crate::runtime_err;
use nbcl::{error::Result, Value};

pub fn set_env(args: Vec<Value>) -> Result<Value> {
    if let Value::Str(var) = &args[0] {
        if let Value::Str(value) = &args[1] {
            std::env::set_var(var, value);
        }
    }
    Ok(Value::Null)
}

pub fn get_env(args: Vec<Value>) -> Result<Value> {
    if let Value::Str(var) = &args[0] {
        let output = std::env::var(var).map_err(|e| runtime_err!("Failed to get env: {e}"))?;

        return Ok(Value::Str(output.to_string()));
    }

    Ok(Value::Str("".into()))
}

pub fn get_home_dir(_args: Vec<Value>) -> Result<Value> {
    let output =
        std::env::var("HOME").map_err(|e| runtime_err!("Failed to get home directory: {e}"))?;

    Ok(Value::Str(output))
}

pub fn get_current_dir(_args: Vec<Value>) -> Result<Value> {
    let output = std::env::current_dir()
        .map_err(|e| runtime_err!("Failed to get current directory: {e}"))?;

    Ok(Value::Str(output.into_os_string().into_string().unwrap_or(String::new())))
}

pub fn get_username(_args: Vec<Value>) -> Result<Value> {
    let output =
        std::env::var("USER").map_err(|e| runtime_err!("Failed to get home directory: {e}"))?;

    Ok(Value::Str(output))
}
