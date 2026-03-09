use anyhow::{anyhow, Result};
use rhai::{Dynamic, Map};
use std::time::Duration;
use super::variables::GlobalVar;

pub enum PropValue<T> {
    Static(T),
    Bound {
        var_name: String,
        initial: T,
        parser: fn(&str) -> Option<T>,
    },
}

impl<T: Clone> PropValue<T> {
    pub fn initial_value(&self) -> T {
        match self {
            PropValue::Static(v) => v.clone(),
            PropValue::Bound { initial, .. } => initial.clone(),
        }
    }
}

fn try_get_global_var(value: &Dynamic) -> Option<GlobalVar> {
    value.clone().try_cast::<GlobalVar>()
}

// === Typed parsers with logging ===
fn parse_string(s: &str) -> Option<String> {
    Some(s.to_string())
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.parse::<bool>() {
        Ok(v) => Some(v),
        Err(e) => {
            log::error!("Failed to parse GlobalVar value '{}' as bool: {}", s, e);
            None
        }
    }
}

fn parse_i64(s: &str) -> Option<i64> {
    match s.parse::<i64>() {
        Ok(v) => Some(v),
        Err(e) => {
            log::error!("Failed to parse GlobalVar value '{}' as i64: {}", s, e);
            None
        }
    }
}

fn parse_f64(s: &str) -> Option<f64> {
    match s.parse::<f64>() {
        Ok(v) => Some(v),
        Err(e) => {
            log::error!("Failed to parse GlobalVar value '{}' as f64: {}", s, e);
            None
        }
    }
}

fn parse_i32(s: &str) -> Option<i32> {
    match s.parse::<i32>() {
        Ok(v) => Some(v),
        Err(e) => {
            log::error!("Failed to parse GlobalVar value '{}' as i32: {}", s, e);
            None
        }
    }
}

// === prop getters ===
pub fn get_string_prop(props: &Map, key: &str, default: Option<&str>) -> Result<PropValue<String>> {
    if let Some(value) = props.get(key) {
        if let Some(var) = try_get_global_var(value) {
            return Ok(PropValue::Bound {
                var_name: var.name,
                initial: var.initial.clone()
                    .try_cast::<String>()
                    .unwrap_or_else(|| default.unwrap_or("").to_string()),
                parser: parse_string,
            });
        }
        value
            .clone()
            .try_cast::<String>()
            .map(PropValue::Static)
            .ok_or_else(|| anyhow!("Expected property `{}` to be a string", key))
    } else {
        default
            .map(|s| PropValue::Static(s.to_string()))
            .ok_or_else(|| anyhow!("Missing required string property `{}`", key))
    }
}

pub fn get_bool_prop(props: &Map, key: &str, default: Option<bool>) -> Result<PropValue<bool>> {
    if let Some(value) = props.get(key) {
        if let Some(var) = try_get_global_var(value) {
            return Ok(PropValue::Bound {
                var_name: var.name,
                initial: var.initial.clone()
                    .try_cast::<bool>()
                    .unwrap_or(default.unwrap_or(false)),
                parser: parse_bool,
            });
        }
        value
            .clone()
            .try_cast::<bool>()
            .map(PropValue::Static)
            .ok_or_else(|| anyhow!("Expected property `{}` to be a bool", key))
    } else {
        default
            .map(PropValue::Static)
            .ok_or_else(|| anyhow!("Missing required bool property `{}`", key))
    }
}

pub fn get_i64_prop(props: &Map, key: &str, default: Option<i64>) -> Result<PropValue<i64>> {
    if let Some(value) = props.get(key) {
        if let Some(var) = try_get_global_var(value) {
            return Ok(PropValue::Bound {
                var_name: var.name,
                initial: var.initial.clone()
                    .try_cast::<i64>()
                    .unwrap_or(default.unwrap_or(0)),
                parser: parse_i64,
            });
        }
        if let Some(v) = value.clone().try_cast::<i64>() {
            Ok(PropValue::Static(v))
        } else if let Some(s) = value.clone().try_cast::<String>() {
            s.parse::<i64>()
                .map(PropValue::Static)
                .map_err(|_| anyhow!("Expected property `{}` to be an i64 or numeric string", key))
        } else {
            Err(anyhow!("Expected property `{}` to be an i64 or numeric string", key))
        }
    } else {
        default
            .map(PropValue::Static)
            .ok_or_else(|| anyhow!("Missing required i64 property `{}`", key))
    }
}

pub fn get_f64_prop(props: &Map, key: &str, default: Option<f64>) -> Result<PropValue<f64>> {
    if let Some(value) = props.get(key) {
        if let Some(var) = try_get_global_var(value) {
            return Ok(PropValue::Bound {
                var_name: var.name,
                initial: var.initial.clone()
                    .try_cast::<f64>()
                    .or_else(|| var.initial.clone().try_cast::<i64>().map(|v| v as f64))
                    .unwrap_or(default.unwrap_or(0.0)),
                parser: parse_f64,
            });
        }
        if let Some(v) = value.clone().try_cast::<f64>() {
            Ok(PropValue::Static(v))
        } else if let Some(v) = value.clone().try_cast::<i64>() {
            Ok(PropValue::Static(v as f64))
        } else if let Some(s) = value.clone().try_cast::<String>() {
            s.parse::<f64>()
                .map(PropValue::Static)
                .map_err(|_| anyhow!("Expected property `{}` to be an f64, i64, or numeric string", key))
        } else {
            Err(anyhow!("Expected property `{}` to be an f64, i64, or numeric string", key))
        }
    } else {
        default
            .map(PropValue::Static)
            .ok_or_else(|| anyhow!("Missing required f64 property `{}`", key))
    }
}

