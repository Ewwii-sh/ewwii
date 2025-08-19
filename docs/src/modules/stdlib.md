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

## Future Plans

Other modules coming soon under `std`:

-   `std::fs` — Filesystem operations (e.g., `read_file`, `write_file`, `list_dir`)
-   `std::path` — Path helpers (e.g., `join`, `basename`, `dirname`)
-   `std::time` — Time utilities (e.g., `now`, `sleep`, `format_time`)
-   `std::math` — Numeric functions (e.g., `clamp`, `lerp`, `map_range`)
-   `std::color` — Color parsing and manipulation (e.g., `hex_to_rgb`, `blend`)

You can easily extend or override these by adding `.rhai` modules in your config path.
