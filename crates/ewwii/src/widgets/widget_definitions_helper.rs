use crate::gtk::prelude::LabelExt;
use anyhow::{anyhow, Result};
use gtk::pango;
use rhai::Map;
use std::process::Command;
use std::time::Duration;

/// General purpose
pub(super) fn get_string_prop(props: &Map, key: &str, default: Option<&str>) -> Result<String> {
    if let Some(value) = props.get(key) {
        value.clone().try_cast::<String>().ok_or_else(|| anyhow!("Expected property `{}` to be a string", key))
    } else {
        default.map(|s| s.to_string()).ok_or_else(|| anyhow!("Missing required string property `{}`", key))
    }
}

pub(super) fn get_bool_prop(props: &Map, key: &str, default: Option<bool>) -> Result<bool> {
    if let Some(value) = props.get(key) {
        value.clone().try_cast::<bool>().ok_or_else(|| anyhow!("Expected property `{}` to be a bool", key))
    } else {
        default.map(|s| s).ok_or_else(|| anyhow!("Missing required bool property `{}`", key))
    }
}

pub(super) fn get_i64_prop(props: &Map, key: &str, default: Option<i64>) -> Result<i64> {
    if let Some(value) = props.get(key) {
        value.clone().try_cast::<i64>().ok_or_else(|| anyhow!("Expected property `{}` to be an i64", key))
    } else {
        default.map(|s| s).ok_or_else(|| anyhow!("Missing required i64 property `{}`", key))
    }
}

pub(super) fn get_f64_prop(props: &Map, key: &str, default: Option<f64>) -> Result<f64> {
    if let Some(value) = props.get(key) {
        value.clone().try_cast::<f64>().ok_or_else(|| anyhow!("Expected property `{}` to be an f64", key))
    } else {
        default.map(|s| s).ok_or_else(|| anyhow!("Missing required f64 property `{}`", key))
    }
}

pub(super) fn get_i32_prop(props: &Map, key: &str, default: Option<i32>) -> Result<i32> {
    if let Some(value) = props.get(key) {
        value.clone().try_cast::<i32>().ok_or_else(|| anyhow!("Expected property `{}` to be an i32", key))
    } else {
        default.map(|s| s).ok_or_else(|| anyhow!("Missing required i32 property `{}`", key))
    }
}

pub(super) fn get_duration_prop(props: &Map, key: &str, default: Option<Duration>) -> Result<Duration> {
    let key_str = get_string_prop(props, key, None)?.trim().to_ascii_lowercase();

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
}

// Run a command and get the output
pub(super) fn run_command<T>(timeout: std::time::Duration, cmd: &str, args: &[T])
where
    T: 'static + std::fmt::Display + Send + Sync + Clone,
{
    use wait_timeout::ChildExt;
    let cmd = replace_placeholders(cmd, args);
    std::thread::Builder::new()
        .name("command-execution-thread".to_string())
        .spawn(move || {
            log::debug!("Running command from widget [timeout: {}ms]: {}", timeout.as_millis(), cmd);
            let child = Command::new("/bin/sh").arg("-c").arg(&cmd).spawn();
            match child {
                Ok(mut child) => match child.wait_timeout(timeout) {
                    // child timed out
                    Ok(None) => {
                        log::error!("WARNING: command {} timed out", &cmd);
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                    Err(err) => log::error!("Failed to execute command {}: {}", cmd, err),
                    Ok(Some(_)) => {}
                },
                Err(err) => log::error!("Failed to launch child process: {}", err),
            }
        })
        .expect("Failed to start command-execution-thread");
}

/// Gtk Box
pub(super) fn parse_orientation(ori: &str) -> Result<gtk::Orientation> {
    match ori.to_ascii_lowercase().as_str() {
        "h" | "horizontal" => Ok(gtk::Orientation::Horizontal),
        "v" | "vertical" => Ok(gtk::Orientation::Vertical),
        other => Err(anyhow!("Invalid orientation: {}", other)),
    }
}

/// Gtk Label
pub(super) fn apply_ellipsize_settings(label: &gtk::Label, truncate: bool, limit_width: i32, truncate_left: bool, show: bool) {
    if (truncate || limit_width != i32::MAX) && show {
        label.set_max_width_chars(if limit_width == i32::MAX { -1 } else { limit_width });
        label.set_ellipsize(if truncate_left { pango::EllipsizeMode::Start } else { pango::EllipsizeMode::End });
    } else {
        label.set_ellipsize(pango::EllipsizeMode::None);
    }
}

pub(super) fn parse_gravity(s: &str) -> Result<pango::Gravity> {
    match s.to_ascii_lowercase().as_str() {
        "south" => Ok(pango::Gravity::South),
        "north" => Ok(pango::Gravity::North),
        "east" => Ok(pango::Gravity::East),
        "west" => Ok(pango::Gravity::West),
        "auto" => Ok(pango::Gravity::Auto),
        _ => Err(anyhow!("Invalid gravity: '{}'", s)),
    }
}

pub(super) fn parse_justification(s: &str) -> Result<gtk::Justification> {
    match s.to_ascii_lowercase().as_str() {
        "left" => Ok(gtk::Justification::Left),
        "right" => Ok(gtk::Justification::Right),
        "center" => Ok(gtk::Justification::Center),
        "fill" => Ok(gtk::Justification::Fill),
        _ => Err(anyhow!("Invalid justification: '{}'", s)),
    }
}

pub(super) fn parse_wrap_mode(s: &str) -> Result<pango::WrapMode> {
    match s.to_ascii_lowercase().as_str() {
        "word" => Ok(pango::WrapMode::Word),
        "char" => Ok(pango::WrapMode::Char),
        "wordchar" | "word_char" | "word-char" => Ok(pango::WrapMode::WordChar),
        _ => Err(anyhow!("Invalid wrap_mode: '{}'", s)),
    }
}

/// Gtk scale (slider)
pub(super) fn parse_position_type(s: &str) -> Result<gtk::PositionType> {
    match s.to_ascii_lowercase().as_str() {
        "left" => Ok(gtk::PositionType::Left),
        "right" => Ok(gtk::PositionType::Right),
        "top" => Ok(gtk::PositionType::Top),
        "bottom" => Ok(gtk::PositionType::Bottom),
        _ => Err(anyhow!("Invalid position type: '{}'", s)),
    }
}

/// Helper of helpers
fn replace_placeholders<T>(cmd: &str, args: &[T]) -> String
where
    T: 'static + std::fmt::Display + Send + Sync + Clone,
{
    if !args.is_empty() {
        let cmd = cmd.replace("{}", &format!("{}", args[0]));
        args.iter().enumerate().fold(cmd, |acc, (i, arg)| acc.replace(&format!("{{{}}}", i), &format!("{}", arg)))
    } else {
        cmd.to_string()
    }
}
