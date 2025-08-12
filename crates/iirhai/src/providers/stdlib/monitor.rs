use rhai::plugin::*;
use gtk::prelude::*;
use gtk::gdk;

#[export_module]
pub mod monitor {
    pub fn count() -> i64 {
        get_monitor_count()
    }

    pub fn primary_resolution() -> (i64, i64) {
        get_primary_monitor_resolution()
    }

    pub fn all_resolutions() -> Vec<(i64, i64)> {
        get_all_monitor_resolutions()
    }

    pub fn dimensions(index: i64) -> (i64, i64, i64, i64) {
        get_monitor_dimensions(index as usize)
    }

    pub fn dpi(index: i64) -> f64 {
        get_monitor_dpi(index as usize)
    }
}

fn get_monitor_count() -> i64 {
    let display = gdk::Display::default().expect("No display found");
    display.n_monitors() as i64
}

fn get_primary_monitor_resolution() -> (i64, i64) {
    let display = gdk::Display::default().expect("No display found");
    if let Some(primary) = display.primary_monitor() {
        let rect = primary.geometry();
        (rect.width() as i64, rect.height() as i64)
    } else {
        (0, 0)
    }
}

fn get_all_monitor_resolutions() -> Vec<(i64, i64)> {
    let display = gdk::Display::default().expect("No display found");
    (0..display.n_monitors())
        .filter_map(|i| display.monitor(i))
        .map(|m| {
            let rect = m.geometry();
            (rect.width() as i64, rect.height() as i64)
        })
        .collect()
}

fn get_monitor_dimensions(index: usize) -> (i64, i64, i64, i64) {
    let display = gdk::Display::default().expect("No display found");
    if let Some(m) = display.monitor(index as i32) {
        let geom = m.geometry();
        (
            geom.x() as i64,
            geom.y() as i64,
            geom.width() as i64,
            geom.height() as i64,
        )
    } else {
        (0, 0, 0, 0)
    }
}

fn get_monitor_dpi(index: usize) -> f64 {
    let display = gdk::Display::default().expect("No display found");
    if let Some(m) = display.monitor(index as i32) {
        m.scale_factor() as f64 * 96.0 // base DPI * scale factor
    } else {
        0.0
    }
}
