use super::variables::{GlobalCompare, GlobalVar};
use crate::prop::PropertyMap;
use anyhow::{anyhow, Result};
use std::time::Duration;

pub enum PropValue<T> {
    Static(T),
    Compare { comp: GlobalCompare, parser: fn(&str) -> Option<T> },
    Bound { var_name: String, initial: T, parser: fn(&str) -> Option<T> },
}

impl<T: Clone + Default> PropValue<T> {
    pub fn initial_value(&self) -> T {
        match self {
            PropValue::Static(v) => v.clone(),
            PropValue::Bound { initial, .. } => initial.clone(),
            PropValue::Compare { .. } => T::default(),
        }
    }
}

// === Helpers ===
fn make_bound<T>(var: GlobalVar, default: T, parser: fn(&str) -> Option<T>) -> PropValue<T>
where
    T: Clone + 'static,
{
    let initial_val = var.initial.as_str().and_then(|s| parser(s)).unwrap_or(default);

    PropValue::Bound { var_name: var.name, initial: initial_val, parser }
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
pub fn get_string_prop(
    props: &PropertyMap,
    key: &str,
    default: Option<&str>,
) -> Result<PropValue<String>> {
    if let Some(value) = props.get(key) {
        if let Some(var) = value.as_global_var() {
            return Ok(make_bound(var.clone(), default.unwrap_or("").to_string(), parse_string));
        }
        if let Some(var) = value.as_global_compare() {
            return Ok(PropValue::Compare { comp: var.clone(), parser: parse_string });
        }

        value
            .as_str()
            .map(String::from)
            .map(PropValue::Static)
            .ok_or_else(|| anyhow!("Expected property `{}` to be a string", key))
    } else {
        default
            .map(|s| PropValue::Static(s.to_string()))
            .ok_or_else(|| anyhow!("Missing required string property `{}`", key))
    }
}

pub fn get_bool_prop(
    props: &PropertyMap,
    key: &str,
    default: Option<bool>,
) -> Result<PropValue<bool>> {
    if let Some(value) = props.get(key) {
        if let Some(var) = value.as_global_var() {
            return Ok(make_bound(var.clone(), default.unwrap_or(false), parse_bool));
        }
        if let Some(var) = value.as_global_compare() {
            return Ok(PropValue::Compare { comp: var.clone(), parser: parse_bool });
        }

        value
            .as_bool()
            .map(PropValue::Static)
            .ok_or_else(|| anyhow!("Expected property `{}` to be a bool", key))
    } else {
        default
            .map(PropValue::Static)
            .ok_or_else(|| anyhow!("Missing required bool property `{}`", key))
    }
}

pub fn get_i64_prop(
    props: &PropertyMap,
    key: &str,
    default: Option<i64>,
) -> Result<PropValue<i64>> {
    if let Some(value) = props.get(key) {
        if let Some(var) = value.as_global_var() {
            return Ok(make_bound(var.clone(), default.unwrap_or(0), parse_i64));
        }
        if let Some(var) = value.as_global_compare() {
            return Ok(PropValue::Compare { comp: var.clone(), parser: parse_i64 });
        }

        // as_int is i64
        if let Some(v) = value.as_int() {
            Ok(PropValue::Static(v))
        } else if let Some(s) = value.as_str() {
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

pub fn get_f64_prop(
    props: &PropertyMap,
    key: &str,
    default: Option<f64>,
) -> Result<PropValue<f64>> {
    if let Some(value) = props.get(key) {
        if let Some(var) = value.as_global_var() {
            let initial = var
                .initial
                .clone()
                .as_float()
                .or_else(|| var.initial.clone().as_int().map(|v| v as f64))
                .unwrap_or(default.unwrap_or(0.0));
            return Ok(PropValue::Bound { var_name: var.name.clone(), initial, parser: parse_f64 });
        }
        if let Some(var) = value.as_global_compare() {
            return Ok(PropValue::Compare { comp: var.clone(), parser: parse_f64 });
        }

        // as_float is f64
        if let Some(v) = value.as_float() {
            Ok(PropValue::Static(v))
        } else if let Some(v) = value.as_int() {
            Ok(PropValue::Static(v as f64))
        } else if let Some(s) = value.as_str() {
            s.parse::<f64>().map(PropValue::Static).map_err(|_| {
                anyhow!("Expected property `{}` to be an f64, i64, or numeric string", key)
            })
        } else {
            Err(anyhow!("Expected property `{}` to be an f64, i64, or numeric string", key))
        }
    } else {
        default
            .map(PropValue::Static)
            .ok_or_else(|| anyhow!("Missing required f64 property `{}`", key))
    }
}

pub fn get_i32_prop(
    props: &PropertyMap,
    key: &str,
    default: Option<i32>,
) -> Result<PropValue<i32>> {
    if let Some(value) = props.get(key) {
        if let Some(var) = value.as_global_var() {
            let initial =
                var.initial.clone().as_int().map(|v| v as i32).unwrap_or(default.unwrap_or(0));
            return Ok(PropValue::Bound { var_name: var.name.clone(), initial, parser: parse_i32 });
        }
        if let Some(var) = value.as_global_compare() {
            return Ok(PropValue::Compare { comp: var.clone(), parser: parse_i32 });
        }

        // as_int is i64
        if let Some(v) = value.as_int() {
            if v >= i32::MIN as i64 && v <= i32::MAX as i64 {
                Ok(PropValue::Static(v as i32))
            } else {
                Err(anyhow!("Value for `{}` is out of range for i32", key))
            }
        } else if let Some(s) = value.as_str() {
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
    props: &PropertyMap,
    key: &str,
    default: Option<Vec<PropValue<String>>>,
) -> Result<Vec<PropValue<String>>> {
    if let Some(value) = props.get(key) {
        let array =
            value.as_array().ok_or_else(|| anyhow!("Expected property `{}` to be a vec", key))?;

        array
            .into_iter()
            .map(|d| {
                if let Some(var) = d.as_global_var() {
                    let initial = var.initial.as_str().map(String::from).unwrap_or_default();
                    Ok(PropValue::Bound {
                        var_name: var.name.clone(),
                        initial,
                        parser: parse_string,
                    })
                } else if let Some(var) = d.as_global_compare() {
                    return Ok(PropValue::Compare { comp: var.clone(), parser: parse_string });
                } else {
                    d.as_str().map(String::from).map(PropValue::Static).ok_or_else(|| {
                        anyhow!("Expected all elements of `{}` to be strings or GlobalVars", key)
                    })
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
        let num = &key_str[..key_str.len()];
        num.parse::<u64>().ok().map(Duration::from_secs)
    }
}

pub fn get_duration_prop(
    props: &PropertyMap,
    key: &str,
    default: Option<Duration>,
) -> Result<Duration> {
    if let Some(value) = props.get(key) {
        // Duration doesn't support GlobalVar binding. It's a static config value
        let raw = value
            .as_str()
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
        PropValue::Compare { .. } => {
            log::error!(
                "Property `{}` does not support variable binding comparision (got GlobalCompare), using default as fallback",
                key,
            );
            T::default()
        }
    }
}

// == Tests ==
#[cfg(test)]
mod tests {
    use super::*;
    use crate::prop::{Property, PropertyMap};
    use std::time::Duration;

    // parse_duration_str
    #[test]
    fn parse_duration_milliseconds() {
        assert_eq!(parse_duration_str("100ms"), Some(Duration::from_millis(100)));
    }

    #[test]
    fn parse_duration_seconds() {
        assert_eq!(parse_duration_str("5s"), Some(Duration::from_secs(5)));
    }

    #[test]
    fn parse_duration_minutes() {
        assert_eq!(parse_duration_str("2m"), Some(Duration::from_secs(120)));
        assert_eq!(parse_duration_str("2min"), Some(Duration::from_secs(120)));
    }

    #[test]
    fn parse_duration_hours() {
        assert_eq!(parse_duration_str("3h"), Some(Duration::from_secs(10800)));
    }

    #[test]
    fn parse_duration_zero_ms() {
        assert_eq!(parse_duration_str("0ms"), Some(Duration::from_millis(0)));
    }

    #[test]
    fn parse_duration_no_suffix() {
        assert_eq!(parse_duration_str("100"), Some(Duration::from_secs(100)));
    }

    #[test]
    fn parse_duration_empty_string() {
        assert_eq!(parse_duration_str(""), None);
    }

    #[test]
    fn parse_duration_suffix_only_no_number() {
        assert_eq!(parse_duration_str("ms"), None);
        assert_eq!(parse_duration_str("s"), None);
        assert_eq!(parse_duration_str("m"), None);
        assert_eq!(parse_duration_str("h"), None);
    }

    #[test]
    fn parse_duration_non_numeric() {
        assert_eq!(parse_duration_str("abcs"), None);
        assert_eq!(parse_duration_str("xyzms"), None);
    }

    #[test]
    fn parse_duration_negative_not_supported() {
        // u64 parse will fail on negative numbers
        assert_eq!(parse_duration_str("-5s"), None);
    }

    // get_i32_prop
    #[test]
    fn i32_prop_from_i64_in_range() {
        let mut map = PropertyMap::new();
        map.insert("val".into(), rhai::Dynamic::from(100i64));
        let result = get_i32_prop(&map, "val", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().initial_value(), 100i32);
    }

    #[test]
    fn i32_prop_from_i64_out_of_range() {
        let mut map = PropertyMap::new();
        map.insert("val".into(), rhai::Dynamic::from(i32::MAX as i64 + 1));
        let result = get_i32_prop(&map, "val", None);
        assert!(result.is_err());
    }

    #[test]
    fn i32_prop_from_numeric_string() {
        let mut map = PropertyMap::new();
        map.insert("val".into(), rhai::Dynamic::from("42".to_string()));
        let result = get_i32_prop(&map, "val", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().initial_value(), 42i32);
    }

    #[test]
    fn i32_prop_from_non_numeric_string_errors() {
        let mut map = PropertyMap::new();
        map.insert("val".into(), rhai::Dynamic::from("hello".to_string()));
        let result = get_i32_prop(&map, "val", None);
        assert!(result.is_err());
    }

    #[test]
    fn i32_prop_missing_with_default() {
        let map = PropertyMap::new();
        let result = get_i32_prop(&map, "val", Some(5));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().initial_value(), 5i32);
    }

    #[test]
    fn i32_prop_missing_no_default_errors() {
        let map = PropertyMap::new();
        let result = get_i32_prop(&map, "val", None);
        assert!(result.is_err());
    }

    // get_f64_prop
    #[test]
    fn f64_prop_from_i64_coerces() {
        let mut map = PropertyMap::new();
        map.insert("val".into(), rhai::Dynamic::from(3i64));
        let result = get_f64_prop(&map, "val", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().initial_value(), 3.0f64);
    }

    #[test]
    fn f64_prop_from_numeric_string() {
        let mut map = PropertyMap::new();
        map.insert("val".into(), rhai::Dynamic::from("1.5".to_string()));
        let result = get_f64_prop(&map, "val", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().initial_value(), 1.5f64);
    }

    #[test]
    fn f64_prop_missing_no_default_errors() {
        let map = PropertyMap::new();
        let result = get_f64_prop(&map, "val", None);
        assert!(result.is_err());
    }
}
