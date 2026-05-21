// -- 1. compile code -- (maybe not)
// 2. eval_code_with
// 3. eval_code_snippet
// --  4. register_poll_listen_globals -- (maybe not)
// 5. call_nbcl_function
use nbcl::{NbclEngine, context::Context};
use crate::{builtins, errors, translate};
use ewwii_shared_utils::ast::WidgetNode;
use anyhow::{anyhow, Result};

pub struct NbclConfigParser {
    pub engine: NbclEngine,
    pub ctx: Option<Context>,
}

impl NbclConfigParser {
    pub fn new() -> Self {
        let mut engine = NbclEngine::new();

        builtins::register_all_nodes(&mut engine);

        Self {
            engine,
            ctx: None,
        }
    }

    pub fn eval_code(&mut self, code: &str, file_id: Option<&str>) -> Result<WidgetNode> {
        let source_ast = self.engine.parse_str(&code)
            .map_err(|e| anyhow!(errors::handle_nbcl_err(e, code, file_id)))?;
        let (tree, ctx) = self.engine.evaluate_ast_for_ctx(source_ast)
            .map_err(|e| anyhow!(errors::handle_nbcl_err(e, code, file_id)))?;

        // set the context
        self.ctx = Some(ctx);

        // translate the tree
        let wnode = WidgetNode::Tree(translate::to_widgetnode(tree.root_nodes)?);
        Ok(wnode.setup_dyn_ids("root"))
    }

    pub fn extension(&self) -> String {
        String::from("nbcl")
    }

    pub fn main_file(&self) -> String {
        String::from("ewwii.nbcl")
    }
}
