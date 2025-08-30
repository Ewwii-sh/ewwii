use gtk::gdk;
use gtk::prelude::*;
use rhai::plugin::*;

#[export_module]
pub mod monitor {
    pub fn count() -> i64 {
        get_monitor_count()
    }

    pub fn primary_resolution() -> Vec<i64> {
        let (w, h) = get_primary_monitor_resolution();
        vec![w, h]
    }

    pub fn primary_resolution_str() -> String {
        let (w, h) = get_primary_monitor_resolution();
        format!("{w}x{h}")
    }

    pub fn all_resolutions() -> Vec<Vec<i64>> {
        get_all_monitor_resolutions().into_iter().map(|(w, h)| vec![w, h]).collect()
    }

    pub fn all_resolutions_str() -> String {
        get_all_monitor_resolutions()
            .into_iter()
            .map(|(w, h)| format!("{w}x{h}"))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn dimensions(index: i64) -> Vec<i64> {
        let (x, y, w, h) = get_monitor_dimensions(index as usize);
        vec![x, y, w, h]
    }

    pub fn dimensions_str(index: i64) -> String {
        let (x, y, w, h) = get_monitor_dimensions(index as usize);
        format!("{x},{y} - {w}x{h}")
    }

    pub fn dpi(index: i64) -> f64 {
        get_monitor_dpi(index as usize)
    }

    pub fn dpi_str(index: i64) -> String {
        format!("{:.1}", get_monitor_dpi(index as usize))
    }
}

fn ensure_gdk_init() {
    if gtk::is_initialized_main_thread() {
        return;
    }
    gtk::init().expect("Failed to initialize GTK");
}

fn get_monitor_count() -> i64 {
    ensure_gdk_init();
    let display = gdk::Display::default().expect("No display found");
    display.n_monitors() as i64
}

fn get_primary_monitor_resolution() -> (i64, i64) {
    ensure_gdk_init();
    let display = gdk::Display::default().expect("No display found");
    if let Some(primary) = display.primary_monitor() {
        let rect = primary.geometry();
        (rect.width() as i64, rect.height() as i64)
    } else {
        (0, 0)
    }
}

fn get_all_monitor_resolutions() -> Vec<(i64, i64)> {
    ensure_gdk_init();
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
    ensure_gdk_init();
    let display = gdk::Display::default().expect("No display found");
    if let Some(m) = display.monitor(index as i32) {
        let geom = m.geometry();
        (geom.x() as i64, geom.y() as i64, geom.width() as i64, geom.height() as i64)
    } else {
        (0, 0, 0, 0)
    }
}

fn get_monitor_dpi(index: usize) -> f64 {
    ensure_gdk_init();
    let display = gdk::Display::default().expect("No display found");
    if let Some(m) = display.monitor(index as i32) {
        m.scale_factor() as f64 * 96.0 // base DPI * scale factor
    } else {
        0.0
    }
}
