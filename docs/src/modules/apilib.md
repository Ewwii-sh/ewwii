# API Library

## `api::wifi`

The `wifi` module provides cross-platform Wi-Fi management for Linux and macOS systems. Functions include scanning networks, querying the current connection, connecting/disconnecting, and enabling/disabling the Wi-Fi adapter.

> **Note:** macOS support is largely untested and may behave differently depending on system configuration.

### Usage

```js
import "api::wifi" as wifi;

// Scan for available networks
let networks = wifi::scan();

// Get current Wi-Fi connection info
let current = wifi::current_connection();

// Connect to a network with password
wifi::connect("MySSID", "MyPassword");

// Connect to a network without password (using saved profile)
wifi::connect_without_password("MySSID");

// Disconnect from the current network
wifi::disconnect();

// Enable/disable the Wi-Fi adapter
wifi::enable_adapter();
wifi::disable_adapter();

// Get adapter connection
wifi::get_adapter_connectivity();
```

### Functions

| Function                         | Description                                                                                     |
| -------------------------------- | ----------------------------------------------------------------------------------------------- |
| `scan()`                         | Returns a list of nearby Wi-Fi networks with `ssid`, `signal`, and `security` fields.           |
| `scan_linux()`                   | **Linux only**. Returns a list of nearby Wi-Fi networks. Equivalent to `scan()`.                |
| `scan_macos()`                   | **macOS only (untested)**. Returns a list of nearby Wi-Fi networks. Equivalent to `scan()`.     |
| `current_connection()`           | Returns information about the current Wi-Fi connection as a map (`ssid`, `signal`, `security`). |
| `connect(ssid, password)`        | Connects to a Wi-Fi network.                                                                    |
| `connect_without_password(ssid)` | Connects to a Wi-Fi network without password by using the saved profile.                        |
| `disconnect()`                   | Disconnects from the currently connected network (does not disable the adapter).                |
| `enable_adapter()`               | Turns the Wi-Fi adapter on.                                                                     |
| `get_adapter_connectivity()`     | Returns a normalized connectivity status of the Wi-Fi adapter.                                  |

### Extra Notes

`get_adapter_connectivity()` has different outcome possibilities in each OS.

**All possible results:**

-   **Linux**: `"full"` (internet available), `"limited"` (network only, no internet), `"portal"` (captive portal), `"none"` (no connectivity)
-   **macOS**: `"full"` (connected to a Wi-Fi network) or `"none"` (not connected)

### Platform Notes

-   **Linux**: Uses `nmcli` for all operations. The module assumes `nmcli` is installed and accessible in `$PATH`.
-   **macOS**: Uses `/System/Library/PrivateFrameworks/Apple80211.framework/.../airport` for scanning and disconnecting, and `networksetup` for connecting and enabling/disabling the adapter.
-   **Unsupported OS**: All functions return an error if the platform is neither Linux nor macOS.

### Returned Data Formats

-   `scan()` (or `scan_linux` / `scan_macos`) returns an **array of maps**:

```js
[
    { ssid: "HomeWiFi", signal: "78", security: "WPA2" },
    { ssid: "CafeNet", signal: "65", security: "WPA" },
];
```

-   `current_connection()` returns a **map**:

```js
{ "ssid": "HomeWiFi", "signal": "78", "security": "WPA2" }
```
