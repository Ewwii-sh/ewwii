import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';


# API Library Module

These are all the API modules available in ewwii.

Each library in this module is under `api::<m>`, where `<m>` is the name of the specific module.

The API library provides system-level functionality, allowing you to interact with external resources and perform advanced operations. Examples include interacting with Wi-Fi, networking, and more.
    

# linux


```Namespace: global/api/linux```




## <code>fn</code> get_battery_perc {#fn-get_battery_perc}

```js
fn get_battery_perc() -> int
```

<Tabs>
    <TabItem value="Description" default>

        Get the current battery percentage.
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        The battery percentage as an integer.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::linux" as linux;
        
        let battery_perc = linux::get_battery_perc();
        ```
    </TabItem>
</Tabs>

## <code>fn</code> get_cpu_info {#fn-get_cpu_info}

```js
fn get_cpu_info() -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Get the device CPU information.
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        An `Array` containing the CPU information in a `Map`:
        - `core_id` (Integer)
        - `model`
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::linux" as linux;
        
        let cpu_info = linux::get_cpu_info();
        let cpu_model = cpu_info[0].model; // get the model of the 1st core
        ```
    </TabItem>
</Tabs>

## <code>fn</code> get_disk_info {#fn-get_disk_info}

```js
fn get_disk_info() -> Map
```

<Tabs>
    <TabItem value="Description" default>

        Get the device disk information.
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        A `Map` containing the disk information:
        - Each key is a `mountpoint`, value is a map with:
            - `device` (String)
            - `total_kb` (Integer)
            - `used_kb` (Integer)
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::linux" as linux;
        
        let disk_info = linux::get_disk_info();
        let device = disk_info.device;
        let total_kb = disk_info.total_kb;
        ```
    </TabItem>
</Tabs>

## <code>fn</code> get_gpu_info {#fn-get_gpu_info}

```js
fn get_gpu_info() -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Get the device GPU information.
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        An `Array` containing the GPU information in a `Map`:
        - `sys_name` (String): e.g., "card0"
        - `model` (String)
        - `vendor` (String)
        - `memory_kb` (Integer)
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::linux" as linux;
        
        let gpu_info = linux::get_gpu_info();
        let model = gpu_info[0].model; // 1st GPU card model
        ```
    </TabItem>
</Tabs>

## <code>fn</code> get_kernel_version {#fn-get_kernel_version}

```js
fn get_kernel_version() -> String
```

<Tabs>
    <TabItem value="Description" default>

        Get the installed Linux Kernel version.
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        The Kernel version as a `String`. (e.g. "6.16.4-arch1-1")
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::linux" as linux;
        
        let k_version = linux::get_kernel_version();
        ```
    </TabItem>
</Tabs>

## <code>fn</code> get_ram_info {#fn-get_ram_info}

```js
fn get_ram_info() -> Map
```

<Tabs>
    <TabItem value="Description" default>

        Get the device RAM information.
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        A `Map` containing the RAM information:
        - `total_kb`
        - `free_kb`
        - `available_kb`
        - `used_kb`
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::linux" as linux;
        
        let ram_info = linux::get_ram_info();
        let used_kb = ram_info.used_kb;
        ```
    </TabItem>
</Tabs>

# wifi


```Namespace: global/api/wifi```




## <code>fn</code> connect {#fn-connect}

```js
fn connect(ssid: String, password: String)
```

<Tabs>
    <TabItem value="Description" default>

        Connects to a Wi-Fi network with the specified SSID and password.
    </TabItem>
    <TabItem value="Arguments" default>


        * `ssid` - The SSID of the Wi-Fi network.
        * `password` - The password of the Wi-Fi network (optional for open networks).
    </TabItem>
    <TabItem value="Returns" default>


        Returns nothing if the connection is successful, or an error message if it fails.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::wifi" as wifi;
        
        wifi::connect("MySecretNetwork", "password123");
        ```
    </TabItem>
</Tabs>

## <code>fn</code> connect_without_password {#fn-connect_without_password}

```js
fn connect_without_password(ssid: String)
```

<Tabs>
    <TabItem value="Description" default>

        Connects to a Wi-Fi network with the specified SSID using saved profile (no password required).
    </TabItem>
    <TabItem value="Arguments" default>


        * `ssid` - The SSID of the Wi-Fi network.
    </TabItem>
    <TabItem value="Returns" default>


        Returns nothing if the connection is successful, or an error message if it fails.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::wifi" as wifi;
        
        wifi::connect_without_password("MySecretNetwork", "password123");
        ```
    </TabItem>
