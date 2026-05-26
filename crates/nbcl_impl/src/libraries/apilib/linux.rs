use nix::libc::{c_char, statvfs};
use nbcl::{error::Result, Value};
use std::ffi::CString;
use std::fs;
use std::path::Path;
use crate::runtime_err;

pub fn get_kernel_version(_args: Vec<Value>) -> Result<Value> {
    let output = fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|v| v.trim().to_string())
        .map_err(|e| runtime_err!("Failed to read kernel version: {}", e))?;

    Ok(Value::Str(output))
}

pub fn get_battery_perc(_args: Vec<Value>) -> Result<Value> {
    let power_path = Path::new("/sys/class/power_supply/");
    let batteries: Vec<_> = fs::read_dir(power_path)
        .map_err(|e| runtime_err!("Failed to read power_supply: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("BAT"))
        .collect();

    if batteries.is_empty() {
        return Err(runtime_err!("No batteries found"));
    }

    for bat in batteries {
        let capacity_path = bat.path().join("capacity");
        if let Ok(content) = fs::read_to_string(capacity_path) {
            if let Ok(percent) = content.trim().parse::<i64>() {
                return Ok(Value::Int(percent));
            }
        }
    }

    Err(runtime_err!("Failed to read battery percentage"))
}

pub fn get_cpu_info(_args: Vec<Value>) -> Result<Value> {
    let content = fs::read_to_string("/proc/cpuinfo")
        .map_err(|e| runtime_err!("Failed to read /proc/cpuinfo: {}", e))?;

    let mut cpus = Vec::new();
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
            let mut cpu_map = Vec::new();
            cpu_map.push(("core_id".into(), Value::Int(core_id)));
            cpu_map.push(("model".into(), Value::Str(model)));
            cpus.push(Value::Map(cpu_map));
            core_id += 1;
        }
    }

    Ok(Value::List(cpus))
}

pub fn get_ram_info(_args: Vec<Value>) -> Result<Value> {
    let mut map = Vec::new();
    let content = fs::read_to_string("/proc/meminfo")
        .map_err(|e| runtime_err!("Failed to read /proc/meminfo: {}", e))?;

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

    map.push(("total_kb".into(), Value::Int(total as i64)));
    map.push(("free_kb".into(), Value::Int(free as i64)));
    map.push(("available_kb".into(), Value::Int(available as i64)));
    map.push(("used_kb".into(), Value::Int((total - available) as i64)));

    Ok(Value::Map(map))
}

pub fn get_gpu_info(_args: Vec<Value>) -> Result<Value> {
    let mut gpus = Vec::new();
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
                let mut gpu = Vec::new();

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

                gpu.push(("sys_name".into(), Value::Str(name)));
                gpu.push(("model".into(), Value::Str(model)));
                gpu.push(("vendor".into(), Value::Str(vendor)));
                gpu.push(("memory_kb".into(), Value::Int(memory_kb as i64)));

                gpus.push(Value::Map(gpu));
            }
        }
    }

    Ok(Value::List(gpus))
}

pub fn get_disk_info(_args: Vec<Value>) -> Result<Value> {
    let mountpoint = "/";

    let mounts = fs::read_to_string("/proc/mounts")
        .map_err(|e| runtime_err!("Failed to read /proc/mounts: {}", e))?;

    let mut device = "unknown";
    for line in mounts.lines() {
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == mountpoint {
            device = parts[0];
            break;
        }
    }

    let c_path = CString::new(mountpoint).map_err(|e| runtime_err!("Invalid mountpoint: {}", e))?;
    let mut stat: statvfs = unsafe { std::mem::zeroed() };
    let res = unsafe { statvfs(c_path.as_ptr() as *const c_char, &mut stat) };
    if res != 0 {
        return Err(runtime_err!("Failed to statvfs for {}", mountpoint).into());
    }

    let total_kb = (stat.f_blocks * stat.f_frsize as u64) / 1024;
    let free_kb = (stat.f_bfree * stat.f_frsize as u64) / 1024;
    let used_kb = total_kb - free_kb;

    let mut info = Vec::new();
    info.push(("device".into(), Value::Str(device.into())));
    info.push(("total_kb".into(), Value::Int(total_kb as i64)));
    info.push(("used_kb".into(), Value::Int(used_kb as i64)));
    info.push(("free_kb".into(), Value::Int(free_kb as i64)));

    Ok(Value::Map(info))
}
