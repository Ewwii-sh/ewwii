use crate::{builtins, errors, libraries, translate};
use anyhow::{anyhow, Result};
use ewwii_shared_utils::ast::WidgetNode;
use nbcl::{context::Context, NbclEngine, Value};

pub struct NbclConfigParser {
    pub engine: NbclEngine,
    pub ctx: Option<Context>,
}

impl NbclConfigParser {
    pub fn new() -> Self {
        let mut engine = NbclEngine::new();

        builtins::register_all_nodes(&mut engine);
        builtins::register_all_fns(&mut engine);

        libraries::register_api_lib(&mut engine);
        libraries::register_core_lib(&mut engine);

        Self { engine, ctx: None }
    }

    pub fn eval_code(&mut self, code: &str, file_id: Option<&str>) -> Result<WidgetNode> {
        let source_ast = self
            .engine
            .parse_str(&code)
            .map_err(|e| anyhow!(errors::handle_nbcl_err(e, code, file_id)))?;
        let (tree, ctx) = self
            .engine
            .evaluate_ast_for_ctx(source_ast)
            .map_err(|e| anyhow!(errors::handle_nbcl_err(e, code, file_id)))?;

        // set the context
        self.ctx = Some(ctx);

        // translate the tree
        let wnode = WidgetNode::Tree(translate::to_widgetnode(tree.root_nodes)?);
        Ok(wnode.setup_dyn_ids("root"))
    }

    pub fn eval_code_snippet(&mut self, code: &str) -> Result<WidgetNode> {
        let tree = self
            .engine
            .evaluate(code)
            .map_err(|e| anyhow!(errors::handle_nbcl_err(e, code, Some("<dyn_eval>"))))?;

        // translate the tree
        let mut all_nodes = translate::to_widgetnode(tree.root_nodes)?;

        if all_nodes.len() <= 1 {
            anyhow::bail!("Snippet must resolve to exactly 1 widget.");
        }

        let node = all_nodes.remove(0);
        Ok(node.setup_dyn_ids("root"))
    }

    pub fn call_nbcl_function(&self, expr: &str) -> Result<()> {
        let Some(ref ctx) = self.ctx else {
            anyhow::bail!("Nbcl context not found.");
        };

        let (fn_name, args_str) =
            expr.split_once('(').ok_or_else(|| anyhow::anyhow!("Invalid expression: {}", expr))?;
        let fn_name = fn_name.trim();
        let args_str = args_str.trim_end_matches(')');

        let args: Vec<Value> = args_str
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| {
                let s = s.trim();
                if let Ok(i) = s.parse::<i64>() {
                    Value::Int(i)
                } else if let Ok(f) = s.parse::<f64>() {
                    Value::Float(f)
                } else {
                    Value::Str(s.to_string())
                }
            })
            .collect();

        self.engine.call_function(fn_name, args, ctx)?;

        Ok(())
    }

    pub fn extension(&self) -> String {
        String::from("nbcl")
    }

    pub fn main_file(&self) -> String {
        String::from("ewwii.nbcl")
    }
}