pub fn get_i32_prop(props: &Map, key: &str, default: Option<i32>) -> Result<PropValue<i32>> {
    if let Some(value) = props.get(key) {
        if let Some(var) = try_get_global_var(value) {
            return Ok(PropValue::Bound {
                var_name: var.name,
                initial: var.initial.clone()
                    .try_cast::<i32>()
                    .or_else(|| var.initial.clone().try_cast::<i64>().map(|v| v as i32))
                    .unwrap_or(default.unwrap_or(0)),
                parser: parse_i32,
            });
        }
        if let Some(v) = value.clone().try_cast::<i32>() {
            Ok(PropValue::Static(v))
        } else if let Some(v) = value.clone().try_cast::<i64>() {
            if v >= i32::MIN as i64 && v <= i32::MAX as i64 {
                Ok(PropValue::Static(v as i32))
            } else {
                Err(anyhow!("Value for `{}` is out of range for i32", key))
            }
        } else if let Some(s) = value.clone().try_cast::<String>() {
            s.parse::<i32>()
                .map(PropValue::Static)
                .map_err(|_| anyhow!("Expected property `{}` to be an i32 or numeric string", key))
        } else {
            Err(anyhow!("Expected property `{}` to be an i32 or numeric string", key))
        }
    } else {
        default
            .map(PropValue::Static)
            .ok_or_else(|| anyhow!("Missing required i32 property `{}`", key))
    }
}

pub fn get_vec_string_prop(
    props: &Map,
    key: &str,
    default: Option<Vec<PropValue<String>>>,
) -> Result<Vec<PropValue<String>>> {
    if let Some(value) = props.get(key) {
        let array = value
            .clone()
            .try_cast::<Vec<Dynamic>>()
            .ok_or_else(|| anyhow!("Expected property `{}` to be a vec", key))?;

        array
            .into_iter()
            .map(|d| {
                if let Some(var) = try_get_global_var(&d) {
                    let initial = var.initial.clone()
                        .try_cast::<String>()
                        .unwrap_or_default();
                    Ok(PropValue::Bound {
                        var_name: var.name,
                        initial,
                        parser: parse_string,
                    })
                } else {
                    d.try_cast::<String>()
                        .map(PropValue::Static)
                        .ok_or_else(|| anyhow!("Expected all elements of `{}` to be strings or GlobalVars", key))
                }
            })
            .collect()
    } else {
        default.ok_or_else(|| anyhow!("Missing required vec property `{}`", key))
    }
}

fn parse_duration_str(key_str: &str) -> Option<Duration> {
    if key_str.ends_with("ms") {
        let num = &key_str[..key_str.len() - 2];
        num.parse::<u64>().ok().map(Duration::from_millis)
    } else if key_str.ends_with("min") {
        let num = &key_str[..key_str.len() - 3];
        num.parse::<u64>().ok().map(|m| Duration::from_secs(m * 60))
    } else if key_str.ends_with("m") {
        let num = &key_str[..key_str.len() - 1];
        num.parse::<u64>().ok().map(|m| Duration::from_secs(m * 60))
    } else if key_str.ends_with("h") {
        let num = &key_str[..key_str.len() - 1];
        num.parse::<u64>().ok().map(|h| Duration::from_secs(h * 3600))
    } else if key_str.ends_with("s") {
        let num = &key_str[..key_str.len() - 1];
        num.parse::<u64>().ok().map(Duration::from_secs)
    } else {
        None
    }
}

pub fn get_duration_prop(props: &Map, key: &str, default: Option<Duration>) -> Result<Duration> {
    if let Some(value) = props.get(key) {
        // Duration doesn't support GlobalVar binding. It's a static config value
        let raw = value
            .clone()
            .try_cast::<String>()
            .ok_or_else(|| anyhow!("Expected property `{}` to be a duration string", key))?;

        let key_str = raw.trim().to_ascii_lowercase();
        parse_duration_str(&key_str)
            .ok_or_else(|| anyhow!("Unsupported duration format: '{}'", key_str))
    } else {
        default.ok_or_else(|| anyhow!("Missing required duration property `{}`", key))
    }
}

// help unwrap the propvalue in cases where a static is expected
pub fn unwrap_static<T: Default>(key: &str, prop: PropValue<T>) -> T {
    match prop {
        PropValue::Static(v) => v,
        PropValue::Bound { var_name, .. } => {
            log::error!(
                "Property `{}` does not support variable binding (got GlobalVar `{}`), using default as fallback",
                key,
                var_name
            );
            T::default()
        }
    }
}