//! Linux API crate

use libc::{c_char, statvfs};
use rhai::{plugin::*, Array, Dynamic, EvalAltResult, Map};
use std::ffi::CString;
use std::fs;
use std::path::Path;

#[export_module]
pub mod linux {
    /// Get the installed Linux Kernel version.
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// The Kernel version as a `String`. (e.g. "6.16.4-arch1-1")
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::linux" as linux;
    ///
    /// let k_version = linux::get_kernel_version();
    /// ```
    #[rhai_fn(return_raw)]
    pub fn get_kernel_version() -> Result<String, Box<EvalAltResult>> {
        fs::read_to_string("/proc/sys/kernel/osrelease")
            .map(|v| v.trim().to_string())
            .map_err(|e| format!("Failed to read kernel version: {}", e).into())
    }

    /// Get the current battery percentage.
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// The battery percentage as an integer.
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::linux" as linux;
    ///
    /// let battery_perc = linux::get_battery_perc();
    /// ```
    #[rhai_fn(return_raw)]
    pub fn get_battery_perc() -> Result<i64, Box<EvalAltResult>> {
        let power_path = Path::new("/sys/class/power_supply/");
        let batteries: Vec<_> = fs::read_dir(power_path)
            .map_err(|e| format!("Failed to read power_supply: {}", e))?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("BAT"))
            .collect();

        if batteries.is_empty() {
            return Err("No batteries found".into());
        }

        for bat in batteries {
            let capacity_path = bat.path().join("capacity");
            if let Ok(content) = fs::read_to_string(capacity_path) {
                if let Ok(percent) = content.trim().parse::<i64>() {
                    return Ok(percent);
                }
            }
        }

