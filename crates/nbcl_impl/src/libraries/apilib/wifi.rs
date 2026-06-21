use crate::runtime_err;
use nbcl::{error::Result, Value};
use std::process::Command;

pub fn scan(_args: Vec<Value>) -> Result<Value> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "SSID,SIGNAL,SECURITY", "dev", "wifi"])
        .output()
        .map_err(|e| runtime_err!("Failed to run nmcli: {e}"))?;

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| runtime_err!("Invalid UTF-8 output from nmcli: {e}"))?;

    let mut result = Vec::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() != 3 {
            continue;
        }
        let mut map = Vec::new();
        map.push(("ssid".into(), Value::Str(parts[0].into())));
        map.push(("signal".into(), Value::Str(parts[1].into())));
        map.push(("security".into(), Value::Str(parts[2].into())));
        result.push(Value::Map(map));
    }
    Ok(Value::List(result))
}

pub fn current_connection(_args: Vec<Value>) -> Result<Value> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "ACTIVE,SSID,SIGNAL,SECURITY", "device", "wifi", "list"])
        .output()
        .map_err(|e| runtime_err!("Failed to run nmcli: {e}"))?;
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| runtime_err!("Invalid UTF-8 output: {}", e))?;
    let mut map = Vec::new();
    if let Some(line) = stdout.lines().find(|l| l.starts_with("yes:")) {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 4 {
            map.push(("ssid".into(), Value::Str(parts[1].into())));
            map.push(("signal".into(), Value::Str(parts[2].into())));
            map.push(("security".into(), Value::Str(parts[3].into())));
        }
    }
    Ok(Value::Map(map))
}

// ssid: &str, password: &str
pub fn connect(args: Vec<Value>) -> Result<Value> {
    let Value::Str(ssid) = &args[0] else {
        return Err(runtime_err!("SSID must be a string"));
    };
    let Value::Str(password) = &args[1] else {
        return Err(runtime_err!("PASSWORD must be a string"));
    };

    let args = vec!["dev", "wifi", "connect", ssid, "password", password];
    let status = Command::new("nmcli")
        .args(&args)
        .status()
        .map_err(|e| runtime_err!("Failed to run nmcli: {e}"))?;
    if status.success() {
        Ok(Value::Null)
    } else {
        Err(runtime_err!("Failed to connect to {}", ssid))
    }
}

// ssid: &str
pub fn connect_without_password(args: Vec<Value>) -> Result<Value> {
    let Value::Str(ssid) = &args[0] else {
        return Err(runtime_err!("SSID must be a string"));
    };

    let args = vec!["dev", "wifi", "connect", ssid];
    let status = Command::new("nmcli")
        .args(&args)
        .status()
        .map_err(|e| runtime_err!("Failed to run nmcli: {e}"))?;
    if status.success() {
        Ok(Value::Null)
    } else {
        Err(runtime_err!("Failed to connect to {}", ssid))
    }
}

pub fn disconnect(_args: Vec<Value>) -> Result<Value> {
    use std::process::Command;

    // Get current active SSID
    let output = Command::new("nmcli")
        .args(["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()
        .map_err(|e| runtime_err!("Failed to run nmcli: {e}"))?;

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| runtime_err!("Invalid UTF-8 from nmcli: {e}"))?;

    let ssid = stdout
        .lines()
        .find(|line| line.starts_with("yes:"))
        .and_then(|line| line.split(':').nth(1))
        .ok_or(runtime_err!("No active Wi-Fi connection"))?;

    let status = Command::new("nmcli")
        .args(["connection", "down", "id", ssid])
        .status()
        .map_err(|e| runtime_err!("Failed to disconnect from {}: {e}", ssid))?;

    if status.success() {
        Ok(Value::Null)
    } else {
        Err(runtime_err!("Failed to disconnect from {}", ssid))
    }
}

pub fn disable_adapter(_args: Vec<Value>) -> Result<Value> {
    let status = Command::new("nmcli")
        .args(["networking", "off"])
        .status()
        .map_err(|e| runtime_err!("Failed to run nmcli: {e}"))?;
    if status.success() {
        Ok(Value::Null)
    } else {
        Err(runtime_err!("Failed to disable adapter"))
    }
}

pub fn enable_adapter(_args: Vec<Value>) -> Result<Value> {
    let status = Command::new("nmcli")
        .args(["networking", "on"])
        .status()
        .map_err(|e| runtime_err!("Failed to run nmcli: {e}"))?;
    if status.success() {
        Ok(Value::Null)
    } else {
        Err(runtime_err!("Failed to enable"))
    }
}

pub fn get_adapter_connectivity(_args: Vec<Value>) -> Result<Value> {
    let output = Command::new("nmcli")
        .args(["networking", "connectivity"])
        .output()
        .map_err(|e| runtime_err!("Failed to run nmcli: {e}"))?;

    if output.status.success() {
        let connectivity = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Value::Str(connectivity))
    } else {
        Err(runtime_err!("Failed to get connectivity"))
    }
}
