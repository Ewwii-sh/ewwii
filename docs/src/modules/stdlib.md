# monitor

```Namespace: global/std/monitor```

<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> all_resolutions </h2>

```rust,ignore
fn all_resolutions() -> Vec<[int;2]>
```

<div>
<div class="tab">
<button group="all_resolutions" id="link-all_resolutions-Description"  class="tablinks active" 
    onclick="openTab(event, 'all_resolutions', 'Description')">
Description
</button>
<button group="all_resolutions" id="link-all_resolutions-Returns"  class="tablinks" 
    onclick="openTab(event, 'all_resolutions', 'Returns')">
Returns
</button>
<button group="all_resolutions" id="link-all_resolutions-Example"  class="tablinks" 
    onclick="openTab(event, 'all_resolutions', 'Example')">
Example
</button>
</div>

<div group="all_resolutions" id="all_resolutions-Description" class="tabcontent"  style="display: block;" >
Get the resolutions of all connected monitors.
</div>
<div group="all_resolutions" id="all_resolutions-Returns" class="tabcontent"  style="display: none;" >

Returns an array of arrays, where each inner array contains the width and height of a monitor.
</div>
<div group="all_resolutions" id="all_resolutions-Example" class="tabcontent"  style="display: none;" >

```js
import "std::monitor" as monitor;

let resolutions = monitor::all_resolutions();
print(resolutions); // Output: [[width1, height1], [width2, height2], ...]
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> all_resolutions_str </h2>

```rust,ignore
fn all_resolutions_str() -> String
```

<div>
<div class="tab">
<button group="all_resolutions_str" id="link-all_resolutions_str-Description"  class="tablinks active" 
    onclick="openTab(event, 'all_resolutions_str', 'Description')">
Description
</button>
<button group="all_resolutions_str" id="link-all_resolutions_str-Returns"  class="tablinks" 
    onclick="openTab(event, 'all_resolutions_str', 'Returns')">
Returns
</button>
<button group="all_resolutions_str" id="link-all_resolutions_str-Example"  class="tablinks" 
    onclick="openTab(event, 'all_resolutions_str', 'Example')">
Example
</button>
</div>

<div group="all_resolutions_str" id="all_resolutions_str-Description" class="tabcontent"  style="display: block;" >
Get the resolutions of all connected monitors as a string.
</div>
<div group="all_resolutions_str" id="all_resolutions_str-Returns" class="tabcontent"  style="display: none;" >

Returns a string where each monitor's resolution is formatted as "width x height", separated by commas.
</div>
<div group="all_resolutions_str" id="all_resolutions_str-Example" class="tabcontent"  style="display: none;" >

```js
import "std::monitor" as monitor;

let resolutions_str = monitor::all_resolutions_str();
print(resolutions_str); // Output: "1920x1080, 1280x720"
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> count </h2>

```rust,ignore
fn count() -> int
```

<div>
<div class="tab">
<button group="count" id="link-count-Description"  class="tablinks active" 
    onclick="openTab(event, 'count', 'Description')">
Description
</button>
<button group="count" id="link-count-Returns"  class="tablinks" 
    onclick="openTab(event, 'count', 'Returns')">
Returns
</button>
<button group="count" id="link-count-Example"  class="tablinks" 
    onclick="openTab(event, 'count', 'Example')">
Example
</button>
</div>

<div group="count" id="count-Description" class="tabcontent"  style="display: block;" >
Get the number of connected monitors.
</div>
<div group="count" id="count-Returns" class="tabcontent"  style="display: none;" >

Returns the total number of connected monitors as an `i64`.
</div>
<div group="count" id="count-Example" class="tabcontent"  style="display: none;" >

