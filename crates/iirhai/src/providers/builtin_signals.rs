// use super::helper::IterAverage;
// use anyhow::{Context, Result};
// use once_cell::sync::Lazy;
// use std::{fs::read_to_string, sync::Mutex};
// use sysinfo::System;

// pub fn register_all_signals(engine: &mut rhai::Engine) {
//     // struct: { <name>: temperature }
//     engine.register_fn("EWWII_TEMPS", get_temperatures);
//     // struct: { total_mem, free_mem, total_swap, free_swap, available_mem, used_mem, used_mem_perc }
//     engine.register_fn("EWWII_RAM", get_ram);
//     // struct: { <mount_point>: { name, total, free, used, used_perc } }
//     engine.register_fn("EWWII_DISK", get_disks);
//     // struct: { <name>: { capacity, status } }
//     engine.register_fn("EWWII_BATTERY", get_battery_wrapper);
//     // struct: { cores: [{ core, freq, usage }], avg }
//     engine.register_fn("EWWII_CPU", get_cpus);
//     // struct: { <name>: { up, down } }
//     engine.register_fn("EWWII_NET", get_net);
//     // the current UNIX timestamp
//     engine.register_fn("EWWII_TIME", get_time);
//     // TODO maybe implement this later once we force creating widgets from file?
//     // engine.register_fn("EWWII_CONFIG_DIR");
//     // engine.register_fn("EWWII_CMD", get_ewwii_executable_cmd);
//     engine.register_fn("EWWII_EXECUTABLE", get_ewwii_executable);
// }

// fn get_ewwii_executable() -> String {
//     std::env::current_exe().map(|x| x.to_string_lossy().into_owned()).unwrap_or_else(|_| "ewwii".to_string())
// }

// // fn get_ewwii_executable_cmd() -> String {
// //     format!(
// //         "\"{}\" --config \"{}\"",
// //         std::env::current_exe().map(|x| x.to_string_lossy().into_owned()).unwrap_or_else(|_| "ewwii".to_string()),
// //         eww_paths.get_config_dir().to_string_lossy().into_owned()
// //     )
// // }

// struct RefreshTime(std::time::Instant);
// impl RefreshTime {
//     pub fn new() -> Self {
//         Self(std::time::Instant::now())
//     }

//     pub fn next_refresh(&mut self) -> std::time::Duration {
//         let now = std::time::Instant::now();
//         let duration = now.duration_since(self.0);
//         self.0 = now;
//         duration
//     }
// }

// static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));
// static DISKS: Lazy<Mutex<sysinfo::Disks>> = Lazy::new(|| Mutex::new(sysinfo::Disks::new_with_refreshed_list()));
// static COMPONENTS: Lazy<Mutex<sysinfo::Components>> = Lazy::new(|| Mutex::new(sysinfo::Components::new_with_refreshed_list()));
// static NETWORKS: Lazy<Mutex<(RefreshTime, sysinfo::Networks)>> =
//     Lazy::new(|| Mutex::new((RefreshTime::new(), sysinfo::Networks::new_with_refreshed_list())));

// pub fn get_disks() -> rhai::Map {
//     let mut disks = DISKS.lock().unwrap();
//     disks.refresh_list();
//     disks.refresh();

//     let mut map = rhai::Map::new();

//     for disk in disks.iter() {
//         let total_space = disk.total_space();
//         let avail_space = disk.available_space();
//         let used_space = total_space - avail_space;

//         let mut disk_map = rhai::Map::new();
//         disk_map.insert("name".into(), disk.name().to_string().into());
//         disk_map.insert("total".into(), total_space.into());
//         disk_map.insert("free".into(), avail_space.into());
//         disk_map.insert("used".into(), used_space.into());
//         disk_map.insert("used_perc".into(), ((used_space as f32 / total_space as f32) * 100.0).into());

//         map.insert(disk.mount_point().display().to_string().into(), disk_map.into());
//     }

//     map
// }

// pub fn get_ram() -> rhai::Map {
//     let mut system = SYSTEM.lock().unwrap();
//     system.refresh_memory();

//     let mut map = rhai::Map::new();
//     let total_memory = system.total_memory();
//     let available_memory = system.available_memory();
//     let used_memory = total_memory as f32 - available_memory as f32;

//     map.insert("total_mem".into(), total_memory.into());
//     map.insert("free_mem".into(), system.free_memory().into());
//     map.insert("total_swap".into(), system.total_swap().into());
//     map.insert("free_swap".into(), system.free_swap().into());
//     map.insert("available_mem".into(), available_memory.into());
//     map.insert("used_mem".into(), used_memory.into());
//     map.insert("used_mem_perc".into(), ((used_memory / total_memory as f64) * 100.0).into());

