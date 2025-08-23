use crate::{
    builtins::register_all_widgets,
    error::{format_eval_error, format_parse_error},
    helper::extract_poll_and_listen_vars,
    module_resolver::SimpleFileResolver,
    providers::register_all_providers,
    widgetnode::WidgetNode,
};
use anyhow::{anyhow, Result};
use rhai::{Dynamic, Engine, Scope, AST};
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
        engine.set_module_resolver(SimpleFileResolver);
        register_all_widgets(&mut engine);
        register_all_providers(&mut engine);

        Self { engine }
    }

    pub fn eval_code(&mut self, code: &str) -> Result<WidgetNode> {
        let mut scope = Scope::new();
        self.engine
            .eval_with_scope::<WidgetNode>(&mut scope, code)
            .map_err(|e| anyhow!(format_eval_error(&e, code, &self.engine)))
    }

    pub fn compile_code(&mut self, code: &str) -> Result<AST> {
        self.engine.compile(code).map_err(|e| anyhow!(format_parse_error(&e, code)))
    }

    pub fn eval_code_with(
        &mut self,
        code: &str,
        rhai_scope: Option<Scope>,
        compiled_ast: Option<&AST>,
    ) -> Result<WidgetNode> {
        let mut scope = match rhai_scope {
            Some(s) => s,
            None => Scope::new(),
        };

        match compiled_ast {
            Some(ast) => self
                .engine
                .eval_ast_with_scope::<WidgetNode>(&mut scope, &ast)
                .map_err(|e| anyhow!(format_eval_error(&e, code, &self.engine))),
            None => self
                .engine
                .eval_with_scope::<WidgetNode>(&mut scope, code)
                .map_err(|e| anyhow!(format_eval_error(&e, code, &self.engine))),
        }
    }

    pub fn code_from_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<String> {
        Ok(fs::read_to_string(&file_path)
            .map_err(|e| anyhow!("Failed to read {:?}: {}", file_path.as_ref(), e))?)
    }

    pub fn initial_poll_listen_scope(code: &str) -> Result<Scope> {
        // Setting the initial value of poll/listen
        let mut scope = Scope::new();
        for (var, initial) in extract_poll_and_listen_vars(code)? {
            let value = match initial {
                Some(val) => Dynamic::from(val),
                None => Dynamic::UNIT,
            };
            scope.set_value(var, value);
        }

        Ok(scope)
    }
}
