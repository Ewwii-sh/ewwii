# Contributing to ewwii

**General things to keep in mind:**

-   Run `cargo fmt` for formatting your code.
-   Always do PR (Pull Request) to iidev branch (especially if its a doc change).

## Codebase

-   `crates/ewwii`: Core of ewwii (ipc, daemon, options, rt engine, gtk, etc.)
-   `crates/iirhai`: Rhai implementation (parsing, modules, poll/listen handlers)
-   `crates/shared_utils`: Utility functions shared between rhai and ewwii (spans, helpers)