//     map
// }

// pub fn get_temperatures() -> String {
//     let mut components = COMPONENTS.lock().unwrap();
//     components.refresh_list();
//     components.refresh();

//     let mut map = rhai::Map::new();

//     for component in components.iter() {
//         let temp = component.temperature();
//         if temp.is_finite() {
//             map.insert(
//                 component.label().to_uppercase().replace(' ', "_").into(),
//                 temp.into(),
//             );
//         }
//     }

//     map
// }

// pub fn get_cpus() -> rhai::Map {
//     let mut system = SYSTEM.lock().unwrap();
//     system.refresh_cpu_specifics(sysinfo::CpuRefreshKind::everything());

//     let cpus = system.cpus();
//     let mut cores = rhai::Array::new();

//     for cpu in cpus {
//         let mut core_map = rhai::Map::new();
//         core_map.insert("core".into(), cpu.name().into());
//         core_map.insert("freq".into(), cpu.frequency().into());
//         core_map.insert("usage".into(), (cpu.cpu_usage() as i64).into());
//         cores.push(core_map.into());
//     }

//     let mut result_map = rhai::Map::new();
//     result_map.insert("cores".into(), cores.into());
//     result_map.insert("avg".into(), cpus.iter().map(|a| a.cpu_usage()).avg().into());

//     result_map
// }

// fn get_battery_wrapper() -> String {
//     get_battery_capacity().unwrap_or_else(|_| "{}".to_string())
// }

// #[cfg(target_os = "macos")]
// pub fn get_battery_capacity() -> Result<String> {
//     let capacity = String::from_utf8(
//         std::process::Command::new("pmset")
//             .args(&["-g", "batt"])
//             .output()
//             .context("\nError while getting the battery value on macos, with `pmset`: ")?
//             .stdout,
//     )?;

//     // Example output of that command:
//     // Now drawing from 'Battery Power'
//     //-InternalBattery-0 (id=11403363)	100%; discharging; (no estimate) present: true
//     let regex = regex!(r"[0-9]*%");
//     let mut number = regex.captures(&capacity).unwrap().get(0).unwrap().as_str().to_string();

//     // Removes the % at the end
//     number.pop();
//     Ok(format!(
//         "{{ \"BAT0\": {{ \"capacity\": \"{}\", \"status\": \"{}\" }}}}",
//         number,
//         capacity.split(";").collect::<Vec<&str>>()[1]
//     ))
// }

// #[cfg(target_os = "linux")]
// pub fn get_battery_capacity() -> Result<String> {
//     use std::{collections::HashMap, sync::atomic::AtomicBool};

//     #[derive(serde::Serialize)]
//     struct BatteryData {
//         capacity: i64,
//         status: String,
//     }

//     #[derive(serde::Serialize)]
//     struct Data {
//         #[serde(flatten)]
//         batteries: HashMap<String, BatteryData>,
//         total_avg: f64,
//     }

//     let mut current = 0_f64;
//     let mut total = 0_f64;
//     let mut batteries = HashMap::new();
//     let power_supply_dir = std::path::Path::new("/sys/class/power_supply");
//     let power_supply_entries = power_supply_dir.read_dir().context("Couldn't read /sys/class/power_supply directory")?;
//     for entry in power_supply_entries {
//         let entry = entry?.path();
//         if !entry.is_dir() {
//             continue;
//         }
//         if let (Ok(capacity), Ok(status)) = (read_to_string(entry.join("capacity")), read_to_string(entry.join("status"))) {
//             batteries.insert(
//                 entry.file_name().context("Couldn't get filename")?.to_string_lossy().to_string(),
//                 BatteryData {
//                     status: status.trim_end_matches('\n').to_string(),
//                     capacity: capacity.trim_end_matches('\n').parse::<f64>()?.round() as i64,
//                 },
//             );
//             if let (Ok(charge_full), Ok(charge_now), Ok(voltage_now)) = (
//                 read_to_string(entry.join("charge_full")),
//                 read_to_string(entry.join("charge_now")),
//                 read_to_string(entry.join("voltage_now")),
//             ) {
//                 // (uAh / 1000000) * U = p and that / one million so that we have microwatt
//                 current += ((charge_now.trim_end_matches('\n').parse::<f64>()? / 1000000_f64)
//                     * voltage_now.trim_end_matches('\n').parse::<f64>()?)
//                     / 1000000_f64;
//                 total += ((charge_full.trim_end_matches('\n').parse::<f64>()? / 1000000_f64)
//                     * voltage_now.trim_end_matches('\n').parse::<f64>()?)
//                     / 1000000_f64;
//             } else if let (Ok(energy_full), Ok(energy_now)) =
//                 (read_to_string(entry.join("energy_full")), read_to_string(entry.join("energy_now")))
//             {
//                 current += energy_now.trim_end_matches('\n').parse::<f64>()?;
//                 total += energy_full.trim_end_matches('\n').parse::<f64>()?;
//             } else {
//                 static WARNED: AtomicBool = AtomicBool::new(false);
//                 if !WARNED.load(std::sync::atomic::Ordering::Relaxed) {
//                     WARNED.store(true, std::sync::atomic::Ordering::Relaxed);
//                     log::warn!(
//                         "Failed to get/calculate uWh: the total_avg value of the battery magic var will probably be a garbage \
//                          value that can not be trusted."
//                     );
//                 }
//             }
//         }
//     }
//     if total == 0_f64 {
//         return Ok(String::from(""));
//     }

