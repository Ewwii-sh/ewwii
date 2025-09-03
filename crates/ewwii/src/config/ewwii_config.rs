use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::rc::Rc;
use crate::{
    // ipc_server,
    // error_handling_ctx,
    paths::EwwPaths,
    window::backend_window_options::BackendWindowOptionsDef,
};

use iirhai::{parser::ParseConfig, ast::WidgetNode};
use rhai::{Map, AST};

// use tokio::{net::UnixStream, runtime::Runtime, sync::mpsc};

/// Load an [`EwwiiConfig`] from the config dir of the given [`crate::EwwPaths`],
/// resetting and applying the global YuckFiles object in [`crate::error_handling_ctx`].
pub fn read_from_ewwii_paths(eww_paths: &EwwPaths) -> Result<EwwiiConfig> {
    EwwiiConfig::read_from_dir(eww_paths)
}

/// Ewwii configuration structure.
#[derive(Debug, Clone, Default)]
pub struct EwwiiConfig {
    windows: HashMap<String, WindowDefinition>,
    root_node: Option<Rc<WidgetNode>>,
    compiled_ast: Option<Rc<AST>>,
}

#[derive(Debug, Clone)]
pub struct WindowDefinition {
    pub name: String,
    pub props: Map,
    pub backend_options: BackendWindowOptionsDef,
    pub root_widget: Rc<WidgetNode>,
}

impl EwwiiConfig {
    /// Load an [`EwwiiConfig`] from the config dir of the given [`crate::EwwPaths`], reading the main config file.
    pub fn read_from_dir(eww_paths: &EwwPaths) -> Result<Self> {
        let rhai_path = eww_paths.get_rhai_path();
        if !rhai_path.exists() {
            bail!("The configuration file `{}` does not exist", rhai_path.display());
        }

        // initialize configuration parser
        let mut config_parser = ParseConfig::new();

        // get code from file
        let rhai_code = config_parser.code_from_file(&rhai_path)?;

        // get the iirhai widget tree
        let compiled_ast = config_parser.compile_code(&rhai_code)?;
        let poll_listen_scope = ParseConfig::initial_poll_listen_scope(&rhai_code)?;
        let config_tree = config_parser.eval_code_with(
            &rhai_code,
            Some(poll_listen_scope),
            Some(&compiled_ast),
        )?;

        let mut window_definitions = HashMap::new();
        let config_tree_clone = config_tree.clone();

        if let WidgetNode::Enter(children) = config_tree_clone {
            for node in children {
                if let WidgetNode::DefWindow { name, props, node } = node {
                    let backend_options = BackendWindowOptionsDef::from_map(&props)?;

                    let win_def = WindowDefinition {
                        name,
                        props,
                        backend_options,
                        root_widget: Rc::new(*node),
                    };
                    window_definitions.insert(win_def.name.clone(), win_def);
                }
            }
        } else {
            bail!("Expected root node to be `Enter`, but got something else.");
        }

        Ok(EwwiiConfig {
            windows: window_definitions,
            root_node: Some(Rc::new(config_tree)),
            compiled_ast: Some(Rc::new(compiled_ast)),
        })
    }

    pub fn get_windows(&self) -> &HashMap<String, WindowDefinition> {
        &self.windows
    }

    pub fn get_window(&self, name: &str) -> Result<&WindowDefinition> {
        self.windows.get(name).with_context(|| {
            format!(
                "No window named '{}' exists in config.\nThis may also be caused by your config failing to load properly, \
                 please check for any other errors in that case.",
                name
            )
        })
    }

    pub fn get_root_node(&self) -> Result<Rc<WidgetNode>> {
        self.root_node.clone().ok_or_else(|| anyhow::anyhow!("root_node is missing"))
    }

    pub fn get_windows_root_widget(config_tree: WidgetNode) -> Result<WidgetNode> {
        if let WidgetNode::Enter(children) = config_tree {
            for node in children {
                if let WidgetNode::DefWindow { node: boxed_node, .. } = node {
                    return Ok(*boxed_node);
                }
            }
            bail!("No `DefWindow` found inside `Enter`");
        } else {
            bail!("Expected root node to be `Enter`, but got something else.");
        }
    }

    pub fn get_borrowed_windows_root_widget(config_tree: &WidgetNode) -> Result<&WidgetNode> {
        if let WidgetNode::Enter(children) = config_tree {
            for node in children {
                if let WidgetNode::DefWindow { node: boxed_node, .. } = node {
                    return Ok(&**boxed_node);
                }
            }
            bail!("No `DefWindow` found inside `Enter`");
        } else {
            bail!("Expected root node to be `Enter`, but got something else.");
        }
    }

    pub fn get_owned_compiled_ast(&self) -> Option<Rc<AST>> {
        self.compiled_ast.clone()
    }
}
