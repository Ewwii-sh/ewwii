//! MacOS wifi api is Untested

use rhai::{Array, Dynamic, EvalAltResult, Map, plugin::*};
use std::process::Command;

#[export_module]
pub mod wifi {
    use super::*;

    #[cfg(target_os = "linux")]
    #[rhai_fn(return_raw)]
    fn scan_linux() -> Result<Array, Box<EvalAltResult>> {
        let output = Command::new("nmcli")
            .args(&["-t", "-f", "SSID,SIGNAL,SECURITY", "dev", "wifi"])
            .output()
            .map_err(|e| format!("Failed to run nmcli: {e}"))?;

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 output from nmcli: {e}"))?;

        let mut result = Array::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() != 3 { continue; }
            let mut map = Map::new();
            map.insert("ssid".into(), parts[0].into());
            map.insert("signal".into(), parts[1].into());
            map.insert("security".into(), parts[2].into());
            result.push(Dynamic::from(map));
        }
        Ok(result)
    }

    #[cfg(target_os = "macos")]
    #[rhai_fn(return_raw)]
    fn scan_macos() -> Result<Array, Box<EvalAltResult>> {
        let output = Command::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport")
            .arg("-s")
            .output()
            .map_err(|e| format!("Failed to run airport: {e}"))?;

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 output from airport: {e}"))?;

        let mut result = Array::new();
        for line in stdout.lines().skip(1) {
            let ssid = line.get(0..32).unwrap_or("").trim().to_string();
            let rssi = line.get(33..36).unwrap_or("").trim().to_string();
            let security = line.get(61..).unwrap_or("").trim().to_string();

            let mut map = Map::new();
            map.insert("ssid".into(), ssid.into());
            map.insert("signal".into(), rssi.into());
            map.insert("security".into(), security.into());
            result.push(Dynamic::from(map));
        }
        Ok(result)
    }

    #[rhai_fn(return_raw)]
    pub fn scan() -> Result<Array, Box<EvalAltResult>> {
        #[cfg(target_os = "linux")]
        { scan_linux() }

        #[cfg(target_os = "macos")]
        { scan_macos() }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        { Err("wifi::scan not supported on this OS".into()) }
    }

    #[rhai_fn(return_raw)]
    pub fn current_connection() -> Result<Map, Box<EvalAltResult>> {
        #[cfg(target_os = "linux")]
        { current_connection_linux() }

        #[cfg(target_os = "macos")]
        { current_connection_macos() }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        { Err("wifi::current_connection not supported on this OS".into()) }
    }

    #[cfg(target_os = "linux")]
    #[rhai_fn(return_raw)]
    pub fn current_connection_linux() -> Result<Map, Box<EvalAltResult>> {
        let output = Command::new("nmcli")
            .args(&["-t", "-f", "ACTIVE,SSID,SIGNAL,SECURITY", "device", "wifi", "list"])
            .output()
            .map_err(|e| format!("Failed to run nmcli: {e}"))?;
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 output: {}", e))?;
        let mut map = Map::new();
        if let Some(line) = stdout.lines().find(|l| l.starts_with("yes:")) {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 4 {
                map.insert("ssid".into(), parts[1].into());
                map.insert("signal".into(), parts[2].into());
                map.insert("security".into(), parts[3].into());
            }
        }
        Ok(map)
    }

    #[cfg(target_os = "macos")]
    #[rhai_fn(return_raw)]
    pub fn current_connection_macos() -> Result<Map, Box<EvalAltResult>> {
        let output = Command::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport")
            .args(&["-I"])
            .output()
            .map_err(|e| format!("Failed to run airport: {e}"))?;

        let stdout = String::from_utf8(output.stdout)?;
        let mut map = Map::new();

        for line in stdout.lines() {
            if let Some(idx) = line.find(':') {
                let key = line[..idx].trim().to_lowercase();
                let value = line[idx + 1..].trim().to_string();
                map.insert(key.into(), value.into());
            }
        }

        Ok(map)
    }

    #[rhai_fn(return_raw)]
    pub fn connect(ssid: &str, password: Option<&str>) -> Result<(), Box<EvalAltResult>> {
        #[cfg(target_os = "linux")]
        {
            let mut args = vec!["dev", "wifi", "connect", ssid];
            if let Some(pw) = password {
                args.push("password");
                args.push(pw);
            }
            let status = Command::new("nmcli").args(&args).status()
                .map_err(|e| format!("Failed to run nmcli: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("Failed to connect to {}", ssid).into())
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(pw) = password {
                let status = Command::new("networksetup")
                    .args(&["-setairportnetwork", "en0", ssid, pw])
                    .status()
                    .map_err(|e| format!("Failed to run networksetup: {e}"))?;
                if status.success() {
                    Ok(())
                } else {
                    Err(format!("Failed to connect to {}", ssid).into())
                }
            } else {
                // No password
                let status = Command::new("networksetup")
                    .args(&["-setairportnetwork", "en0", ssid])
                    .status()
                    .map_err(|e| format!("Failed to run networksetup: {e}"))?;
                if status.success() {
                    Ok(())
                } else {
                    Err(format!("Failed to connect to {}", ssid).into())
                }
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err("wifi::connect not supported on this OS".into())
        }
    }

    #[rhai_fn(return_raw)]
    pub fn disconnect() -> Result<(), Box<EvalAltResult>> {
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;

            // Get current active SSID
            let output = Command::new("nmcli")
                .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
                .output()
                .map_err(|e| format!("Failed to run nmcli: {e}"))?;

            let stdout = String::from_utf8(output.stdout)
                .map_err(|e| format!("Invalid UTF-8 from nmcli: {e}"))?;

            let ssid = stdout
                .lines()
                .find(|line| line.starts_with("yes:"))
                .and_then(|line| line.split(':').nth(1))
                .ok_or("No active Wi-Fi connection")?;

            let status = Command::new("nmcli")
                .args(&["connection", "down", "id", ssid])
                .status()
                .map_err(|e| format!("Failed to disconnect from {}: {e}", ssid))?;

            if status.success() {
                Ok(())
            } else {
                Err(format!("Failed to disconnect from {}", ssid).into())
            }
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;

            let status = Command::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport")
                .arg("-z")
                .status()
                .map_err(|e| format!("Failed to run airport: {e}"))?;

            if status.success() {
                Ok(())
            } else {
                Err("Failed to disconnect from current Wi-Fi".into())
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err("wifi::disconnect not supported on this OS".into())
        }
    }


    #[rhai_fn(return_raw)]
    pub fn disable_adapter() -> Result<(), Box<EvalAltResult>> {
        #[cfg(target_os = "linux")]
        {
            let status = Command::new("nmcli")
                .args(&["networking", "off"])
                .status()
                .map_err(|e| format!("Failed to run nmcli: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err("Failed to disconnect".into())
            }
        }

        #[cfg(target_os = "macos")]
        {
            let status = Command::new("networksetup")
                .args(&["-setairportpower", "en0", "off"])
                .status()
                .map_err(|e| format!("Failed to run networksetup: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err("Failed to disconnect".into())
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err("wifi::disconnect not supported on this OS".into())
        }
    }

    #[rhai_fn(return_raw)]
    pub fn enable_adapter() -> Result<(), Box<EvalAltResult>> {
        #[cfg(target_os = "linux")]
        {
            let status = Command::new("nmcli")
                .args(&["networking", "on"])
                .status()
                .map_err(|e| format!("Failed to run nmcli: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err("Failed to enable".into())
            }
        }

        #[cfg(target_os = "macos")]
        {
            let status = Command::new("networksetup")
                .args(&["-setairportpower", "en0", "on"])
                .status()
                .map_err(|e| format!("Failed to run networksetup: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err("Failed to enable".into())
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err("wifi::enable not supported on this OS".into())
        }
    }
}
