use crate::builtins::register_all_widgets;
use crate::error::format_rhai_error;
use crate::widgetnode::WidgetNode;
use anyhow::{anyhow, Result};
use rhai::{Engine, Scope};
use std::fs;
use std::path::Path;

pub struct ParseConfig {
    engine: Engine,
    scope: Scope<'static>,
}

impl ParseConfig {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        let scope = Scope::new();
        engine.set_max_expr_depths(128, 128);
        register_all_widgets(&mut engine);
        // register_all_variables(&mut scope);
        Self { engine, scope }
    }

    pub fn parse_widget_code(&mut self, code: &str) -> Result<WidgetNode> {
        self.engine
            .eval_with_scope::<WidgetNode>(&mut self.scope, code)
            .map_err(|e| anyhow!(format_rhai_error(&e, code)))
    }

    pub fn parse_widget_from_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<WidgetNode> {
        let code = fs::read_to_string(&file_path).map_err(|e| anyhow!("Failed to read {:?}: {}", file_path.as_ref(), e))?;
        self.parse_widget_code(&code)
    }
}
