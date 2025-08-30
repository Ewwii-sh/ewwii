# wifi

```Namespace: global/api/wifi```

<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> connect </h2>

```rust,ignore
fn connect(ssid: String, password: String)
```

<div>
<div class="tab">
<button group="connect" id="link-connect-Description"  class="tablinks active" 
    onclick="openTab(event, 'connect', 'Description')">
Description
</button>
<button group="connect" id="link-connect-Arguments"  class="tablinks" 
    onclick="openTab(event, 'connect', 'Arguments')">
Arguments
</button>
<button group="connect" id="link-connect-Returns"  class="tablinks" 
    onclick="openTab(event, 'connect', 'Returns')">
Returns
</button>
<button group="connect" id="link-connect-Example"  class="tablinks" 
    onclick="openTab(event, 'connect', 'Example')">
Example
</button>
</div>

<div group="connect" id="connect-Description" class="tabcontent"  style="display: block;" >
Connects to a Wi-Fi network with the specified SSID and password.
</div>
<div group="connect" id="connect-Arguments" class="tabcontent"  style="display: none;" >

* `ssid` - The SSID of the Wi-Fi network.
* `password` - The password of the Wi-Fi network (optional for open networks).
</div>
<div group="connect" id="connect-Returns" class="tabcontent"  style="display: none;" >

Returns nothing if the connection is successful, or an error message if it fails.
</div>
<div group="connect" id="connect-Example" class="tabcontent"  style="display: none;" >

```js
import "api::wifi" as wifi;

wifi::connect("MySecretNetwork", "password123");
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> connect_without_password </h2>

```rust,ignore
fn connect_without_password(ssid: String)
```

<div>
<div class="tab">
<button group="connect_without_password" id="link-connect_without_password-Description"  class="tablinks active" 
    onclick="openTab(event, 'connect_without_password', 'Description')">
Description
</button>
<button group="connect_without_password" id="link-connect_without_password-Arguments"  class="tablinks" 
    onclick="openTab(event, 'connect_without_password', 'Arguments')">
Arguments
</button>
<button group="connect_without_password" id="link-connect_without_password-Returns"  class="tablinks" 
    onclick="openTab(event, 'connect_without_password', 'Returns')">
Returns
</button>
<button group="connect_without_password" id="link-connect_without_password-Example"  class="tablinks" 
    onclick="openTab(event, 'connect_without_password', 'Example')">
Example
</button>
</div>

<div group="connect_without_password" id="connect_without_password-Description" class="tabcontent"  style="display: block;" >
Connects to a Wi-Fi network with the specified SSID using saved profile (no password required).
</div>
<div group="connect_without_password" id="connect_without_password-Arguments" class="tabcontent"  style="display: none;" >

* `ssid` - The SSID of the Wi-Fi network.
</div>
<div group="connect_without_password" id="connect_without_password-Returns" class="tabcontent"  style="display: none;" >

Returns nothing if the connection is successful, or an error message if it fails.
</div>
<div group="connect_without_password" id="connect_without_password-Example" class="tabcontent"  style="display: none;" >

```js
import "api::wifi" as wifi;

wifi::connect_without_password("MySecretNetwork", "password123");
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> current_connection </h2>

```rust,ignore
fn current_connection() -> Map
```

<div>
<div class="tab">
<button group="current_connection" id="link-current_connection-Description"  class="tablinks active" 
    onclick="openTab(event, 'current_connection', 'Description')">
Description
</button>
<button group="current_connection" id="link-current_connection-Arguments"  class="tablinks" 
    onclick="openTab(event, 'current_connection', 'Arguments')">
Arguments
</button>
<button group="current_connection" id="link-current_connection-Returns"  class="tablinks" 
    onclick="openTab(event, 'current_connection', 'Returns')">
Returns
</button>
<button group="current_connection" id="link-current_connection-Example"  class="tablinks" 
    onclick="openTab(event, 'current_connection', 'Example')">
Example
</button>
</div>

<div group="current_connection" id="current_connection-Description" class="tabcontent"  style="display: block;" >
Retrieves the current active Wi-Fi connection's details (SSID, signal, and security).
</div>
<div group="current_connection" id="current_connection-Arguments" class="tabcontent"  style="display: none;" >

This function does not require any arguments.
</div>
<div group="current_connection" id="current_connection-Returns" class="tabcontent"  style="display: none;" >

A `Map` containing the current connection's SSID, signal strength, and security type.
</div>
<div group="current_connection" id="current_connection-Example" class="tabcontent"  style="display: none;" >

