use gtk::gdk;
use gtk::prelude::*;
use rhai::plugin::*;

#[export_module]
pub mod monitor {
    /// Get the number of connected monitors.
    ///
    /// # Returns
    ///
    /// Returns the total number of connected monitors as an `i64`.
    ///
    /// # Example
    ///
    /// ```js
    /// import "std::monitor" as monitor;
    ///
    /// let count = monitor::count();
    /// print(count); // Output: Number of connected monitors
    /// ```
    pub fn count() -> i64 {
        get_monitor_count()
    }

    /// Get the resolution of the primary monitor.
    ///
    /// # Returns
    ///
    /// Returns an array containing the width and height of the primary monitor as two `i64` values.
    ///
    /// # Example
    ///
    /// ```js
    /// import "std::monitor" as monitor;
    ///
    /// let resolution = monitor::primary_resolution();
    /// print(resolution); // Output: [width, height]
    /// ```
    pub fn primary_resolution() -> [i64; 2] {
        let (w, h) = get_primary_monitor_resolution();
        [w, h]
    }

    /// Get the resolution of the primary monitor as a string.
    ///
    /// # Returns
    ///
    /// Returns the resolution of the primary monitor as a string in the format "width x height".
    ///
    /// # Example
    ///
    /// ```js
    /// import "std::monitor" as monitor;
    ///
    /// let resolution_str = monitor::primary_resolution_str();
    /// print(resolution_str); // Output: "1920x1080"
    /// ```
    pub fn primary_resolution_str() -> String {
        let (w, h) = get_primary_monitor_resolution();
        format!("{w}x{h}")
    }

    /// Get the resolutions of all connected monitors.
    ///
    /// # Returns
    ///
    /// Returns an array of arrays, where each inner array contains the width and height of a monitor.
    ///
    /// # Example
    ///
    /// ```js
    /// import "std::monitor" as monitor;
    ///
    /// let resolutions = monitor::all_resolutions();
    /// print(resolutions); // Output: [[width1, height1], [width2, height2], ...]
    /// ```
    pub fn all_resolutions() -> Vec<[i64; 2]> {
        get_all_monitor_resolutions()
            .into_iter()
            .map(|(w, h)| [w, h])
            .collect()
    }

    /// Get the resolutions of all connected monitors as a string.
    ///
    /// # Returns
    ///
    /// Returns a string where each monitor's resolution is formatted as "width x height", separated by commas.
    ///
    /// # Example
    ///
    /// ```js
    /// import "std::monitor" as monitor;
    ///
    /// let resolutions_str = monitor::all_resolutions_str();
    /// print(resolutions_str); // Output: "1920x1080, 1280x720"
    /// ```
    pub fn all_resolutions_str() -> String {
        get_all_monitor_resolutions()
            .into_iter()
            .map(|(w, h)| format!("{w}x{h}"))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Get the dimensions (x, y, width, height) of a specific monitor.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the monitor (0-based).
    ///
    /// # Returns
    ///
    /// Returns an array with the monitor's position (x, y) and size (width, height).
    ///
    /// # Example
    ///
    /// ```js
    /// import "std::monitor" as monitor;
    ///
    /// let dimensions = monitor::dimensions(0);
    /// print(dimensions); // Output: [x, y, width, height]
    /// ```
    pub fn dimensions(index: i64) -> [i64; 4] {
        let (x, y, w, h) = get_monitor_dimensions(index as usize);
        [x, y, w, h]
    }

    /// Get the dimensions of a specific monitor as a string.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the monitor (0-based).
    ///
    /// # Returns
    ///
    /// Returns the monitor's dimensions as a string in the format "x,y - width x height".
    ///
    /// # Example
    ///
    /// ```js
    /// import "std::monitor" as monitor;
    ///
    /// let dimensions_str = monitor::dimensions_str(0);
    /// print(dimensions_str); // Output: "0,0 - 1920x1080"
    /// ```
    pub fn dimensions_str(index: i64) -> String {
        let (x, y, w, h) = get_monitor_dimensions(index as usize);
        format!("{x},{y} - {w}x{h}")
    }

    /// Get the DPI (dots per inch) of a specific monitor.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the monitor (0-based).
    ///
    /// # Returns
    ///
    /// Returns the DPI (scale factor * base DPI) of the monitor as a `f64`.
    ///
    /// # Example
    ///
    /// ```js
    /// import "std::monitor" as monitor;
    ///
    /// let dpi = monitor::dpi(0);
    /// print(dpi); // Output: DPI of the monitor
    /// ```
    pub fn dpi(index: i64) -> f64 {
        get_monitor_dpi(index as usize)
    }

    /// Get the DPI of a specific monitor as a string.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the monitor (0-based).
    ///
    /// # Returns
    ///
    /// Returns the DPI of the monitor as a string formatted to 1 decimal place.
    ///
    /// # Example
    ///
    /// ```js
    /// import "std::monitor" as monitor;
    ///
    /// let dpi_str = monitor::dpi_str(0);
    /// print(dpi_str); // Output: "96.0"
    /// ```
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
