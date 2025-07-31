use anyhow::{anyhow, Result};

pub(super) fn parse_orientation(ori: &str) -> Result<gtk::Orientation> {
    match ori.to_ascii_lowercase().as_str() {
        "h" | "horizontal" => Ok(gtk::Orientation::Horizontal),
        "v" | "vertical" => Ok(gtk::Orientation::Vertical),
        other => Err(anyhow!("Invalid orientation: {}", other)),
    }
}
