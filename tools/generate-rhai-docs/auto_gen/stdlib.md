import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';


# Std Library Module

These are all the standard modules in ewwii.

Each library in this module is under `std::<m>`, where `<m>` is the name of the specific module.
These modules provide essential functionalities that are will be useful for making widgets.
They cover tasks like string manipulation, environmental variable manipuation, running shell commands, and more.
    

# command


```Namespace: global/std/command```




## <code>fn</code> run {#fn-run}

```js
fn run(cmd: String)
```

<Tabs>
    <TabItem value="Description" default>

        Executes a shell command without capturing the output.
    </TabItem>
    <TabItem value="Arguments" default>


        * `cmd` - The shell command to execute as a string.
    </TabItem>
    <TabItem value="Returns" default>


        This function returns nothing if the command executes successfully. If there is an error
        running the command, it returns the error.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::command" as cmd;
        
        // Run a shell command (e.g., list directory contents)
        cmd::run("ls -l");
        ```
    </TabItem>
</Tabs>

## <code>fn</code> run_and_read {#fn-run_and_read}

```js
fn run_and_read(cmd: String) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Executes a shell command and captures its output.
    </TabItem>
    <TabItem value="Arguments" default>


        * `cmd` - The shell command to execute as a string.
    </TabItem>
    <TabItem value="Returns" default>


        This function returns the standard output of the command as a `string`. If the command fails,
        it returns the error.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::command" as cmd;
        
        // Run a shell command and capture its output
        let output = cmd::run_and_read("echo 'Hello, world!'");
        print(output); // output: Hello, world!
        ```
    </TabItem>
</Tabs>

# text


```Namespace: global/std/text```

A module providing utility functions for string manipulation.


## <code>fn</code> to_camel_case {#fn-to_camel_case}

```js
fn to_camel_case(text: String) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Converts a string to camel case.
    </TabItem>
    <TabItem value="Arguments" default>


        * `text` - A string to be converted to camel case.
    </TabItem>
    <TabItem value="Returns" default>


        Returns the `text` in camel case format.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::text" as text;
        
        let result = text::to_camel_case("hello world example");
        print(result); // output: "helloWorldExample"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> to_lower {#fn-to_lower}

```js
fn to_lower(s: String) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Converts a string to lowercase.
    </TabItem>
    <TabItem value="Arguments" default>


        * `s` - A string to be converted to lowercase.
    </TabItem>
    <TabItem value="Returns" default>


        Returns the string in lowercase.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::text" as text;
        
        let result = text::to_lower("HELLO");
        print(result); // output: "hello"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> to_slug {#fn-to_slug}

```js
fn to_slug(text: String) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Converts a string to a slug (lowercase words joined by hyphens).
    </TabItem>
    <TabItem value="Arguments" default>


        * `text` - A string to be converted to a slug.
    </TabItem>
    <TabItem value="Returns" default>


        Returns the `text` as a slug.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::text" as text;
        
        let result = text::to_slug("Hello World!");
        print(result); // output: "hello-world"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> to_upper {#fn-to_upper}

```js
fn to_upper(s: String) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Converts a string to uppercase.
    </TabItem>
    <TabItem value="Arguments" default>


        * `s` - A string to be converted to uppercase.
    </TabItem>
    <TabItem value="Returns" default>


        Returns the string in uppercase.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::text" as text;
        
        let result = text::to_upper("hello");
        print(result); // output: "HELLO"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> truncate_chars {#fn-truncate_chars}

```js
fn truncate_chars(text: String, max_chars: int) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Truncates a string to the specified number of characters.
    </TabItem>
    <TabItem value="Arguments" default>


        * `text` - A string to be truncated.
        * `max_chars` - The maximum number of characters to keep in the string.
    </TabItem>
    <TabItem value="Returns" default>


        Returns a truncated string.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::text" as text;
        
        let result = text::truncate_chars("Hello World!", 5);
        print(result); // output: "Hello"
        ```
    </TabItem>
</Tabs>

# monitor


```Namespace: global/std/monitor```




## <code>fn</code> all_resolutions {#fn-all_resolutions}

```js
fn all_resolutions() -> Vec<[int;2]>
```

<Tabs>
    <TabItem value="Description" default>

        Get the resolutions of all connected monitors.
    </TabItem>
    <TabItem value="Returns" default>


        Returns an array of arrays, where each inner array contains the width and height of a monitor.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::monitor" as monitor;
        
        let resolutions = monitor::all_resolutions();
        print(resolutions); // Output: [[width1, height1], [width2, height2], ...]
        ```
    </TabItem>
