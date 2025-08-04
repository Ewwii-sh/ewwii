use anyhow::{anyhow, Result};
use rhai::{Map, Dynamic};
use std::time::Duration;

/// General purpose helpers
pub fn get_string_prop(props: &Map, key: &str, default: Option<&str>) -> Result<String> {
    if let Some(value) = props.get(key) {
        value.clone().try_cast::<String>().ok_or_else(|| anyhow!("Expected property `{}` to be a string", key))
    } else {
        default.map(|s| s.to_string()).ok_or_else(|| anyhow!("Missing required string property `{}`", key))
    }
}

pub fn get_bool_prop(props: &Map, key: &str, default: Option<bool>) -> Result<bool> {
    if let Some(value) = props.get(key) {
        value.clone().try_cast::<bool>().ok_or_else(|| anyhow!("Expected property `{}` to be a bool", key))
    } else {
        default.map(|s| s).ok_or_else(|| anyhow!("Missing required bool property `{}`", key))
    }
}

pub fn get_i64_prop(props: &Map, key: &str, default: Option<i64>) -> Result<i64> {
    if let Some(value) = props.get(key) {
        value.clone().try_cast::<i64>().ok_or_else(|| anyhow!("Expected property `{}` to be an i64", key))
    } else {
        default.map(|s| s).ok_or_else(|| anyhow!("Missing required i64 property `{}`", key))
    }
}

pub fn get_f64_prop(props: &Map, key: &str, default: Option<f64>) -> Result<f64> {
    if let Some(value) = props.get(key) {
        value.clone().try_cast::<f64>().ok_or_else(|| anyhow!("Expected property `{}` to be an f64", key))
    } else {
        default.map(|s| s).ok_or_else(|| anyhow!("Missing required f64 property `{}`", key))
    }
}

pub fn get_i32_prop(props: &Map, key: &str, default: Option<i32>) -> Result<i32> {
    if let Some(value) = props.get(key) {
        value.clone().try_cast::<i32>().ok_or_else(|| anyhow!("Expected property `{}` to be an i32", key))
    } else {
        default.map(|s| s).ok_or_else(|| anyhow!("Missing required i32 property `{}`", key))
    }
}


pub fn get_vec_string_prop(props: &Map, key: &str, default: Option<Vec<String>>) -> Result<Vec<String>> {
    if let Some(value) = props.get(key) {
        let array = value.clone().try_cast::<Vec<Dynamic>>()
            .ok_or_else(|| anyhow!("Expected property `{}` to be a vec", key))?;

        array.into_iter()
            .map(|d| d.try_cast::<String>().ok_or_else(|| anyhow!("Expected all elements of `{}` to be strings", key)))
            .collect()
    } else {
        default.ok_or_else(|| anyhow!("Missing required vec property `{}`", key))
    }
}

pub fn get_duration_prop(props: &Map, key: &str, default: Option<Duration>) -> Result<Duration> {
    if let Ok(raw) = get_string_prop(props, key, None) {
        let key_str = raw.trim().to_ascii_lowercase();
        if key_str.ends_with("ms") {
            let num = &key_str[..key_str.len() - 2];
            let ms = num.parse::<u64>().map_err(|_| anyhow!("Invalid ms value: '{}'", key_str))?;
            Ok(Duration::from_millis(ms))
        } else if key_str.ends_with("s") {
            let num = &key_str[..key_str.len() - 1];
            let s = num.parse::<u64>().map_err(|_| anyhow!("Invalid s value: '{}'", key_str))?;
            Ok(Duration::from_secs(s))
        } else if key_str.ends_with("min") {
            let num = &key_str[..key_str.len() - 3];
            let mins = num.parse::<u64>().map_err(|_| anyhow!("Invalid min value: '{}'", key_str))?;
            Ok(Duration::from_secs(mins * 60))
        } else if key_str.ends_with("h") {
            let num = &key_str[..key_str.len() - 1];
            let hrs = num.parse::<u64>().map_err(|_| anyhow!("Invalid h value: '{}'", key_str))?;
            Ok(Duration::from_secs(hrs * 3600))
        } else {
            Err(anyhow!("Unsupported duration format: '{}'", key_str))
        }
    } else {
        default.ok_or_else(|| anyhow!("No value for duration and no default provided"))
    }
}