```js
import "std::monitor" as monitor;

let count = monitor::count();
print(count); // Output: Number of connected monitors
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> dimensions </h2>

```rust,ignore
fn dimensions(index: int) -> [int;4]
```

<div>
<div class="tab">
<button group="dimensions" id="link-dimensions-Description"  class="tablinks active" 
    onclick="openTab(event, 'dimensions', 'Description')">
Description
</button>
<button group="dimensions" id="link-dimensions-Arguments"  class="tablinks" 
    onclick="openTab(event, 'dimensions', 'Arguments')">
Arguments
</button>
<button group="dimensions" id="link-dimensions-Returns"  class="tablinks" 
    onclick="openTab(event, 'dimensions', 'Returns')">
Returns
</button>
<button group="dimensions" id="link-dimensions-Example"  class="tablinks" 
    onclick="openTab(event, 'dimensions', 'Example')">
Example
</button>
</div>

<div group="dimensions" id="dimensions-Description" class="tabcontent"  style="display: block;" >
Get the dimensions (x, y, width, height) of a specific monitor.
</div>
<div group="dimensions" id="dimensions-Arguments" class="tabcontent"  style="display: none;" >

* `index` - The index of the monitor (0-based).
</div>
<div group="dimensions" id="dimensions-Returns" class="tabcontent"  style="display: none;" >

Returns an array with the monitor's position (x, y) and size (width, height).
</div>
<div group="dimensions" id="dimensions-Example" class="tabcontent"  style="display: none;" >

```js
import "std::monitor" as monitor;

let dimensions = monitor::dimensions(0);
print(dimensions); // Output: [x, y, width, height]
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> dimensions_str </h2>

```rust,ignore
fn dimensions_str(index: int) -> String
```

<div>
<div class="tab">
<button group="dimensions_str" id="link-dimensions_str-Description"  class="tablinks active" 
    onclick="openTab(event, 'dimensions_str', 'Description')">
Description
</button>
<button group="dimensions_str" id="link-dimensions_str-Arguments"  class="tablinks" 
    onclick="openTab(event, 'dimensions_str', 'Arguments')">
Arguments
</button>
<button group="dimensions_str" id="link-dimensions_str-Returns"  class="tablinks" 
    onclick="openTab(event, 'dimensions_str', 'Returns')">
Returns
</button>
<button group="dimensions_str" id="link-dimensions_str-Example"  class="tablinks" 
    onclick="openTab(event, 'dimensions_str', 'Example')">
Example
</button>
</div>

<div group="dimensions_str" id="dimensions_str-Description" class="tabcontent"  style="display: block;" >
Get the dimensions of a specific monitor as a string.
</div>
<div group="dimensions_str" id="dimensions_str-Arguments" class="tabcontent"  style="display: none;" >

* `index` - The index of the monitor (0-based).
</div>
<div group="dimensions_str" id="dimensions_str-Returns" class="tabcontent"  style="display: none;" >

Returns the monitor's dimensions as a string in the format "x,y - width x height".
</div>
<div group="dimensions_str" id="dimensions_str-Example" class="tabcontent"  style="display: none;" >

```js
import "std::monitor" as monitor;

let dimensions_str = monitor::dimensions_str(0);
print(dimensions_str); // Output: "0,0 - 1920x1080"
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> dpi </h2>

```rust,ignore
fn dpi(index: int) -> float
```

<div>
<div class="tab">
<button group="dpi" id="link-dpi-Description"  class="tablinks active" 
    onclick="openTab(event, 'dpi', 'Description')">
Description
</button>
<button group="dpi" id="link-dpi-Arguments"  class="tablinks" 
    onclick="openTab(event, 'dpi', 'Arguments')">
Arguments
</button>
<button group="dpi" id="link-dpi-Returns"  class="tablinks" 
    onclick="openTab(event, 'dpi', 'Returns')">
Returns
</button>
<button group="dpi" id="link-dpi-Example"  class="tablinks" 
    onclick="openTab(event, 'dpi', 'Example')">
Example
</button>
</div>

<div group="dpi" id="dpi-Description" class="tabcontent"  style="display: block;" >
Get the DPI (dots per inch) of a specific monitor.
</div>
<div group="dpi" id="dpi-Arguments" class="tabcontent"  style="display: none;" >

* `index` - The index of the monitor (0-based).
</div>
<div group="dpi" id="dpi-Returns" class="tabcontent"  style="display: none;" >

Returns the DPI (scale factor * base DPI) of the monitor as a `f64`.
</div>
<div group="dpi" id="dpi-Example" class="tabcontent"  style="display: none;" >

```js
import "std::monitor" as monitor;

let dpi = monitor::dpi(0);
print(dpi); // Output: DPI of the monitor
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> dpi_str </h2>

```rust,ignore
fn dpi_str(index: int) -> String
```

