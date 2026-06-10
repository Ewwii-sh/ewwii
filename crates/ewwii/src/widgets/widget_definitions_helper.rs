use anyhow::{anyhow, Result};
use gtk4::pango;
use std::process::Command;

use crate::widgets::graph::{LineStyle, RenderType};

// Run a command and get the output
pub(super) fn run_command<T>(timeout: std::time::Duration, cmd: &str, args: &[T])
where
    T: 'static + std::fmt::Display + Send + Sync + Clone,
{
    if cmd.is_empty() {
        return;
    }

    use wait_timeout::ChildExt;
    let cmd = replace_placeholders(cmd, args);
    std::thread::Builder::new()
        .name("command-execution-thread".to_string())
        .spawn(move || {
            log::debug!(
                "Running command from widget [timeout: {}ms]: {}",
                timeout.as_millis(),
                cmd
            );
            let mut command = Command::new("/bin/sh");
            command.arg("-c").arg(&cmd);

            let child = command.spawn();
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
pub(super) fn parse_align(o: &str) -> Result<gtk4::Align> {
    match o.to_ascii_lowercase().as_str() {
        "fill" => Ok(gtk4::Align::Fill),
        "baseline" => Ok(gtk4::Align::Baseline),
        "center" => Ok(gtk4::Align::Center),
        "start" => Ok(gtk4::Align::Start),
        "end" => Ok(gtk4::Align::End),
        other => Err(anyhow!("Invalid alignment: {}", other)),
    }
}

/// Gtk Box
pub(super) fn parse_orientation(ori: &str) -> Result<gtk4::Orientation> {
    match ori.to_ascii_lowercase().as_str() {
        "h" | "horizontal" => Ok(gtk4::Orientation::Horizontal),
        "v" | "vertical" => Ok(gtk4::Orientation::Vertical),
        other => Err(anyhow!("Invalid orientation: {}", other)),
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

pub(super) fn parse_justification(s: &str) -> Result<gtk4::Justification> {
    match s.to_ascii_lowercase().as_str() {
        "left" => Ok(gtk4::Justification::Left),
        "right" => Ok(gtk4::Justification::Right),
        "center" => Ok(gtk4::Justification::Center),
        "fill" => Ok(gtk4::Justification::Fill),
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
pub(super) fn parse_position_type(s: &str) -> Result<gtk4::PositionType> {
    match s.to_ascii_lowercase().as_str() {
        "left" => Ok(gtk4::PositionType::Left),
        "right" => Ok(gtk4::PositionType::Right),
        "top" => Ok(gtk4::PositionType::Top),
        "bottom" => Ok(gtk4::PositionType::Bottom),
        _ => Err(anyhow!("Invalid position type: '{}'", s)),
    }
}

/// Gtk flow box
pub(super) fn parse_selection_model(s: &str) -> Result<gtk4::SelectionMode> {
    match s.to_ascii_lowercase().as_str() {
        "none" => Ok(gtk4::SelectionMode::None),
        "single" => Ok(gtk4::SelectionMode::Single),
        "browse" => Ok(gtk4::SelectionMode::Browse),
        "multiple" => Ok(gtk4::SelectionMode::Multiple),
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
        args.iter()
            .enumerate()
            .fold(cmd, |acc, (i, arg)| acc.replace(&format!("{{{}}}", i), &format!("{}", arg)))
    } else {
        cmd.to_string()
    }
}

/// Revealer
pub(super) fn parse_revealer_transition(t: &str) -> Result<gtk4::RevealerTransitionType> {
    match t.to_ascii_lowercase().as_str() {
        "slideright" => Ok(gtk4::RevealerTransitionType::SlideRight),
        "slideleft" => Ok(gtk4::RevealerTransitionType::SlideLeft),
        "slideup" => Ok(gtk4::RevealerTransitionType::SlideUp),
        "slidedown" => Ok(gtk4::RevealerTransitionType::SlideDown),
        "fade" | "crossfade" => Ok(gtk4::RevealerTransitionType::Crossfade),
        "none" => Ok(gtk4::RevealerTransitionType::None),
        _ => Err(anyhow!("Invalid transition: '{}'", t)),
    }
}

/// Event box
// dragtype - "file", "text"
pub(super) enum DragEntryType {
    File,
    Text,
}

pub(super) fn parse_dragtype(o: &str) -> Result<DragEntryType> {
    match o.to_ascii_lowercase().as_str() {
        "file" => Ok(DragEntryType::File),
        "text" => Ok(DragEntryType::Text),
        _ => Err(anyhow!("Invalid drag type: '{}'", o)),
    }
}

/// Stack widget
// transition - "slideright", "slideleft", "slideup", "slidedown", "crossfade", "none"
pub(super) fn parse_stack_transition(t: &str) -> Result<gtk4::StackTransitionType> {
    match t.to_ascii_lowercase().as_str() {
        "slideright" => Ok(gtk4::StackTransitionType::SlideRight),
        "slideleft" => Ok(gtk4::StackTransitionType::SlideLeft),
        "slideup" => Ok(gtk4::StackTransitionType::SlideUp),
        "slidedown" => Ok(gtk4::StackTransitionType::SlideDown),
        "fade" | "crossfade" => Ok(gtk4::StackTransitionType::Crossfade),
        "none" => Ok(gtk4::StackTransitionType::None),
        _ => Err(anyhow!("Invalid stack transition: '{}'", t)),
    }
}

/// Graph line style
pub(super) fn parse_graph_line_style(t: &str) -> Result<LineStyle> {
    match t.to_ascii_lowercase().as_str() {
        "miter" => Ok(LineStyle::Miter),
        "bevel" => Ok(LineStyle::Bevel),
        "round" => Ok(LineStyle::Round),
        _ => Err(anyhow!("Invalid graph line style: '{}'", t)),
    }
}

/// Graph render type
pub(super) fn parse_graph_render_type(t: &str) -> Result<RenderType> {
    match t.to_ascii_lowercase().as_str() {
        "line" => Ok(RenderType::Line),
        "step-line" => Ok(RenderType::StepLine),
        "fill" => Ok(RenderType::Fill),
        "step-fill" => Ok(RenderType::StepFill),
        _ => Err(anyhow!("Invalid graph render type: '{}'", t)),
    }
}

/// Picture widget
pub(super) fn parse_content_fit(cf: &str) -> Result<gtk4::ContentFit> {
    match cf.to_ascii_lowercase().as_str() {
        "fill" => Ok(gtk4::ContentFit::Fill),
        "contain" => Ok(gtk4::ContentFit::Contain),
        "cover" => Ok(gtk4::ContentFit::Cover),
        "scaledown" => Ok(gtk4::ContentFit::ScaleDown),
        _ => Err(anyhow!("Invalid content fit: '{}'", cf)),
    }
}
