use crate::gtk::prelude::LabelExt;
use anyhow::{anyhow, Result};
use gtk::pango;
use rhai::Map;
use std::process::Command;
use std::time::Duration;

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

/// ALL WIDGETS
pub(super) fn parse_align(o: &str) -> Result<gtk::Align> {
    match o.to_ascii_lowercase().as_str() {
        "fill" => Ok(gtk::Align::Fill),
        "baseline" => Ok(gtk::Align::Baseline),
        "center" => Ok(gtk::Align::Center),
        "start" => Ok(gtk::Align::Start),
        "end" => Ok(gtk::Align::End),
        other => Err(anyhow!("Invalid alignment: {}", other)),
    }
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

/// Revealer
pub(super) fn parse_revealer_transition(t: &str) -> Result<gtk::RevealerTransitionType> {
    match t.to_ascii_lowercase().as_str() {
        "slideright" => Ok(gtk::RevealerTransitionType::SlideRight),
        "slideleft" => Ok(gtk::RevealerTransitionType::SlideLeft),
        "slideup" => Ok(gtk::RevealerTransitionType::SlideUp),
        "slidedown" => Ok(gtk::RevealerTransitionType::SlideDown),
        "fade" | "crossfade" => Ok(gtk::RevealerTransitionType::Crossfade),
        "none" => Ok(gtk::RevealerTransitionType::None),
        _ => Err(anyhow!("Invalid transition: '{}'", t)),
    }
}

/// Gtk Image
// icon-size - "menu", "small-toolbar", "toolbar", "large-toolbar", "button", "dnd", "dialog"
pub(super) fn parse_icon_size(o: &str) -> Result<gtk::IconSize> {
    match o.to_ascii_lowercase().as_str() {
        "menu" => Ok(gtk::IconSize::Menu),
        "small-toolbar" | "toolbar" => Ok(gtk::IconSize::SmallToolbar),
        "large-toolbar" => Ok(gtk::IconSize::LargeToolbar),
        "button" => Ok(gtk::IconSize::Button),
        "dnd" => Ok(gtk::IconSize::Dnd),
        "dialog" => Ok(gtk::IconSize::Dialog),
        _ => Err(anyhow!("Invalid icon size: '{}'", 0)),
    }
}