<div>
<div class="tab">
<button group="dpi_str" id="link-dpi_str-Description"  class="tablinks active" 
    onclick="openTab(event, 'dpi_str', 'Description')">
Description
</button>
<button group="dpi_str" id="link-dpi_str-Arguments"  class="tablinks" 
    onclick="openTab(event, 'dpi_str', 'Arguments')">
Arguments
</button>
<button group="dpi_str" id="link-dpi_str-Returns"  class="tablinks" 
    onclick="openTab(event, 'dpi_str', 'Returns')">
Returns
</button>
<button group="dpi_str" id="link-dpi_str-Example"  class="tablinks" 
    onclick="openTab(event, 'dpi_str', 'Example')">
Example
</button>
</div>

<div group="dpi_str" id="dpi_str-Description" class="tabcontent"  style="display: block;" >
Get the DPI of a specific monitor as a string.
</div>
<div group="dpi_str" id="dpi_str-Arguments" class="tabcontent"  style="display: none;" >

* `index` - The index of the monitor (0-based).
</div>
<div group="dpi_str" id="dpi_str-Returns" class="tabcontent"  style="display: none;" >

Returns the DPI of the monitor as a string formatted to 1 decimal place.
</div>
<div group="dpi_str" id="dpi_str-Example" class="tabcontent"  style="display: none;" >

```js
import "std::monitor" as monitor;

let dpi_str = monitor::dpi_str(0);
print(dpi_str); // Output: "96.0"
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> primary_resolution </h2>

```rust,ignore
fn primary_resolution() -> [int;2]
```

<div>
<div class="tab">
<button group="primary_resolution" id="link-primary_resolution-Description"  class="tablinks active" 
    onclick="openTab(event, 'primary_resolution', 'Description')">
Description
</button>
<button group="primary_resolution" id="link-primary_resolution-Returns"  class="tablinks" 
    onclick="openTab(event, 'primary_resolution', 'Returns')">
Returns
</button>
<button group="primary_resolution" id="link-primary_resolution-Example"  class="tablinks" 
    onclick="openTab(event, 'primary_resolution', 'Example')">
Example
</button>
</div>

<div group="primary_resolution" id="primary_resolution-Description" class="tabcontent"  style="display: block;" >
Get the resolution of the primary monitor.
</div>
<div group="primary_resolution" id="primary_resolution-Returns" class="tabcontent"  style="display: none;" >

Returns an array containing the width and height of the primary monitor as two `i64` values.
</div>
<div group="primary_resolution" id="primary_resolution-Example" class="tabcontent"  style="display: none;" >

```js
import "std::monitor" as monitor;

let resolution = monitor::primary_resolution();
print(resolution); // Output: [width, height]
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> primary_resolution_str </h2>

```rust,ignore
fn primary_resolution_str() -> String
```

<div>
<div class="tab">
<button group="primary_resolution_str" id="link-primary_resolution_str-Description"  class="tablinks active" 
    onclick="openTab(event, 'primary_resolution_str', 'Description')">
Description
</button>
<button group="primary_resolution_str" id="link-primary_resolution_str-Returns"  class="tablinks" 
    onclick="openTab(event, 'primary_resolution_str', 'Returns')">
Returns
</button>
<button group="primary_resolution_str" id="link-primary_resolution_str-Example"  class="tablinks" 
    onclick="openTab(event, 'primary_resolution_str', 'Example')">
Example
</button>
</div>

<div group="primary_resolution_str" id="primary_resolution_str-Description" class="tabcontent"  style="display: block;" >
Get the resolution of the primary monitor as a string.
</div>
<div group="primary_resolution_str" id="primary_resolution_str-Returns" class="tabcontent"  style="display: none;" >

Returns the resolution of the primary monitor as a string in the format "width x height".
</div>
<div group="primary_resolution_str" id="primary_resolution_str-Example" class="tabcontent"  style="display: none;" >

```js
import "std::monitor" as monitor;

let resolution_str = monitor::primary_resolution_str();
print(resolution_str); // Output: "1920x1080"
```
</div>

</div>
</div>
</br>

# text

```Namespace: global/std/text```

<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> to_camel_case </h2>

```rust,ignore
fn to_camel_case(text: String) -> String
```

