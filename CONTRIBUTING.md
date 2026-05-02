# Contributing to ewwii

**General things to keep in mind:**

- Run `cargo fmt` for formatting your code.
- Note down your changes in `CHANGELOG.md` once you're done.
- Ensure the code compiles correctly.

## Codebase

- `crates/ewwii`: Core of ewwii (ipc, daemon, options, globalvar handlers, gtk, etc.)
- `crates/plugin_api`: Plugin API (used by both ewwii and plugin for communication)
- `crates/rhai_impl`: Rhai implementation (parsing, modules)
- `crates/shared_utils`: Utility functions shared between rhai and ewwii (spans, helpers)
