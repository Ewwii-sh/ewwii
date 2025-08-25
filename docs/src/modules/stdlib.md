# Standard Library

## `std::env`

The `std::env` module provides access to common system-level environment queries. It is supported on Unix-based systems (Linux, macOS).

### Usage

```rust,ignore
import "std::env" as env;

// Get an environment variable, or fallback to a default
let shell = env::get_env("SHELL") ?? "unknown";

// Set an environment variable (current process only)
env::set_env("DEBUG_MODE", "true");

// Get the user's home directory
let home = env::get_home_dir() ?? "/home/user";

// Get the current working directory
let cwd = env::get_current_dir() ?? "/";

// Get the current username
let user = env::get_username() ?? "nobody";
```

### Functions

| Function          | Description                                             |
| ----------------- | ------------------------------------------------------- |
| `get_env`         | Gets an environment variable's value                    |
| `set_env`         | Sets an environment variable (current process only)     |
| `get_home_dir`    | Returns the current user's home directory path if found |
| `get_current_dir` | Returns the current working directory                   |
| `get_username`    | Gets the current user's username from `$USER`           |

## `std::text`

The `std::text` module provides access to more string manipulation that Rhai lacks.

### Usage

```rust,ignore
import "std::text" as text;

// Convert a string to a URL-friendly slug
let slug = text::to_slug("Ewwii is cool!"); // output: "ewwii-is-cool"

// Convert a string to camelCase
let camel = text::to_camel_case("my cool project"); // output: "myCoolProject"

// Truncate a string to N characters (without splitting in the middle of a character)
let short = text::truncate_chars("hello world", 5); // output: "hello"

// Convert a string to uppercase
let upper = text::to_upper("hello"); // output: "HELLO"

// Convert a string to lowercase
let lower = text::to_lower("HELLO"); // output: "hello"
```

### Functions

| Function         | Description                                                            |
| ---------------- | ---------------------------------------------------------------------- |
| `to_slug`        | Converts a string into a lowercase, hyphen-separated slug              |
| `to_camel_case`  | Converts a string into camelCase, removing non-alphanumeric characters |
| `truncate_chars` | Truncates a string to a maximum number of characters (UTF-8 safe)      |
| `to_upper`       | Converts a string to uppercase                                         |
| `to_lower`       | Converts a string to lowercase                                         |

## `std::monitor`

The `std::monitor` module provides utilities for querying information about connected monitors, including their resolution, dimensions, and DPI.

### Usage

```rust,ignore
import "std::monitor" as monitor;

// Get number of monitors
let count = monitor::count(); // e.g., 2

// Get resolution of the primary monitor
let (w, h) = monitor::primary_resolution();
let res_str = monitor::primary_resolution_str(); // e.g., "1920x1080"

// Get resolutions of all monitors
let all_res = monitor::all_resolutions();
let all_res_str = monitor::all_resolutions_str(); // e.g., "1920x1080, 1280x1024"

// Get dimensions of a specific monitor
let (x, y, w, h) = monitor::dimensions(0);
let dim_str = monitor::dimensions_str(0); // e.g., "1920x1080"

// Get DPI of a monitor
let dpi = monitor::dpi(0);
let dpi_str = monitor::dpi_str(0); // e.g., "96.0"
```

### Functions

| Function                   | Description                                                                                   |
| -------------------------- | --------------------------------------------------------------------------------------------- |
| `count()`                  | Returns the number of connected monitors.                                                     |
| `primary_resolution()`     | Returns the width and height of the primary monitor as a tuple `(width, height)`.             |
| `primary_resolution_str()` | Returns the primary monitor resolution as a string in the format `"WIDTHxHEIGHT"`.            |
| `all_resolutions()`        | Returns a vector of `(width, height)` tuples for all connected monitors.                      |
| `all_resolutions_str()`    | Returns a comma-separated string of all monitor resolutions in `"WIDTHxHEIGHT"` format.       |
| `dimensions(index)`        | Returns `(x, y, width, height)` for the monitor at the given index.                           |
| `dimensions_str(index)`    | Returns the dimensions of the monitor at the given index as a formatted string `"x,y - WxH"`. |
| `dpi(index)`               | Returns the DPI (dots per inch) of the monitor at the given index, accounting for scaling.    |
| `dpi_str(index)`           | Returns the DPI as a formatted string with one decimal place, e.g., `"96.0"`.                 |

### Notes

-   Monitor indices are zero-based: the primary monitor is index 0.
-   DPI calculation assumes a base of 96 DPI multiplied by the monitor’s scale factor.
-   The module automatically initializes GTK if it hasn’t been initialized on the main thread.

## `std::json`

The `std::json` module provides utilities for working with JSON data within Rhai scripts. It allows parsing, serializing, and manipulating JSON objects dynamically.

### Usage

```rust,ignore
import "std::json" as json;

// Parse a JSON string
let json_val = json::parse_json(r#"{"name":"Alice","age":30}"#);

// Convert JSON back to string
let json_str = json::to_string(json_val);

// Get a value from a JSON object
let name = json::get(json_val, "name"); // "Alice"

// Set a value in a JSON object
json::set(json_val, "age", 31);
```

### Functions

| Function       | Description                                                                                                       |
| -------------- | ----------------------------------------------------------------------------------------------------------------- |
| `parse_json()` | Parses a JSON string into a Rhai `Dynamic` representing a `serde_json::Value`. Returns an error if parsing fails. |
| `to_string()`  | Serializes a `Dynamic` JSON value back into a JSON string.                                                        |
| `get()`        | Retrieves a value by key from a JSON object. Returns `()` if the key does not exist.                              |
| `set()`        | Sets a key-value pair in a JSON object. Returns an error if the value is not a JSON object.                       |

