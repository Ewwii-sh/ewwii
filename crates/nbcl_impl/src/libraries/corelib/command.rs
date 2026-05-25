use nbcl::{Value, error::Result};
use std::process::Command;
use crate::runtime_err;

pub fn run(args: Vec<Value>) -> Result<Value> {
    if let Value::Str(cmd) = &args[0] {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .map_err(|e| runtime_err!("Failed to run command: {}", e))?;
    }
    Ok(Value::Null)
}

pub fn run_and_read(args: Vec<Value>) -> Result<Value> {
    if let Value::Str(cmd) = &args[0] {
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| runtime_err!("Failed to run command: {}", e))?;

        return Ok(Value::Str(String::from_utf8_lossy(&output.stdout).to_string()))
    }

    Ok(Value::Str("".into()))
}
