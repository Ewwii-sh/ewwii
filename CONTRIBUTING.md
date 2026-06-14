# Contributing to ewwii

Contribution does not always have to be code contribution. Any form of contribution is highly appreciated. Even if you can't contribute, making your ewwii configuration and sharing it with the community is also a great way to help the project out!

## Code Contribution

### General things to keep in mind:

- Run `cargo fmt` for formatting your code.
- Note down your changes in `CHANGELOG.md` once you're done.
- Ensure the code compiles correctly.

### Codebase

- `crates/ewwii`: Core of ewwii (ipc, daemon, options, signal handlers, gtk4, etc.)
- `crates/plugin_api`: Plugin API (used by both ewwii and plugin for communication)
- `crates/nbcl_impl`: Nbcl implementation (parsing, modules)
- `crates/shared_utils`: Utility functions shared between nbcl, plugins, and ewwii (spans, helpers)

## Other Contribution

If are not a programmer but want to help ewwii out, then you can contribute in one of the following ways:

- Contributing to [documentation](https://github.com/ewwii-sh/docs).
- Contributing to [website](https://github.com/ewwii-sh/ewwii-sh.github.io).
- Writing plugins for ewwii.
- Making your ewwii configuration and sharing it with the community.