</Tabs>

## <code>fn</code> all_resolutions_str {#fn-all_resolutions_str}

```js
fn all_resolutions_str() -> String
```

<Tabs>
    <TabItem value="Description" default>

        Get the resolutions of all connected monitors as a string.
    </TabItem>
    <TabItem value="Returns" default>


        Returns a string where each monitor's resolution is formatted as "width x height", separated by commas.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::monitor" as monitor;
        
        let resolutions_str = monitor::all_resolutions_str();
        print(resolutions_str); // Output: "1920x1080, 1280x720"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> count {#fn-count}

```js
fn count() -> int
```

<Tabs>
    <TabItem value="Description" default>

        Get the number of connected monitors.
    </TabItem>
    <TabItem value="Returns" default>


        Returns the total number of connected monitors as an `i64`.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::monitor" as monitor;
        
        let count = monitor::count();
        print(count); // Output: Number of connected monitors
        ```
    </TabItem>
</Tabs>

## <code>fn</code> dimensions {#fn-dimensions}

```js
fn dimensions(index: int) -> [int;4]
```

<Tabs>
    <TabItem value="Description" default>

        Get the dimensions (x, y, width, height) of a specific monitor.
    </TabItem>
    <TabItem value="Arguments" default>


        * `index` - The index of the monitor (0-based).
    </TabItem>
    <TabItem value="Returns" default>


        Returns an array with the monitor's position (x, y) and size (width, height).
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::monitor" as monitor;
        
        let dimensions = monitor::dimensions(0);
        print(dimensions); // Output: [x, y, width, height]
        ```
    </TabItem>
</Tabs>

## <code>fn</code> dimensions_str {#fn-dimensions_str}

```js
fn dimensions_str(index: int) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Get the dimensions of a specific monitor as a string.
    </TabItem>
    <TabItem value="Arguments" default>


        * `index` - The index of the monitor (0-based).
    </TabItem>
    <TabItem value="Returns" default>


        Returns the monitor's dimensions as a string in the format "x,y - width x height".
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::monitor" as monitor;
        
        let dimensions_str = monitor::dimensions_str(0);
        print(dimensions_str); // Output: "0,0 - 1920x1080"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> dpi {#fn-dpi}

```js
fn dpi(index: int) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Get the DPI (dots per inch) of a specific monitor.
    </TabItem>
    <TabItem value="Arguments" default>


        * `index` - The index of the monitor (0-based).
    </TabItem>
    <TabItem value="Returns" default>


        Returns the DPI (scale factor * base DPI) of the monitor as a `f64`.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::monitor" as monitor;
        
        let dpi = monitor::dpi(0);
        print(dpi); // Output: DPI of the monitor
        ```
    </TabItem>
</Tabs>

## <code>fn</code> dpi_str {#fn-dpi_str}

```js
fn dpi_str(index: int) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Get the DPI of a specific monitor as a string.
    </TabItem>
    <TabItem value="Arguments" default>


        * `index` - The index of the monitor (0-based).
    </TabItem>
    <TabItem value="Returns" default>


        Returns the DPI of the monitor as a string formatted to 1 decimal place.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::monitor" as monitor;
        
        let dpi_str = monitor::dpi_str(0);
        print(dpi_str); // Output: "96.0"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> primary_resolution {#fn-primary_resolution}

```js
fn primary_resolution() -> [int;2]
```

