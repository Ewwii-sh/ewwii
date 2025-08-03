use crate::{
    builtins::register_all_widgets, error::format_rhai_error, helper::extract_poll_and_listen_vars,
    providers::register_all_providers, widgetnode::WidgetNode,
};
use anyhow::{anyhow, Result};
use rhai::{Dynamic, Engine, Scope, EvalAltResult};
use std::fs;
use std::path::Path;

pub struct ParseConfig {
    engine: Engine,
    // scope: Scope<'a>,
}

impl ParseConfig {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        // let scope = Scope::new();

        engine.set_max_expr_depths(128, 128);
        register_all_widgets(&mut engine);
        register_all_providers(&mut engine);

        Self { engine }
    }

    pub fn parse_widget_code(&mut self, code: &str) -> Result<WidgetNode> {
        /// Setting the initial value of poll/listen
        let mut scope = Scope::new();
        for (var, initial) in extract_poll_and_listen_vars(code)? {
            let value = match initial {
                Some(val) => Dynamic::from(val),
                None => Dynamic::UNIT,
            };
            scope.set_value(var, value);
        }

        self.engine
            .eval_with_scope::<WidgetNode>(&mut scope, code)
            .map_err(|e| anyhow!(format_rhai_error(&e, code)))
    }

    pub fn parse_widget_from_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<WidgetNode> {
        let code = fs::read_to_string(&file_path).map_err(|e| anyhow!("Failed to read {:?}: {}", file_path.as_ref(), e))?;
        self.parse_widget_code(&code)
    }
}