        Err("Failed to read battery percentage".into())
    }

    /// Get the device CPU information.
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// An `Array` containing the CPU information in a `Map`:
    /// - `core_id` (Integer)
    /// - `model`
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::linux" as linux;
    ///
    /// let cpu_info = linux::get_cpu_info();
    /// let cpu_model = cpu_info[0].model; // get the model of the 1st core
    /// ```
    #[rhai_fn(return_raw)]
    pub fn get_cpu_info() -> Result<Array, Box<EvalAltResult>> {
        let content = fs::read_to_string("/proc/cpuinfo")
            .map_err(|e| format!("Failed to read /proc/cpuinfo: {}", e))?;

        let mut cpus = Array::new();
        let mut core_id = 0;

        for block in content.split("\n\n") {
            let mut model_name = None;
            for line in block.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim();
                    if key == "model name" {
                        model_name = Some(value.to_string());
                        break; // stop once we found the model in this block
                    }
                }
            }

            if let Some(model) = model_name {
                let mut cpu_map = Map::new();
                cpu_map.insert("core_id".into(), core_id.into());
                cpu_map.insert("model".into(), Dynamic::from(model));
                cpus.push(Dynamic::from(cpu_map));
                core_id += 1;
            }
        }

        Ok(cpus)
    }

    /// Get the device RAM information.
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// A `Map` containing the RAM information:
    /// - `total_kb`
    /// - `free_kb`
    /// - `available_kb`
    /// - `used_kb`
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::linux" as linux;
    ///
    /// let ram_info = linux::get_ram_info();
    /// let used_kb = ram_info.used_kb;
    /// ```
    #[rhai_fn(return_raw)]
    pub fn get_ram_info() -> Result<Map, Box<EvalAltResult>> {
        let mut map = Map::new();
        let content = fs::read_to_string("/proc/meminfo")
            .map_err(|e| format!("Failed to read /proc/meminfo: {}", e))?;

        let mut total = 0u64;
        let mut free = 0u64;
        let mut available = 0u64;

        for line in content.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let value = value
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .parse::<u64>()
                    .unwrap_or(0);
                match key.trim() {
                    "MemTotal" => total = value,
                    "MemFree" => free = value,
                    "MemAvailable" => available = value,
                    _ => {}
                }
            }
        }

        map.insert("total_kb".into(), Dynamic::from(total));
        map.insert("free_kb".into(), Dynamic::from(free));
        map.insert("available_kb".into(), Dynamic::from(available));
        map.insert("used_kb".into(), Dynamic::from(total - available));

        Ok(map)
    }

    /// Get the device GPU information.
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// An `Array` containing the GPU information in a `Map`:
    /// - `sys_name` (String): e.g., "card0"
    /// - `model` (String)
    /// - `vendor` (String)
    /// - `memory_kb` (Integer)
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::linux" as linux;
    ///
    /// let gpu_info = linux::get_gpu_info();
    /// let model = gpu_info[0].model; // 1st GPU card model
    /// ```
    #[rhai_fn(return_raw)]
    pub fn get_gpu_info() -> Result<Array, Box<EvalAltResult>> {
        let mut gpus = Array::new();
        let drm_path = Path::new("/sys/class/drm/");

        if drm_path.exists() {
            if let Ok(entries) = fs::read_dir(drm_path) {
                for e in entries.filter_map(|e| e.ok()) {
                    let name = e.file_name().to_string_lossy().into_owned();

                    // Only consider main GPU cards (skip connectors like card0-HDMI-A-1)
                    if !name.starts_with("card") || name.contains('-') {
                        continue;
                    }

                    let device_path = e.path().join("device");
                    let mut gpu = Map::new();

                    // vendor ID
                    let vendor = fs::read_to_string(device_path.join("vendor"))
                        .unwrap_or_else(|_| "unknown".to_string())
                        .trim()
                        .to_string();

                    // device/model ID
                    let model = fs::read_to_string(device_path.join("device"))
                        .unwrap_or_else(|_| name.clone())
                        .trim()
                        .to_string();

                    // VRAM (in KB)
                    let memory_kb = fs::read_to_string(device_path.join("mem_info_vram_total"))
                        .ok()
                        .and_then(|s| s.trim().parse::<u64>().ok())
                        .unwrap_or(0);

                    gpu.insert("sys_name".into(), Dynamic::from(name));
                    gpu.insert("model".into(), Dynamic::from(model));
                    gpu.insert("vendor".into(), Dynamic::from(vendor));
                    gpu.insert("memory_kb".into(), Dynamic::from(memory_kb));

                    gpus.push(Dynamic::from(gpu));
                }
            }
        }

        Ok(gpus)
    }

    /// Get the device disk information.
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// A `Map` containing the disk information:
    /// - Each key is a `mountpoint`, value is a map with:
    ///     - `device` (String)
    ///     - `total_kb` (Integer)
    ///     - `used_kb` (Integer)
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::linux" as linux;
    ///
    /// let disk_info = linux::get_disk_info();
    /// let device = disk_info.device;
    /// let total_kb = disk_info.total_kb;
    /// ```
    #[rhai_fn(return_raw)]
    pub fn get_disk_info() -> Result<Map, Box<EvalAltResult>> {
        let mountpoint = "/";

        let mounts = fs::read_to_string("/proc/mounts")
            .map_err(|e| format!("Failed to read /proc/mounts: {}", e))?;

        let mut device = "unknown";
        for line in mounts.lines() {
            let parts: Vec<_> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == mountpoint {
                device = parts[0];
                break;
            }
        }

        let c_path = CString::new(mountpoint).map_err(|e| format!("Invalid mountpoint: {}", e))?;
        let mut stat: statvfs = unsafe { std::mem::zeroed() };
        let res = unsafe { statvfs(c_path.as_ptr() as *const c_char, &mut stat) };
        if res != 0 {
            return Err(format!("Failed to statvfs for {}", mountpoint).into());
        }

        let total_kb = (stat.f_blocks * stat.f_frsize as u64) / 1024;
        let free_kb = (stat.f_bfree * stat.f_frsize as u64) / 1024;
        let used_kb = total_kb - free_kb;

        let mut info = Map::new();
        info.insert("device".into(), device.into());
        info.insert("total_kb".into(), Dynamic::from(total_kb));
        info.insert("used_kb".into(), Dynamic::from(used_kb));
        info.insert("free_kb".into(), Dynamic::from(free_kb));

        Ok(info)
    }
}
