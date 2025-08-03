use crate::{
    builtins::register_all_widgets, error::format_rhai_error, helper::extract_poll_and_listen_vars,
    providers::register_all_providers, widgetnode::WidgetNode,
};
use anyhow::{anyhow, Result};
use rhai::{Dynamic, Engine, Scope};
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
        register_all_providers(&mut engine);

        Self { engine, scope }
    }

    pub fn parse_widget_code(&mut self, code: &str) -> Result<WidgetNode> {
        for var in extract_poll_and_listen_vars(code) {
            self.scope.set_value(var, Dynamic::UNIT);
        }

        self.engine.eval_with_scope::<WidgetNode>(&mut self.scope, code).map_err(|e| anyhow!(format_rhai_error(&e, code)))
    }

    pub fn parse_widget_from_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<WidgetNode> {
        let code = fs::read_to_string(&file_path).map_err(|e| anyhow!("Failed to read {:?}: {}", file_path.as_ref(), e))?;
        self.parse_widget_code(&code)
    }
}