</Tabs>

## <code>fn</code> current_connection {#fn-current_connection}

```js
fn current_connection() -> Map
```

<Tabs>
    <TabItem value="Description" default>

        Retrieves the current active Wi-Fi connection's details (SSID, signal, and security).
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        A `Map` containing the current connection's SSID, signal strength, and security type.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::wifi" as wifi;
        
        let connection = wifi::current_connection();
        ```
    </TabItem>
</Tabs>

## <code>fn</code> disable_adapter {#fn-disable_adapter}

```js
fn disable_adapter()
```

<Tabs>
    <TabItem value="Description" default>

        Disables the Wi-Fi adapter.
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        Returns nothing if the connection is successful, or an error message if it fails.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::wifi" as wifi;
        
        wifi::disable_adapter();
        ```
    </TabItem>
</Tabs>

## <code>fn</code> disconnect {#fn-disconnect}

```js
fn disconnect()
```

<Tabs>
    <TabItem value="Description" default>

        Disconnects from the current Wi-Fi network.
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        Returns nothing if the connection is successful, or an error message if it fails.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::wifi" as wifi;
        
        wifi::disconnect();
        ```
    </TabItem>
</Tabs>

## <code>fn</code> enable_adapter {#fn-enable_adapter}

```js
fn enable_adapter()
```

<Tabs>
    <TabItem value="Description" default>

        Enables the Wi-Fi adapter.
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        Returns nothing if the connection is successful, or an error message if it fails.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::wifi" as wifi;
        
        wifi::enable_adapter();
        ```
    </TabItem>
</Tabs>

## <code>fn</code> get_adapter_connectivity {#fn-get_adapter_connectivity}

```js
fn get_adapter_connectivity() -> String
```

<Tabs>
    <TabItem value="Description" default>

        Get the currenet state of adapter.
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        Returns the state of the adapter as a `string` and returns an error if getting the state failed.
        
        **Possible returns in Linux:**
        
        - `"full"` (internet available)
        - `"limited"` (network only, no internet)
        - `"portal"` (captive portal)
        - `"none"` (no connectivity)
        
        **Possible returns in macOS:**
        
        - `"full"` (connected to a Wi-Fi network)
        - `"none"` (not connected)
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::wifi" as wifi;
        
        wifi::enable_adapter();
        ```
    </TabItem>
</Tabs>

## <code>fn</code> scan {#fn-scan}

```js
fn scan() -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Scans for all available Wi-Fi connections, platform-dependent (Linux or macOS).
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        An `Array` containing information about each Wi-Fi connection, where each entry is a `Map`
        with keys "ssid", "signal", and "security" representing the Wi-Fi network's SSID, signal strength,
        and security type.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::wifi" as wifi;
        
        let networks = wifi::scan();
        ```
    </TabItem>
</Tabs>

## <code>fn</code> scan_linux {#fn-scan_linux}

```js
fn scan_linux() -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Scans for all available Wi-Fi connections on Linux.
    </TabItem>
    <TabItem value="Arguments" default>


        This function does not require any arguments.
    </TabItem>
    <TabItem value="Returns" default>


        An `Array` containing information about each Wi-Fi connection, where each entry is a `Map`
        with keys "ssid", "signal", and "security" representing the Wi-Fi network's SSID, signal strength,
        and security type.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "api::wifi" as wifi;
        
        let networks = wifi::scan();
        ```
    </TabItem>
</Tabs>