### Notes

-   All JSON values are represented as Rhai `Dynamic` objects internally.
-   Keys that do not exist in a JSON object return a `UNIT` value.
-   `set()` only works on JSON objects; trying to set a key on a non-object JSON value will produce an error.
-   Parsing and serialization errors are returned as Rhai `EvalAltResult` errors.

## `std::math`

The `std::math` module provides a collection of mathematical constants and functions.
It includes basic arithmetic, trigonometry, exponentiation, logarithms, and utility functions.

### Usage

```rust,ignore
import "std::math" as math;

// Constants
print(math::PI); // 3.14159...
print(math::E); // 2.71828...
print(math::TAU); // 6.28318...

// Basic math
let x = math::abs(-42.0); // 42
let y = math::sqrt(9.0); // 3
let z = math::pow(2.0, 10.0); // 1024

// Trigonometry
let s = math::sin(math::PI / 2); // ~1
let c = math::cos(0.0); // 1
let t = math::tan(math::PI / 4); // ~1

// Exponentials & logs
let e = math::exp(1.0); // ~2.718
let l = math::ln(math::E); // 1
let l10 = math::log10(100.0); // 2
let l2 = math::log(2.0, 8.0); // 3

// Inverse trig
let a = math::asin(1.0); // PI/2
let b = math::acos(0.0); // PI/2
let c = math::atan(1.0); // PI/4
let d = math::atan2(1.0, 1.0); // PI/4

// Hyperbolic
let sh = math::sinh(1.0);
let ch = math::cosh(1.0);
let th = math::tanh(1.0);

// Utilities
let f = math::floor(3.7); // 3
let r = math::round(3.5); // 4
let m = math::min(10.0, 20.0); // 10
let M = math::max(10.0, 20.0); // 20
let cl = math::clamp(15.0, 0.0, 10.0); // 10

// Other Utilities
let tof = math::to_float(42);   // 42 -> 42.0
let toi = math::to_int(3.14);   // truncates toward zero -> 3
// NOTE: to_int does NOT round!
// If you want nearest integer, use: to_int(math::round(3.14))
```

### Constants

| Constant | Value    | Description      |
| -------- | -------- | ---------------- |
| `PI`     | 3.14159… | Circle ratio π   |
| `E`      | 2.71828… | Euler’s number   |
| `TAU`    | 6.28318… | Full circle (2π) |

### Functions

| Function             | Description                                       |
| -------------------- | ------------------------------------------------- |
| `abs(x)`             | Absolute value of `x`                             |
| `sqrt(x)`            | Square root of `x`                                |
| `pow(base, exp)`     | Base raised to power of exponent                  |
| `sin(x)`             | Sine of `x` (radians)                             |
| `cos(x)`             | Cosine of `x` (radians)                           |
| `tan(x)`             | Tangent of `x` (radians)                          |
| `exp(x)`             | e raised to the power `x`                         |
| `ln(x)`              | Natural log of `x`                                |
| `log10(x)`           | Base-10 log of `x`                                |
| `log(base, x)`       | Log of `x` in `base`                              |
| `asin(x)`            | Inverse sine                                      |
| `acos(x)`            | Inverse cosine                                    |
| `atan(x)`            | Inverse tangent                                   |
| `atan2(y, x)`        | Arctangent of `y/x` considering quadrant          |
| `sinh(x)`            | Hyperbolic sine                                   |
| `cosh(x)`            | Hyperbolic cosine                                 |
| `tanh(x)`            | Hyperbolic tangent                                |
| `floor(x)`           | Round down                                        |
| `ceil(x)`            | Round up                                          |
| `round(x)`           | Round to nearest                                  |
| `trunc(x)`           | Round toward zero                                 |
| `fract(x)`           | Fractional part                                   |
| `min(a, b)`          | Smaller of two values                             |
| `max(a, b)`          | Larger of two values                              |
| `clamp(x, min, max)` | Clamp `x` into `[min, max]`                       |
| `to_float`           | Convert an integer or float into a floating-point |
| `to_int`             | Convert an integer or float into an integer       |

### Note

All functions in this module work with floating-point numbers (f64).

If you pass an integer (e.g. `0`), Rhai will report an error because there is no math::cos(i64). Use a floating-point literal instead (e.g. `0.0`).

All math functions return `f64`. If you need an integer result, use `to_int` to convert.

## `std::command`

The `std::command` module provides functions which you can use to run shell commands on your system.

### Usage

```rust,ignore
import "std::command" as command;

// run a command
command::run("notify-send Hello!");

// run a command and read output from stdout
let output = command::run_and_read("pwd"); // example output: /home/foo/.config/ewwii/
```

### Functions

| Function          | Description                                        |
| ----------------- | -------------------------------------------------- |
| `run(x)`          | Run the shell command in `x`                       |
| `run_and_read(x)` | Run the shell command in `x` and return the stdout |

### Note

The functions in `std::command` execute arbitrary shell commands. Only run scripts you trust, as misuse can compromise your system.

This, along with features like `poll`, `listen`, `onclick`, `onhover`, etc., which also run shell commands, can be abused by bad actors. Always verify and trust a package before installing it via [eiipm](https://github.com/Ewwii-sh/eiipm). Even if a package is registered in [eii-manifests](https://github.com/Ewwii-sh/eii-manifests), bad actors could change the code of their package without the awareness of maintainers.