```js
import "api::wifi" as wifi;

let connection = wifi::current_connection();
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> disable_adapter </h2>

```rust,ignore
fn disable_adapter()
```

<div>
<div class="tab">
<button group="disable_adapter" id="link-disable_adapter-Description"  class="tablinks active" 
    onclick="openTab(event, 'disable_adapter', 'Description')">
Description
</button>
<button group="disable_adapter" id="link-disable_adapter-Arguments"  class="tablinks" 
    onclick="openTab(event, 'disable_adapter', 'Arguments')">
Arguments
</button>
<button group="disable_adapter" id="link-disable_adapter-Returns"  class="tablinks" 
    onclick="openTab(event, 'disable_adapter', 'Returns')">
Returns
</button>
<button group="disable_adapter" id="link-disable_adapter-Example"  class="tablinks" 
    onclick="openTab(event, 'disable_adapter', 'Example')">
Example
</button>
</div>

<div group="disable_adapter" id="disable_adapter-Description" class="tabcontent"  style="display: block;" >
Disables the Wi-Fi adapter.
</div>
<div group="disable_adapter" id="disable_adapter-Arguments" class="tabcontent"  style="display: none;" >

This function does not require any arguments.
</div>
<div group="disable_adapter" id="disable_adapter-Returns" class="tabcontent"  style="display: none;" >

Returns nothing if the connection is successful, or an error message if it fails.
</div>
<div group="disable_adapter" id="disable_adapter-Example" class="tabcontent"  style="display: none;" >

```js
import "api::wifi" as wifi;

wifi::disable_adapter();
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> disconnect </h2>

```rust,ignore
fn disconnect()
```

<div>
<div class="tab">
<button group="disconnect" id="link-disconnect-Description"  class="tablinks active" 
    onclick="openTab(event, 'disconnect', 'Description')">
Description
</button>
<button group="disconnect" id="link-disconnect-Arguments"  class="tablinks" 
    onclick="openTab(event, 'disconnect', 'Arguments')">
Arguments
</button>
<button group="disconnect" id="link-disconnect-Returns"  class="tablinks" 
    onclick="openTab(event, 'disconnect', 'Returns')">
Returns
</button>
<button group="disconnect" id="link-disconnect-Example"  class="tablinks" 
    onclick="openTab(event, 'disconnect', 'Example')">
Example
</button>
</div>

<div group="disconnect" id="disconnect-Description" class="tabcontent"  style="display: block;" >
Disconnects from the current Wi-Fi network.
</div>
<div group="disconnect" id="disconnect-Arguments" class="tabcontent"  style="display: none;" >

This function does not require any arguments.
</div>
<div group="disconnect" id="disconnect-Returns" class="tabcontent"  style="display: none;" >

Returns nothing if the connection is successful, or an error message if it fails.
</div>
<div group="disconnect" id="disconnect-Example" class="tabcontent"  style="display: none;" >

```js
import "api::wifi" as wifi;

wifi::disconnect();
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> enable_adapter </h2>

```rust,ignore
fn enable_adapter()
```

<div>
<div class="tab">
<button group="enable_adapter" id="link-enable_adapter-Description"  class="tablinks active" 
    onclick="openTab(event, 'enable_adapter', 'Description')">
Description
</button>
<button group="enable_adapter" id="link-enable_adapter-Arguments"  class="tablinks" 
    onclick="openTab(event, 'enable_adapter', 'Arguments')">
Arguments
</button>
<button group="enable_adapter" id="link-enable_adapter-Returns"  class="tablinks" 
    onclick="openTab(event, 'enable_adapter', 'Returns')">
Returns
</button>
<button group="enable_adapter" id="link-enable_adapter-Example"  class="tablinks" 
    onclick="openTab(event, 'enable_adapter', 'Example')">
Example
</button>
</div>

<div group="enable_adapter" id="enable_adapter-Description" class="tabcontent"  style="display: block;" >
Enables the Wi-Fi adapter.
</div>
<div group="enable_adapter" id="enable_adapter-Arguments" class="tabcontent"  style="display: none;" >

This function does not require any arguments.
</div>
<div group="enable_adapter" id="enable_adapter-Returns" class="tabcontent"  style="display: none;" >

Returns nothing if the connection is successful, or an error message if it fails.
</div>
<div group="enable_adapter" id="enable_adapter-Example" class="tabcontent"  style="display: none;" >

```js
import "api::wifi" as wifi;

wifi::enable_adapter();
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> get_adapter_connectivity </h2>

```rust,ignore
fn get_adapter_connectivity() -> String
```

<div>
<div class="tab">
<button group="get_adapter_connectivity" id="link-get_adapter_connectivity-Description"  class="tablinks active" 
    onclick="openTab(event, 'get_adapter_connectivity', 'Description')">
