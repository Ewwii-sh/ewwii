use rhai::{Engine, Scope};
use crate::builtins::{register_all_widgets, register_all_variables};
use crate::widgetnode::WidgetNode;
use std::fs;
use std::path::Path;
use anyhow::{Result, anyhow};

pub struct ParseConfig {
    engine: Engine,
    scope: Scope<'static>,
}

impl ParseConfig {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        let mut scope = Scope::new();
        engine.set_max_expr_depths(128, 128);
        register_all_widgets(&mut engine);
        // register_all_variables(&mut scope);
        Self {
            engine,
            scope,
        }
    }

    pub fn parse_widget_code(&mut self, code: &str) -> Result<WidgetNode> {
        self.engine
            .eval_with_scope::<WidgetNode>(&mut self.scope, code)
            .map_err(|e| {
                let mut msg = format!("Rhai eval error: {}", e);
                if let Some(pos) = e.position().line() {
                    msg.push_str(&format!(
                        " at line {}, col {}",
                        pos,
                        e.position().position().unwrap_or(0)
                    ));
                }
                anyhow!(msg)
            })
    }

    pub fn parse_widget_from_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<WidgetNode> {
        let code = fs::read_to_string(&file_path)
            .map_err(|e| anyhow!("Failed to read {:?}: {}", file_path.as_ref(), e))?;
        self.parse_widget_code(&code)
    }
}