<div>
<div class="tab">
<button group="to_camel_case" id="link-to_camel_case-Description"  class="tablinks active" 
    onclick="openTab(event, 'to_camel_case', 'Description')">
Description
</button>
<button group="to_camel_case" id="link-to_camel_case-Arguments"  class="tablinks" 
    onclick="openTab(event, 'to_camel_case', 'Arguments')">
Arguments
</button>
<button group="to_camel_case" id="link-to_camel_case-Returns"  class="tablinks" 
    onclick="openTab(event, 'to_camel_case', 'Returns')">
Returns
</button>
<button group="to_camel_case" id="link-to_camel_case-Example"  class="tablinks" 
    onclick="openTab(event, 'to_camel_case', 'Example')">
Example
</button>
</div>

<div group="to_camel_case" id="to_camel_case-Description" class="tabcontent"  style="display: block;" >
Converts a string to camel case.
</div>
<div group="to_camel_case" id="to_camel_case-Arguments" class="tabcontent"  style="display: none;" >

* `text` - A string to be converted to camel case.
</div>
<div group="to_camel_case" id="to_camel_case-Returns" class="tabcontent"  style="display: none;" >

Returns the `text` in camel case format.
</div>
<div group="to_camel_case" id="to_camel_case-Example" class="tabcontent"  style="display: none;" >

```js
import "std::text" as text;

let result = text::to_camel_case("hello world example");
print(result); // output: "helloWorldExample"
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> to_lower </h2>

```rust,ignore
fn to_lower(s: String) -> String
```

<div>
<div class="tab">
<button group="to_lower" id="link-to_lower-Description"  class="tablinks active" 
    onclick="openTab(event, 'to_lower', 'Description')">
Description
</button>
<button group="to_lower" id="link-to_lower-Arguments"  class="tablinks" 
    onclick="openTab(event, 'to_lower', 'Arguments')">
Arguments
</button>
<button group="to_lower" id="link-to_lower-Returns"  class="tablinks" 
    onclick="openTab(event, 'to_lower', 'Returns')">
Returns
</button>
<button group="to_lower" id="link-to_lower-Example"  class="tablinks" 
    onclick="openTab(event, 'to_lower', 'Example')">
Example
</button>
</div>

<div group="to_lower" id="to_lower-Description" class="tabcontent"  style="display: block;" >
Converts a string to lowercase.
</div>
<div group="to_lower" id="to_lower-Arguments" class="tabcontent"  style="display: none;" >

* `s` - A string to be converted to lowercase.
</div>
<div group="to_lower" id="to_lower-Returns" class="tabcontent"  style="display: none;" >

Returns the string in lowercase.
</div>
<div group="to_lower" id="to_lower-Example" class="tabcontent"  style="display: none;" >

```js
import "std::text" as text;

let result = text::to_lower("HELLO");
print(result); // output: "hello"
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> to_slug </h2>

```rust,ignore
fn to_slug(text: String) -> String
```

<div>
<div class="tab">
<button group="to_slug" id="link-to_slug-Description"  class="tablinks active" 
    onclick="openTab(event, 'to_slug', 'Description')">
Description
</button>
<button group="to_slug" id="link-to_slug-Arguments"  class="tablinks" 
    onclick="openTab(event, 'to_slug', 'Arguments')">
Arguments
</button>
<button group="to_slug" id="link-to_slug-Returns"  class="tablinks" 
    onclick="openTab(event, 'to_slug', 'Returns')">
Returns
</button>
<button group="to_slug" id="link-to_slug-Example"  class="tablinks" 
    onclick="openTab(event, 'to_slug', 'Example')">
Example
</button>
</div>

<div group="to_slug" id="to_slug-Description" class="tabcontent"  style="display: block;" >
Converts a string to a slug (lowercase words joined by hyphens).
</div>
<div group="to_slug" id="to_slug-Arguments" class="tabcontent"  style="display: none;" >

* `text` - A string to be converted to a slug.
</div>
<div group="to_slug" id="to_slug-Returns" class="tabcontent"  style="display: none;" >

Returns the `text` as a slug.
</div>
<div group="to_slug" id="to_slug-Example" class="tabcontent"  style="display: none;" >