Description
</button>
<button group="get_adapter_connectivity" id="link-get_adapter_connectivity-Arguments"  class="tablinks" 
    onclick="openTab(event, 'get_adapter_connectivity', 'Arguments')">
Arguments
</button>
<button group="get_adapter_connectivity" id="link-get_adapter_connectivity-Returns"  class="tablinks" 
    onclick="openTab(event, 'get_adapter_connectivity', 'Returns')">
Returns
</button>
<button group="get_adapter_connectivity" id="link-get_adapter_connectivity-Example"  class="tablinks" 
    onclick="openTab(event, 'get_adapter_connectivity', 'Example')">
Example
</button>
</div>

<div group="get_adapter_connectivity" id="get_adapter_connectivity-Description" class="tabcontent"  style="display: block;" >
Get the currenet state of adapter.
</div>
<div group="get_adapter_connectivity" id="get_adapter_connectivity-Arguments" class="tabcontent"  style="display: none;" >

This function does not require any arguments.
</div>
<div group="get_adapter_connectivity" id="get_adapter_connectivity-Returns" class="tabcontent"  style="display: none;" >

Returns the state of the adapter as a `string` and returns an error if getting the state failed.

**Possible returns in Linux:** 

- `"full"` (internet available)
- `"limited"` (network only, no internet)
- `"portal"` (captive portal)
- `"none"` (no connectivity)

**Possible returns in macOS:** 

- `"full"` (connected to a Wi-Fi network)
- `"none"` (not connected)
</div>
<div group="get_adapter_connectivity" id="get_adapter_connectivity-Example" class="tabcontent"  style="display: none;" >

```js
import "api::wifi" as wifi;

wifi::enable_adapter();
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> scan </h2>

```rust,ignore
fn scan() -> Array
```

<div>
<div class="tab">
<button group="scan" id="link-scan-Description"  class="tablinks active" 
    onclick="openTab(event, 'scan', 'Description')">
Description
</button>
<button group="scan" id="link-scan-Arguments"  class="tablinks" 
    onclick="openTab(event, 'scan', 'Arguments')">
Arguments
</button>
<button group="scan" id="link-scan-Returns"  class="tablinks" 
    onclick="openTab(event, 'scan', 'Returns')">
Returns
</button>
<button group="scan" id="link-scan-Example"  class="tablinks" 
    onclick="openTab(event, 'scan', 'Example')">
Example
</button>
</div>

<div group="scan" id="scan-Description" class="tabcontent"  style="display: block;" >
Scans for all available Wi-Fi connections, platform-dependent (Linux or macOS).
</div>
<div group="scan" id="scan-Arguments" class="tabcontent"  style="display: none;" >

This function does not require any arguments.
</div>
<div group="scan" id="scan-Returns" class="tabcontent"  style="display: none;" >

An `Array` containing information about each Wi-Fi connection, where each entry is a `Map`
with keys "ssid", "signal", and "security" representing the Wi-Fi network's SSID, signal strength,
and security type.
</div>
<div group="scan" id="scan-Example" class="tabcontent"  style="display: none;" >

```js
import "api::wifi" as wifi;

let networks = wifi::scan();
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> scan_linux </h2>

```rust,ignore
fn scan_linux() -> Array
```

<div>
<div class="tab">
<button group="scan_linux" id="link-scan_linux-Description"  class="tablinks active" 
    onclick="openTab(event, 'scan_linux', 'Description')">
Description
</button>
<button group="scan_linux" id="link-scan_linux-Arguments"  class="tablinks" 
    onclick="openTab(event, 'scan_linux', 'Arguments')">
Arguments
</button>
<button group="scan_linux" id="link-scan_linux-Returns"  class="tablinks" 
    onclick="openTab(event, 'scan_linux', 'Returns')">
Returns
</button>
<button group="scan_linux" id="link-scan_linux-Example"  class="tablinks" 
    onclick="openTab(event, 'scan_linux', 'Example')">
Example
</button>
</div>

<div group="scan_linux" id="scan_linux-Description" class="tabcontent"  style="display: block;" >
Scans for all available Wi-Fi connections on Linux.
</div>
<div group="scan_linux" id="scan_linux-Arguments" class="tabcontent"  style="display: none;" >

This function does not require any arguments.
</div>
<div group="scan_linux" id="scan_linux-Returns" class="tabcontent"  style="display: none;" >

An `Array` containing information about each Wi-Fi connection, where each entry is a `Map`
with keys "ssid", "signal", and "security" representing the Wi-Fi network's SSID, signal strength,
and security type.
</div>
<div group="scan_linux" id="scan_linux-Example" class="tabcontent"  style="display: none;" >

```js
import "api::wifi" as wifi;

let networks = wifi::scan();
```
</div>

</div>
</div>
</br>
