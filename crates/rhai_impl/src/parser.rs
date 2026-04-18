use crate::{
    builtins::register_all_widgets,
    error::{format_eval_error, format_parse_error},
    helper::extract_poll_and_listen_vars,
    module_resolver::SimpleFileResolver,
    providers::register_all_providers,
};
use ewwii_shared_utils::ast::WidgetNode;
use ewwii_shared_utils::prop::Property;
use anyhow::{anyhow, Result};
use rhai::{Dynamic, Engine, ImmutableString, Module, Scope, AST, FuncArgs};
use ewwii_shared_utils::variables::GlobalVar;
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    pub static EWWII_CONFIG_AST: RefCell<Option<AST>> = RefCell::new(None);
}

pub struct RhaiParseConfig {
    pub engine: Engine,
    all_nodes: Rc<RefCell<Vec<WidgetNode>>>,
}

impl RhaiParseConfig {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        let all_nodes = Rc::new(RefCell::new(Vec::new()));

        engine.set_max_expr_depths(128, 128);
        engine.set_module_resolver(SimpleFileResolver);

        register_all_widgets(&mut engine, &all_nodes);
        register_all_providers(&mut engine);

        Self { engine, all_nodes }
    }

    pub fn compile_code(&mut self, code: &str, file_id: Option<&str>) -> Result<AST> {
        let file_id = file_id.unwrap_or("<rhai>");

        let mut ast = self
            .engine
            .compile(code)
            .map_err(|e| anyhow!(format_parse_error(&e, code, Some(file_id))))?;

        ast.set_source(ImmutableString::from(file_id));

        EWWII_CONFIG_AST.with(|p| {
            *p.borrow_mut() = Some(ast.clone());
        });

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
            let ast = self.compile_code(code, file_id)?;

            let _ = self
                .engine
                .eval_ast_with_scope::<Dynamic>(&mut scope, &ast)
                .map_err(|e| anyhow!(format_eval_error(&e, code, &self.engine, file_id)))?;
        };

        // Merge all nodes in all_nodes (`tree([])`) into a single root node
        let merged_node = {
            let mut all_nodes_vec = self.all_nodes.borrow_mut();

            let mut merged_children = Vec::new();
            for node in all_nodes_vec.drain(..) {
                match node {
                    WidgetNode::Tree(children) => merged_children.extend(children),
                    // I think that the following line is redundant as
                    // it will be tree(..) 100% of the time. But, what if it
                    // is an empty tree or smth? Idk.. it works... so I'll keep it.
                    other => merged_children.push(other),
                }
            }

            WidgetNode::Tree(merged_children)
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

        // Clear all nodes
        self.all_nodes.borrow_mut().clear();

        Ok(node)
    }

    pub fn register_poll_listen_globals(&mut self, code: &str) -> Result<()> {
        let mut global_module = Module::new();
        self.engine.register_type::<GlobalVar>();

        for (name, initial) in extract_poll_and_listen_vars(code)? {
            let value = match initial {
                Some(v) => v.into(),
                None => Property::None,
            };
            let glob_var = GlobalVar::from(name.clone(), value);
            global_module.set_var(name, glob_var);
        }

        self.engine.register_global_module(global_module.into());
        Ok(())
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

    pub fn call_callback<T>(
        &self, 
        handle: u64,
        args: impl FuncArgs
    ) -> Result<T, Box<rhai::EvalAltResult>>
    where 
        T: rhai::Variant + Clone,
    {
        let fnptr = crate::callback::get_callback(handle)
            .ok_or_else(|| format!("Callback handle {} not found in registry", handle))?;

        EWWII_CONFIG_AST.with(|ast_cell| {
            let ast_ref = ast_cell.borrow();

            let ast = match ast_ref.as_ref() {
                Some(a) => a,
                None => return Err("AST not initialized".into()),
            };

            fnptr.call(&self.engine, ast, args)
        })
    }

    pub fn extension(&self) -> String {
        String::from("rhai")
    }

    pub fn main_file(&self) -> String {
        String::from("ewwii.rhai")
    }
}