```js
import "std::text" as text;

let result = text::to_slug("Hello World!");
print(result); // output: "hello-world"
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> to_upper </h2>

```rust,ignore
fn to_upper(s: String) -> String
```

<div>
<div class="tab">
<button group="to_upper" id="link-to_upper-Description"  class="tablinks active" 
    onclick="openTab(event, 'to_upper', 'Description')">
Description
</button>
<button group="to_upper" id="link-to_upper-Arguments"  class="tablinks" 
    onclick="openTab(event, 'to_upper', 'Arguments')">
Arguments
</button>
<button group="to_upper" id="link-to_upper-Returns"  class="tablinks" 
    onclick="openTab(event, 'to_upper', 'Returns')">
Returns
</button>
<button group="to_upper" id="link-to_upper-Example"  class="tablinks" 
    onclick="openTab(event, 'to_upper', 'Example')">
Example
</button>
</div>

<div group="to_upper" id="to_upper-Description" class="tabcontent"  style="display: block;" >
Converts a string to uppercase.
</div>
<div group="to_upper" id="to_upper-Arguments" class="tabcontent"  style="display: none;" >

* `s` - A string to be converted to uppercase.
</div>
<div group="to_upper" id="to_upper-Returns" class="tabcontent"  style="display: none;" >

Returns the string in uppercase.
</div>
<div group="to_upper" id="to_upper-Example" class="tabcontent"  style="display: none;" >

```js
import "std::text" as text;

let result = text::to_upper("hello");
print(result); // output: "HELLO"
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> truncate_chars </h2>

```rust,ignore
fn truncate_chars(text: String, max_chars: int) -> String
```

<div>
<div class="tab">
<button group="truncate_chars" id="link-truncate_chars-Description"  class="tablinks active" 
    onclick="openTab(event, 'truncate_chars', 'Description')">
Description
</button>
<button group="truncate_chars" id="link-truncate_chars-Arguments"  class="tablinks" 
    onclick="openTab(event, 'truncate_chars', 'Arguments')">
Arguments
</button>
<button group="truncate_chars" id="link-truncate_chars-Returns"  class="tablinks" 
    onclick="openTab(event, 'truncate_chars', 'Returns')">
Returns
</button>
<button group="truncate_chars" id="link-truncate_chars-Example"  class="tablinks" 
    onclick="openTab(event, 'truncate_chars', 'Example')">
Example
</button>
</div>

<div group="truncate_chars" id="truncate_chars-Description" class="tabcontent"  style="display: block;" >
Truncates a string to the specified number of characters.
</div>
<div group="truncate_chars" id="truncate_chars-Arguments" class="tabcontent"  style="display: none;" >

* `text` - A string to be truncated.
* `max_chars` - The maximum number of characters to keep in the string.
</div>
<div group="truncate_chars" id="truncate_chars-Returns" class="tabcontent"  style="display: none;" >

Returns a truncated string.
</div>
<div group="truncate_chars" id="truncate_chars-Example" class="tabcontent"  style="display: none;" >

```js
import "std::text" as text;

let result = text::truncate_chars("Hello World!", 5);
print(result); // output: "Hello"
```
</div>

</div>
</div>
</br>

# command

```Namespace: global/std/command```

<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> run </h2>

```rust,ignore
fn run(cmd: String)
```

<div>
<div class="tab">
<button group="run" id="link-run-Description"  class="tablinks active" 
    onclick="openTab(event, 'run', 'Description')">
Description
</button>
<button group="run" id="link-run-Arguments"  class="tablinks" 
    onclick="openTab(event, 'run', 'Arguments')">
Arguments
</button>
<button group="run" id="link-run-Returns"  class="tablinks" 
    onclick="openTab(event, 'run', 'Returns')">
Returns
</button>
<button group="run" id="link-run-Example"  class="tablinks" 
    onclick="openTab(event, 'run', 'Example')">
Example
</button>
</div>

<div group="run" id="run-Description" class="tabcontent"  style="display: block;" >
Executes a shell command without capturing the output.
</div>
<div group="run" id="run-Arguments" class="tabcontent"  style="display: none;" >

* `cmd` - The shell command to execute as a string.
</div>
<div group="run" id="run-Returns" class="tabcontent"  style="display: none;" >

This function returns nothing if the command executes successfully. If there is an error
running the command, it returns the error.
</div>
<div group="run" id="run-Example" class="tabcontent"  style="display: none;" >

