# ewwii_plugin_api

A shared interface providing traits for building plugins for ewwii.

## Example

A simple example showing how to use this interface is provided below:

```rust
use ewwii_plugin_api::{EwwiiAPI, Plugin};

pub struct DummyStructure;

impl Plugin for DummyStructure {
	// critical for ewwii to launch the plugin
    fn init(&self, host: &dyn EwwiiAPI) {
        // will be printed by the host
        host.log("Plugin says Hello!");
        host.rhai_engine_action(Box::new(|engine| {
            let ast = engine.compile("1+1");
            println!("Compiled AST: {:#?}", ast);
        }));
    }
}

// Critical for ewwii to see the plugin
#[unsafe(no_mangle)]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(DummyStructure)
}
```