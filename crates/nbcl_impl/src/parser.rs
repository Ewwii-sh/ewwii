use crate::{builtins, errors, libraries, translate};
use anyhow::{anyhow, Result};
use ewwii_plugin_api::IpcRequest;
use ewwii_shared_utils::ast::WidgetNode;
use ewwii_shared_utils::prop::Callback;
use nbcl::{context::EvalContext, NbclEngine, Value};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
pub struct NbclConfigParser {
    pub engine: NbclEngine,
    pub ctx: Option<EvalContext>,
}

impl NbclConfigParser {
    pub fn new(ipc_tx: UnboundedSender<IpcRequest>) -> Self {
        let mut engine = NbclEngine::new();

        builtins::register_all_nodes(&mut engine);
        builtins::register_all_fns(&mut engine, ipc_tx);

        libraries::register_api_lib(&mut engine);
        libraries::register_core_lib(&mut engine);

        Self { engine, ctx: None }
    }

    pub fn eval_code(&mut self, code: &str, file_id: Option<&str>) -> Result<WidgetNode> {
        let mut eval_ctx = EvalContext::from(&self.engine);

        let source_ast = self
            .engine
            .parse_str(code)
            .map_err(|e| anyhow!(errors::handle_nbcl_err(e, code, file_id, None)))?;

        let tree = self.engine.eval_ast_with_eval_ctx(source_ast, &mut eval_ctx).map_err(|e| {
            anyhow!(errors::handle_nbcl_err(e, code, file_id, Some(eval_ctx.clone())))
        })?;

        // set the context
        self.ctx = Some(eval_ctx);

        // translate the tree
        let wnode = WidgetNode::Tree(translate::to_widgetnode(tree.root_nodes)?);
        Ok(wnode.setup_dyn_ids("root"))
    }

    pub fn eval_code_snippet(&mut self, code: &str) -> Result<WidgetNode> {
        let tree = self
            .engine
            .evaluate(code)
            .map_err(|e| anyhow!(errors::handle_nbcl_err(e, code, Some("<dyn_eval>"), None)))?;

        // translate the tree
        let mut all_nodes = translate::to_widgetnode(tree.root_nodes)?;

        if all_nodes.len() != 1 {
            anyhow::bail!("Snippet must resolve to exactly 1 widget, but found {}", all_nodes.len());
        }

        let node = all_nodes.remove(0);
        Ok(node.setup_dyn_ids("root"))
    }

    pub fn run_nbcl_expr(&self, expr: &str) -> Result<()> {
        let Some(ref eval_ctx) = self.ctx else {
            anyhow::bail!("Nbcl evaluation context not found.");
        };

        let source_ast = self
            .engine
            .parse_str(expr)
            .map_err(|e| anyhow!(errors::handle_nbcl_err(e, expr, Some("<expr>"), None)))?;

        let mut tmp_ectx = eval_ctx.clone();

        self.engine.eval_ast_with_eval_ctx(source_ast, &mut tmp_ectx).map_err(|e| {
            anyhow!(errors::handle_nbcl_err(e, expr, Some("<expr>"), Some(tmp_ectx.clone())))
        })?;

        Ok(())
    }

    pub fn handle_callback(&self, callback: &Callback) {
        let Some(handle) = &callback.handle else {
            log::error!("Callback handle is missing!");
            return;
        };
        let name = &callback.name;
        let Some(ctx) = &self.ctx else {
            log::error!("Evaluation context not found for callback!");
            return;
        };

        match handle.as_ref() {
            "<mutate>" => {
                let Some(data_vec) = &callback.data else {
                    log::error!("Data is required for <mutate>");
                    return;
                };
                let sig_val = Value::Str(data_vec[0].clone());

                match self.engine.call_function(name, vec![sig_val], ctx) {
                    Ok(value) => {
                        if let Some(ret) = &callback.ret {
                            let Value::Str(value_str) = value else {
                                log::error!("Return value of mutate closure/lambda must be a string.");
                                return;
                            };

                            *ret.borrow_mut() = value_str;
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to call function: {}", e)
                    }
                }
            }
            "<script>" => {
                if let Err(e) = self.engine.call_function(
                    name,
                    vec![Value::Object("WidgetCtrl".into(), Box::new(Value::Str(String::new())))],
                    ctx,
                ) {
                    log::error!("Failed to call function: {}", e);
                }
            }
            _ => {
                log::warn!("Known callback. Calling without any paramters.");
                if let Err(e) = self.engine.call_function(
                    name,
                    vec![],
                    ctx,
                ) {
                    log::error!("Failed to call function: {}", e);
                }
            }
        }
    }

    pub fn extension(&self) -> String {
        String::from("nbcl")
    }

    pub fn main_file(&self) -> String {
        String::from("ewwii.nbcl")
    }
}