```js
import "std::command" as cmd;

// Run a shell command (e.g., list directory contents)
cmd::run("ls -l");
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> run_and_read </h2>

```rust,ignore
fn run_and_read(cmd: String) -> String
```

<div>
<div class="tab">
<button group="run_and_read" id="link-run_and_read-Description"  class="tablinks active" 
    onclick="openTab(event, 'run_and_read', 'Description')">
Description
</button>
<button group="run_and_read" id="link-run_and_read-Arguments"  class="tablinks" 
    onclick="openTab(event, 'run_and_read', 'Arguments')">
Arguments
</button>
<button group="run_and_read" id="link-run_and_read-Returns"  class="tablinks" 
    onclick="openTab(event, 'run_and_read', 'Returns')">
Returns
</button>
<button group="run_and_read" id="link-run_and_read-Example"  class="tablinks" 
    onclick="openTab(event, 'run_and_read', 'Example')">
Example
</button>
</div>

<div group="run_and_read" id="run_and_read-Description" class="tabcontent"  style="display: block;" >
Executes a shell command and captures its output.
</div>
<div group="run_and_read" id="run_and_read-Arguments" class="tabcontent"  style="display: none;" >

* `cmd` - The shell command to execute as a string.
</div>
<div group="run_and_read" id="run_and_read-Returns" class="tabcontent"  style="display: none;" >

This function returns the standard output of the command as a `string`. If the command fails,
it returns the error.
</div>
<div group="run_and_read" id="run_and_read-Example" class="tabcontent"  style="display: none;" >

```js
import "std::command" as cmd;

// Run a shell command and capture its output
let output = cmd::run_and_read("echo 'Hello, world!'");
print(output); // output: Hello, world!
```
</div>

</div>
</div>
</br>

# env

```Namespace: global/std/env```

<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> get_current_dir </h2>

```rust,ignore
fn get_current_dir() -> String
```

<div>
<div class="tab">
<button group="get_current_dir" id="link-get_current_dir-Description"  class="tablinks active" 
    onclick="openTab(event, 'get_current_dir', 'Description')">
Description
</button>
<button group="get_current_dir" id="link-get_current_dir-Returns"  class="tablinks" 
    onclick="openTab(event, 'get_current_dir', 'Returns')">
Returns
</button>
<button group="get_current_dir" id="link-get_current_dir-Example"  class="tablinks" 
    onclick="openTab(event, 'get_current_dir', 'Example')">
Example
</button>
</div>

<div group="get_current_dir" id="get_current_dir-Description" class="tabcontent"  style="display: block;" >
Get the current working directory.
</div>
<div group="get_current_dir" id="get_current_dir-Returns" class="tabcontent"  style="display: none;" >

This function returns the current working directory as a `String`. If there is an error
(e.g., if the path cannot be retrieved), it returns a `Result::Err` with the error message.
</div>
<div group="get_current_dir" id="get_current_dir-Example" class="tabcontent"  style="display: none;" >

```js
import "std::env" as env;

// Get the current working directory
let current_dir = env::get_current_dir();
print(current_dir); // output: /home/username/project
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> get_env </h2>

```rust,ignore
fn get_env(var: String) -> String
```

<div>
<div class="tab">
<button group="get_env" id="link-get_env-Description"  class="tablinks active" 
    onclick="openTab(event, 'get_env', 'Description')">
Description
</button>
<button group="get_env" id="link-get_env-Arguments"  class="tablinks" 
    onclick="openTab(event, 'get_env', 'Arguments')">
Arguments
</button>
<button group="get_env" id="link-get_env-Returns"  class="tablinks" 
    onclick="openTab(event, 'get_env', 'Returns')">
Returns
</button>
<button group="get_env" id="link-get_env-Example"  class="tablinks" 
    onclick="openTab(event, 'get_env', 'Example')">
Example
</button>
</div>

<div group="get_env" id="get_env-Description" class="tabcontent"  style="display: block;" >
Get the value of an environment variable.
</div>
<div group="get_env" id="get_env-Arguments" class="tabcontent"  style="display: none;" >

* `var` - The name of the environment variable to retrieve.
</div>
<div group="get_env" id="get_env-Returns" class="tabcontent"  style="display: none;" >