<Tabs>
    <TabItem value="Description" default>

        Get the resolution of the primary monitor.
    </TabItem>
    <TabItem value="Returns" default>


        Returns an array containing the width and height of the primary monitor as two `i64` values.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::monitor" as monitor;
        
        let resolution = monitor::primary_resolution();
        print(resolution); // Output: [width, height]
        ```
    </TabItem>
</Tabs>

## <code>fn</code> primary_resolution_str {#fn-primary_resolution_str}

```js
fn primary_resolution_str() -> String
```

<Tabs>
    <TabItem value="Description" default>

        Get the resolution of the primary monitor as a string.
    </TabItem>
    <TabItem value="Returns" default>


        Returns the resolution of the primary monitor as a string in the format "width x height".
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::monitor" as monitor;
        
        let resolution_str = monitor::primary_resolution_str();
        print(resolution_str); // Output: "1920x1080"
        ```
    </TabItem>
</Tabs>

# env


```Namespace: global/std/env```




## <code>fn</code> get_current_dir {#fn-get_current_dir}

```js
fn get_current_dir() -> String
```

<Tabs>
    <TabItem value="Description" default>

        Get the current working directory.
    </TabItem>
    <TabItem value="Returns" default>


        This function returns the current working directory as a `String`. If there is an error
        (e.g., if the path cannot be retrieved), it returns a `Result::Err` with the error message.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::env" as env;
        
        // Get the current working directory
        let current_dir = env::get_current_dir();
        print(current_dir); // output: /home/username/project
        ```
    </TabItem>
</Tabs>

## <code>fn</code> get_env {#fn-get_env}

```js
fn get_env(var: String) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Get the value of an environment variable.
    </TabItem>
    <TabItem value="Arguments" default>


        * `var` - The name of the environment variable to retrieve.
    </TabItem>
    <TabItem value="Returns" default>


        This function returns the value of the environment variable as a `String`.
        If the variable is not found or there is an error, it returns a `Result::Err` with the error message.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::env" as env;
        
        // Get the value of the "HOME" environment variable
        let home_dir = env::get_env("HOME");
        print(home_dir); // output: /home/username
        ```
    </TabItem>
</Tabs>

## <code>fn</code> get_home_dir {#fn-get_home_dir}

```js
fn get_home_dir() -> String
```

<Tabs>
    <TabItem value="Description" default>

        Get the path to the home directory.
    </TabItem>
    <TabItem value="Returns" default>


        This function returns the value of the "HOME" environment variable as a `String`.
        If the variable is not found or there is an error, it returns a `Result::Err` with the error message.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::env" as env;
        
        // Get the home directory
        let home_dir = env::get_home_dir();
        print(home_dir); // output: /home/username
        ```
    </TabItem>
</Tabs>

## <code>fn</code> get_username {#fn-get_username}

```js
fn get_username() -> String
```

<Tabs>
    <TabItem value="Description" default>

        Get the current username.
    </TabItem>
    <TabItem value="Returns" default>


        This function returns the value of the "USER" environment variable as a `String`.
        If the variable is not found or there is an error, it returns a `Result::Err` with the error message.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::env" as env;
        
        // Get the username of the current user
        let username = env::get_username();
        print(username); // output: username
        ```
    </TabItem>
</Tabs>

## <code>fn</code> set_env {#fn-set_env}

```js
fn set_env(var: String, value: String)
```

<Tabs>
    <TabItem value="Description" default>

        Set the value of an environment variable.
    </TabItem>
    <TabItem value="Arguments" default>


        * `var` - The name of the environment variable to set.
        * `value` - The value to assign to the environment variable.
    </TabItem>
    <TabItem value="Returns" default>


        This function does not return a value.
    </TabItem>
    <TabItem value="Example" default>


        ```javascript
        import "std::env" as env;
        
        // Set the value of the "MY_VAR" environment variable
        env::set_env("MY_VAR", "SomeValue");
        ```
    </TabItem>
</Tabs>