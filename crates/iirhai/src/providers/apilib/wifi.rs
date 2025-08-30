//! MacOS Wi-Fi API is untested.

use rhai::{plugin::*, Array, Dynamic, EvalAltResult, Map};
use std::process::Command;

#[export_module]
pub mod wifi {
    use super::*;

    /// Scans for all available Wi-Fi connections on Linux.
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// An `Array` containing information about each Wi-Fi connection, where each entry is a `Map`
    /// with keys "ssid", "signal", and "security" representing the Wi-Fi network's SSID, signal strength,
    /// and security type.
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::wifi" as wifi;
    ///
    /// let networks = wifi::scan();
    /// ```
    #[cfg(target_os = "linux")]
    #[rhai_fn(return_raw)]
    pub fn scan_linux() -> Result<Array, Box<EvalAltResult>> {
        let output = Command::new("nmcli")
            .args(&["-t", "-f", "SSID,SIGNAL,SECURITY", "dev", "wifi"])
            .output()
            .map_err(|e| format!("Failed to run nmcli: {e}"))?;

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 output from nmcli: {e}"))?;

        let mut result = Array::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() != 3 {
                continue;
            }
            let mut map = Map::new();
            map.insert("ssid".into(), parts[0].into());
            map.insert("signal".into(), parts[1].into());
            map.insert("security".into(), parts[2].into());
            result.push(Dynamic::from(map));
        }
        Ok(result)
    }

    /// Scans for all available Wi-Fi connections on macOS.
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// An `Array` containing information about each Wi-Fi connection, where each entry is a `Map`
    /// with keys "ssid", "signal", and "security" representing the Wi-Fi network's SSID, signal strength,
    /// and security type.
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::wifi" as wifi;
    ///
    /// let networks = wifi::scan();
    /// ```
    #[cfg(target_os = "macos")]
    #[rhai_fn(return_raw)]
    pub fn scan_macos() -> Result<Array, Box<EvalAltResult>> {
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

    /// Scans for all available Wi-Fi connections, platform-dependent (Linux or macOS).
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// An `Array` containing information about each Wi-Fi connection, where each entry is a `Map`
    /// with keys "ssid", "signal", and "security" representing the Wi-Fi network's SSID, signal strength,
    /// and security type.
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::wifi" as wifi;
    ///
    /// let networks = wifi::scan();
    /// ```
    #[rhai_fn(return_raw)]
    pub fn scan() -> Result<Array, Box<EvalAltResult>> {
        #[cfg(target_os = "linux")]
        {
            scan_linux()
        }

        #[cfg(target_os = "macos")]
        {
            scan_macos()
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err("wifi::scan not supported on this OS".into())
        }
    }

    /// Retrieves the current active Wi-Fi connection's details (SSID, signal, and security).
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// A `Map` containing the current connection's SSID, signal strength, and security type.
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::wifi" as wifi;
    ///
    /// let connection = wifi::current_connection();
    /// ```
    #[rhai_fn(return_raw)]
    pub fn current_connection() -> Result<Map, Box<EvalAltResult>> {
        #[cfg(target_os = "linux")]
        {
            current_connection_linux()
        }

        #[cfg(target_os = "macos")]
        {
            current_connection_macos()
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err("wifi::current_connection not supported on this OS".into())
        }
    }

    #[cfg(target_os = "linux")]
    #[rhai_fn(return_raw)]
    pub fn current_connection_linux() -> Result<Map, Box<EvalAltResult>> {
        let output = Command::new("nmcli")
            .args(&["-t", "-f", "ACTIVE,SSID,SIGNAL,SECURITY", "device", "wifi", "list"])
            .output()
            .map_err(|e| format!("Failed to run nmcli: {e}"))?;
        let stdout =
            String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 output: {}", e))?;
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

    /// Connects to a Wi-Fi network with the specified SSID and password.
    ///
    /// # Arguments
    ///
    /// * `ssid` - The SSID of the Wi-Fi network.
    /// * `password` - The password of the Wi-Fi network (optional for open networks).
    ///
    /// # Returns
    ///
    /// Returns nothing if the connection is successful, or an error message if it fails.
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::wifi" as wifi;
    ///
    /// wifi::connect("MySecretNetwork", "password123");
    /// ```
    #[rhai_fn(return_raw)]
    pub fn connect(ssid: &str, password: &str) -> Result<(), Box<EvalAltResult>> {
        #[cfg(target_os = "linux")]
        {
            let args = vec!["dev", "wifi", "connect", ssid, "password", password];
            let status = Command::new("nmcli")
                .args(&args)
                .status()
                .map_err(|e| format!("Failed to run nmcli: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("Failed to connect to {}", ssid).into())
            }
        }

        #[cfg(target_os = "macos")]
        {
            let status = Command::new("networksetup")
                .args(&["-setairportnetwork", "en0", ssid, password])
                .status()
                .map_err(|e| format!("Failed to run networksetup: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("Failed to connect to {}", ssid).into())
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err("wifi::connect not supported on this OS".into())
        }
    }

    /// Connects to a Wi-Fi network with the specified SSID using saved profile (no password required).
    ///
    /// # Arguments
    ///
    /// * `ssid` - The SSID of the Wi-Fi network.
    ///
    /// # Returns
    ///
    /// Returns nothing if the connection is successful, or an error message if it fails.
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::wifi" as wifi;
    ///
    /// wifi::connect_without_password("MySecretNetwork", "password123");
    /// ```
    #[rhai_fn(return_raw)]
    pub fn connect_without_password(ssid: &str) -> Result<(), Box<EvalAltResult>> {
        #[cfg(target_os = "linux")]
        {
            let args = vec!["dev", "wifi", "connect", ssid];
            let status = Command::new("nmcli")
                .args(&args)
                .status()
                .map_err(|e| format!("Failed to run nmcli: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("Failed to connect to {}", ssid).into())
            }
        }

        #[cfg(target_os = "macos")]
        {
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

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err("wifi::connect not supported on this OS".into())
        }
    }

    /// Disconnects from the current Wi-Fi network.
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// Returns nothing if the connection is successful, or an error message if it fails.
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::wifi" as wifi;
    ///
    /// wifi::disconnect();
    /// ```
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

            let status =
                Command::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport")
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
    
    /// Disables the Wi-Fi adapter.
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// Returns nothing if the connection is successful, or an error message if it fails.
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::wifi" as wifi;
    ///
    /// wifi::disable_adapter();
    /// ```
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
                Err("Failed to disable adapter".into())
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
                Err("Failed to disable adapter".into())
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err("wifi::disable_adapter not supported on this OS".into())
        }
    }

    /// Enables the Wi-Fi adapter.
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// Returns nothing if the connection is successful, or an error message if it fails.
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::wifi" as wifi;
    ///
    /// wifi::enable_adapter();
    /// ```
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
            Err("wifi::enable_adapter not supported on this OS".into())
        }
    }

    /// Get the currenet state of adapter.
    ///
    /// # Arguments
    ///
    /// This function does not require any arguments.
    ///
    /// # Returns
    ///
    /// Returns the state of the adapter as a `string` and returns an error if getting the state failed.
    ///
    /// **Possible returns in Linux:** 
    ///
    /// - `"full"` (internet available)
    /// - `"limited"` (network only, no internet)
    /// - `"portal"` (captive portal)
    /// - `"none"` (no connectivity)
    ///
    /// **Possible returns in macOS:** 
    /// 
    /// - `"full"` (connected to a Wi-Fi network)
    /// - `"none"` (not connected)
    ///
    /// # Example
    ///
    /// ```js
    /// import "api::wifi" as wifi;
    ///
    /// wifi::enable_adapter();
    /// ```
    #[rhai_fn(return_raw)]
    pub fn get_adapter_connectivity() -> Result<String, Box<EvalAltResult>> {
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("nmcli")
                .args(&["networking", "connectivity"])
                .output()
                .map_err(|e| format!("Failed to run nmcli: {e}"))?;

            if output.status.success() {
                let connectivity = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(connectivity)
            } else {
                Err("Failed to get connectivity".into())
            }
        }

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("networksetup")
                .args(&["-getairportnetwork", "en0"])
                .output()
                .map_err(|e| format!("Failed to run networksetup: {e}"))?;

            if output.status.success() {
                let network_info = String::from_utf8_lossy(&output.stdout).trim().to_string();

                // Normalize to "full" or "none"
                if network_info.contains("You are not associated") {
                    Ok("none".to_string())
                } else {
                    Ok("full".to_string())
                }
            } else {
                Err("Failed to get connectivity".into())
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err("wifi::get_adapter_connectivity not supported on this OS".into())
        }
    }
}