This function returns the value of the environment variable as a `String`. 
If the variable is not found or there is an error, it returns a `Result::Err` with the error message.
</div>
<div group="get_env" id="get_env-Example" class="tabcontent"  style="display: none;" >

```js
import "std::env" as env; 

// Get the value of the "HOME" environment variable
let home_dir = env::get_env("HOME");
print(home_dir); // output: /home/username
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> get_home_dir </h2>

```rust,ignore
fn get_home_dir() -> String
```

<div>
<div class="tab">
<button group="get_home_dir" id="link-get_home_dir-Description"  class="tablinks active" 
    onclick="openTab(event, 'get_home_dir', 'Description')">
Description
</button>
<button group="get_home_dir" id="link-get_home_dir-Returns"  class="tablinks" 
    onclick="openTab(event, 'get_home_dir', 'Returns')">
Returns
</button>
<button group="get_home_dir" id="link-get_home_dir-Example"  class="tablinks" 
    onclick="openTab(event, 'get_home_dir', 'Example')">
Example
</button>
</div>

<div group="get_home_dir" id="get_home_dir-Description" class="tabcontent"  style="display: block;" >
Get the path to the home directory.
</div>
<div group="get_home_dir" id="get_home_dir-Returns" class="tabcontent"  style="display: none;" >

This function returns the value of the "HOME" environment variable as a `String`.
If the variable is not found or there is an error, it returns a `Result::Err` with the error message.
</div>
<div group="get_home_dir" id="get_home_dir-Example" class="tabcontent"  style="display: none;" >

```js
import "std::env" as env;

// Get the home directory
let home_dir = env::get_home_dir();
print(home_dir); // output: /home/username
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> get_username </h2>

```rust,ignore
fn get_username() -> String
```

<div>
<div class="tab">
<button group="get_username" id="link-get_username-Description"  class="tablinks active" 
    onclick="openTab(event, 'get_username', 'Description')">
Description
</button>
<button group="get_username" id="link-get_username-Returns"  class="tablinks" 
    onclick="openTab(event, 'get_username', 'Returns')">
Returns
</button>
<button group="get_username" id="link-get_username-Example"  class="tablinks" 
    onclick="openTab(event, 'get_username', 'Example')">
Example
</button>
</div>

<div group="get_username" id="get_username-Description" class="tabcontent"  style="display: block;" >
Get the current username.
</div>
<div group="get_username" id="get_username-Returns" class="tabcontent"  style="display: none;" >

This function returns the value of the "USER" environment variable as a `String`.
If the variable is not found or there is an error, it returns a `Result::Err` with the error message.
</div>
<div group="get_username" id="get_username-Example" class="tabcontent"  style="display: none;" >

```js
import "std::env" as env;

// Get the username of the current user
let username = env::get_username();
print(username); // output: username
```
</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> set_env </h2>

```rust,ignore
fn set_env(var: String, value: String)
```

<div>
<div class="tab">
<button group="set_env" id="link-set_env-Description"  class="tablinks active" 
    onclick="openTab(event, 'set_env', 'Description')">
Description
</button>
<button group="set_env" id="link-set_env-Arguments"  class="tablinks" 
    onclick="openTab(event, 'set_env', 'Arguments')">
Arguments
</button>
<button group="set_env" id="link-set_env-Returns"  class="tablinks" 
    onclick="openTab(event, 'set_env', 'Returns')">
Returns
</button>
<button group="set_env" id="link-set_env-Example"  class="tablinks" 
    onclick="openTab(event, 'set_env', 'Example')">
Example
</button>
</div>

<div group="set_env" id="set_env-Description" class="tabcontent"  style="display: block;" >
Set the value of an environment variable.
</div>
<div group="set_env" id="set_env-Arguments" class="tabcontent"  style="display: none;" >

* `var` - The name of the environment variable to set.
* `value` - The value to assign to the environment variable.
</div>
<div group="set_env" id="set_env-Returns" class="tabcontent"  style="display: none;" >

This function does not return a value.
</div>
<div group="set_env" id="set_env-Example" class="tabcontent"  style="display: none;" >

```js
import "std::env" as env; 

// Set the value of the "MY_VAR" environment variable
env::set_env("MY_VAR", "SomeValue");
```
</div>

</div>
</div>
</br>
