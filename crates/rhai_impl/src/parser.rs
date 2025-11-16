use crate::{
    ast::WidgetNode,
    builtins::register_all_widgets,
    error::{format_eval_error, format_parse_error},
    helper::extract_poll_and_listen_vars,
    module_resolver::SimpleFileResolver,
    providers::register_all_providers,
    updates::ReactiveVarStore,
};
use anyhow::{anyhow, Result};
use rhai::{Dynamic, Engine, ImmutableString, OptimizationLevel, Scope, AST};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

pub struct ParseConfig {
    engine: Engine,
    all_nodes: Rc<RefCell<Vec<WidgetNode>>>,
    keep_signal: Rc<RefCell<Vec<u64>>>,
}

impl ParseConfig {
    pub fn new(pl_handler_store: Option<ReactiveVarStore>) -> Self {
        let mut engine = Engine::new();
        let all_nodes = Rc::new(RefCell::new(Vec::new()));
        let keep_signal = Rc::new(RefCell::new(Vec::new()));

        engine.set_max_expr_depths(128, 128);
        engine
            .set_module_resolver(SimpleFileResolver { pl_handler_store: pl_handler_store.clone() });

        register_all_widgets(&mut engine, &all_nodes, &keep_signal);
        register_all_providers(&mut engine, pl_handler_store);

        Self { engine, all_nodes, keep_signal }
    }

    pub fn compile_code(&mut self, code: &str, file_path: &str) -> Result<AST> {
        let mut ast = self
            .engine
            .compile(code)
            .map_err(|e| anyhow!(format_parse_error(&e, code, Some(file_path))))?;

        ast.set_source(ImmutableString::from(file_path));
        Ok(ast)
    }

    pub fn eval_code_with(
        &mut self,
        code: &str,
        rhai_scope: Option<Scope>,
        compiled_ast: Option<&AST>,
        file_id: Option<&str>,
    ) -> Result<WidgetNode> {
        let mut scope = match rhai_scope {
            Some(s) => s,
            None => Scope::new(),
        };

        // Just eval as node will be in `all_nodes`
        if let Some(ast) = compiled_ast {
            let _ = self
                .engine
                .eval_ast_with_scope::<Dynamic>(&mut scope, &ast)
                .map_err(|e| anyhow!(format_eval_error(&e, code, &self.engine, file_id)))?;
        } else {
            let _ = self
                .engine
                .eval_with_scope::<Dynamic>(&mut scope, code)
                .map_err(|e| anyhow!(format_eval_error(&e, code, &self.engine, file_id)))?;
        };

        // Retain signals
        crate::updates::retain_signals(&self.keep_signal.borrow());

        // Merge all nodes in all_nodes (`enter([])`) into a single root node
        let merged_node = {
            let mut all_nodes_vec = self.all_nodes.borrow_mut();

            let mut merged_children = Vec::new();
            for node in all_nodes_vec.drain(..) {
                match node {
                    WidgetNode::Enter(children) => merged_children.extend(children),
                    // I think that the following line is redundant as
                    // it will be enter(..) 100% of the time. But, what if it
                    // is an empty enter or smth? Idk.. it works... so I'll keep it.
                    other => merged_children.push(other),
                }
            }

            WidgetNode::Enter(merged_children)
        };

        Ok(merged_node.setup_dyn_ids("root"))
    }

    pub fn eval_code_snippet(&mut self, code: &str) -> Result<WidgetNode> {
        let mut scope = Scope::new();

        // Just eval as node will be in `all_nodes`
        let node = self
            .engine
            .eval_with_scope::<WidgetNode>(&mut scope, code)
            .map_err(|e| anyhow!(format_eval_error(&e, code, &self.engine, Some("<dyn eval>"))))?;

        // Retain signals
        crate::updates::retain_signals(&self.keep_signal.borrow());

        // Clear all nodes
        self.all_nodes.borrow_mut().clear();

        Ok(node)
    }

    pub fn code_from_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<String> {
        Ok(fs::read_to_string(&file_path)
            .map_err(|e| anyhow!("Failed to read {:?}: {}", file_path.as_ref(), e))?)
    }

    pub fn initial_poll_listen_scope(code: &str) -> Result<Scope<'_>> {
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

    pub fn call_rhai_fn(&self, ast: &AST, expr: &str, scope: Option<&mut Scope>) -> Result<()> {
        // very naive split
        let (fn_name, args_str) =
            expr.split_once('(').ok_or_else(|| anyhow::anyhow!("Invalid expression: {}", expr))?;
        let fn_name = fn_name.trim();
        let args_str = args_str.trim_end_matches(')');

        // parse args into Dynamics
        let args: Vec<rhai::Dynamic> = args_str
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| {
                let s = s.trim();
                if let Ok(i) = s.parse::<i64>() {
                    rhai::Dynamic::from(i)
                } else if let Ok(f) = s.parse::<f64>() {
                    rhai::Dynamic::from(f)
                } else {
                    rhai::Dynamic::from(s.to_string())
                }
            })
            .collect();

        let mut scope = match scope {
            Some(s) => s,
            None => &mut Scope::new(),
        };

        match self.engine.call_fn::<rhai::Dynamic>(&mut scope, ast, fn_name, args) {
            Ok(result) => {
                log::debug!("Call `{}` returned {:?}", fn_name, result);
                Ok(())
            }
            Err(e) => {
                log::error!("Call `{}` failed: {}", fn_name, e);
                Err(anyhow::anyhow!(e.to_string()))
            }
        }
    }

    pub fn set_opt_level(&mut self, opt_lvl: OptimizationLevel) {
        self.engine.set_optimization_level(opt_lvl);
    }

    pub fn action_with_engine<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Engine) -> R,
    {
        let result = f(&mut self.engine);

        let engine_ptr: *const Engine = &self.engine as *const Engine;
        log::trace!("Engine pointer after closure: {:p}", engine_ptr);

        result
    }
}