//     Ok(serde_json::to_string(&(Data { batteries, total_avg: (current / total) * 100_f64 })).unwrap())
// }

// #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "openbsd"))]
// pub fn get_battery_capacity() -> Result<String> {
//     let batteries = String::from_utf8(
//         // I have only tested `apm` on FreeBSD, but it *should* work on all of the listed targets,
//         // based on what I can tell from their online man pages.
//         std::process::Command::new("apm")
//             .output()
//             .context("\nError while getting the battery values on bsd, with `apm`: ")?
//             .stdout,
//     )?;

//     // `apm` output should look something like this:
//     // $ apm
//     // ...
//     // Remaining battery life: 87%
//     // Remaining battery time: unknown
//     // Number of batteries: 1
//     // Battery 0
//     //         Battery Status: charging
//     //         Remaining battery life: 87%
//     //         Remaining battery time: unknown
//     // ...
//     // last 4 lines are repeated for each battery.
//     // see also:
//     // https://www.freebsd.org/cgi/man.cgi?query=apm&manpath=FreeBSD+13.1-RELEASE+and+Ports
//     // https://man.openbsd.org/amd64/apm.8
//     // https://man.netbsd.org/apm.8
//     let mut json = String::from('{');
//     let re_total = regex!(r"(?m)^Remaining battery life: (\d+)%");
//     let re_single = regex!(r"(?sm)^Battery (\d+):.*?Status: (\w+).*?(\d+)%");
//     for bat in re_single.captures_iter(&batteries) {
//         json.push_str(&format!(
//             r#""BAT{}": {{ "status": "{}", "capacity": {} }}, "#,
//             bat.get(1).unwrap().as_str(),
//             bat.get(2).unwrap().as_str(),
//             bat.get(3).unwrap().as_str(),
//         ))
//     }

//     json.push_str(&format!(r#""total_avg": {}}}"#, re_total.captures(&batteries).unwrap().get(1).unwrap().as_str()));
//     Ok(json)
// }

// #[cfg(not(target_os = "macos"))]
// #[cfg(not(target_os = "linux"))]
// #[cfg(not(target_os = "netbsd"))]
// #[cfg(not(target_os = "freebsd"))]
// #[cfg(not(target_os = "openbsd"))]
// pub fn get_battery_capacity() -> Result<String> {
//     Err(anyhow::anyhow!("Eww doesn't support your OS for getting the battery capacity"))
// }

// pub fn get_net() -> String {
//     let (ref mut last_refresh, ref mut networks) = &mut *NETWORKS.lock().unwrap();

//     networks.refresh_list();
//     let elapsed = last_refresh.next_refresh();

//     networks
//         .iter()
//         .map(|(name, data)| {
//             let transmitted = data.transmitted() as f64 / elapsed.as_secs_f64();
//             let received = data.received() as f64 / elapsed.as_secs_f64();
//             (name, serde_json::json!({ "NET_UP": transmitted, "NET_DOWN": received }))
//         })
//         .collect::<serde_json::Value>()
//         .to_string()
// }

// pub fn get_time() -> String {
//     chrono::offset::Utc::now().timestamp().to_string()
// }

pub fn register_all_signals(_resolver: &mut rhai::module_resolvers::StaticModuleResolver) {
    // TODO
}
