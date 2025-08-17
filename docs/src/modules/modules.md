# Modules

Modules undoubtedly are one of the most powerful features in Rhai. They provide infinite extensibility to ewwii.

Every module follows the syntax:

```rust,ignore
import "std::env" as env;
let home = env::get_home_dir(); // returns `$HOME` env var value
```

This allows you to write expressive, modular Rhai code with functions grouped logically under `std` or custom namespaces.

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

## Future Plans

Other modules coming soon under `std`:

-   `std::fs` — Filesystem operations (e.g., `read_file`, `write_file`, `list_dir`)
-   `std::path` — Path helpers (e.g., `join`, `basename`, `dirname`)
-   `std::time` — Time utilities (e.g., `now`, `sleep`, `format_time`)
-   `std::math` — Numeric functions (e.g., `clamp`, `lerp`, `map_range`)
-   `std::color` — Color parsing and manipulation (e.g., `hex_to_rgb`, `blend`)

You can easily extend or override these by adding `.rhai` modules in your config path.
