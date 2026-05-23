# ewwii_plugin_api

`ewwii_plugin_api` provides the core traits, types, and abstractions
that bridge the **ewwii** host and its plugins.

This crate provides a safe and easy plugin development API
through a simple and flexible interface for cross-boundary communication.

## Usage

There are two ways to define a plugin: the **Recommended Macro** for
standard plugins, and the **Manual Implementation** for full control.

### 1. Recommended: Using `auto_plugin!`
For most use cases, the macro handles the boilerplate of exporting
symbols and implementing traits.

```rust
use ewwii_plugin_api::{auto_plugin, PluginInfo};

auto_plugin!(
    DummyStructure,
    PluginInfo::new("com.app.example", "1.0.0"),
    host,
    {
        host.log("Plugin says Hello!");
    }
);
```

### 2. Advanced: Manual Implementation
Use this approach if your plugin needs to maintain internal state,
implement additional traits, or manage complex lifetimes.

```rust
use ewwii_plugin_api::{EwwiiAPI, Plugin, PluginInfo, export_plugin};

#[derive(Default)]
pub struct MyPlugin {
    count: std::sync::atomic::AtomicUsize,
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> PluginInfo {
        PluginInfo::new("com.app.example", "1.0.0")
    }

    fn init(&self, host: &dyn EwwiiAPI) {
        host.log("Manual plugin initialized.");
    }
}

// This macro exports the C-compatible symbols required by the host loader
export_plugin!(MyPlugin);
```
