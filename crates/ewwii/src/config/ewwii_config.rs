use crate::{
    // ipc_server,
    // error_handling_ctx,
    paths::EwwiiPaths,
    window::backend_window_options::BackendWindowOptionsDef,
};
use anyhow::{bail, Context, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use ewwii_plugin_api::CustomConfigEngine;
use ewwii_rhai_impl::parser::RhaiParseConfig;
use ewwii_shared_utils::ast::WidgetNode;
use ewwii_shared_utils::prop::PropertyMap;
use rhai::AST;

pub enum ConfigEngine {
    Default(RhaiParseConfig),
    Custom(CustomConfigEngine),
}

impl ConfigEngine {
    pub fn is_default(&self) -> bool {
        matches!(self, ConfigEngine::Default(_))
    }
}

// NOTE: These global variables are used for the proper functioning
// of bind function and for access to AST across the whole program.
thread_local! {
    pub static EWWII_CONFIG_PARSER: RefCell<Option<RhaiParseConfig>> = RefCell::new(None);
}

/// Load an [`EwwiiConfig`] from the config dir of the given [`crate::EwwiiPaths`],
/// resetting and applying the global YuckFiles object in [`crate::error_handling_ctx`].
pub fn read_from_ewwii_paths(ewwii_paths: &EwwiiPaths) -> Result<EwwiiConfig> {
    EwwiiConfig::read_from_dir(ewwii_paths)
}

/// Ewwii configuration structure.
#[derive(Debug, Clone, Default)]
pub struct EwwiiConfig {
    windows: HashMap<String, WindowDefinition>,
    root_node: Option<Rc<WidgetNode>>,
    compiled_ast: Option<Rc<RefCell<AST>>>,
}

#[derive(Debug, Clone)]
pub struct WindowDefinition {
    pub name: String,
    pub props: PropertyMap,
    pub backend_options: BackendWindowOptionsDef,
    pub root_widget: Rc<WidgetNode>,
}

impl EwwiiConfig {
    /// Load an [`EwwiiConfig`] from the config dir of the given [`crate::EwwiiPaths`], reading the main config file.
    pub fn read_from_dir(ewwii_paths: &EwwiiPaths) -> Result<Self> {
        EWWII_CONFIG_PARSER.with(|p| {
            let mut parser = p.borrow_mut();
            let config_parser = parser.as_mut().context("Config parser not initialized")?;

            let mainfile = config_parser.main_file();
            let configlang_path = ewwii_paths.get_configlang_path(&mainfile);
            if !configlang_path.exists() {
                bail!("The configuration file `{}` does not exist", configlang_path.display());
            }

            // get code from file
            let config_code = crate::paths::code_from_file(&configlang_path)?;
            let configlang_path_opt_str = configlang_path.to_str();

            // get the rhai widget tree
            let compiled_ast = config_parser.compile_code(&config_code, configlang_path_opt_str)?;

            config_parser.register_poll_listen_globals(&config_code)?;

            let config_tree = config_parser.eval_code_with(
                &config_code,
                None,
                Some(&compiled_ast),
                configlang_path_opt_str,
            )?;

            let mut window_definitions = HashMap::new();

            if let WidgetNode::Tree(children) = config_tree.clone() {
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
                compiled_ast: Some(Rc::new(RefCell::new(compiled_ast))),
            })
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

    pub fn get_owned_compiled_ast(&self) -> Option<Rc<RefCell<AST>>> {
        self.compiled_ast.clone()
    }

    pub fn replace_data(&mut self, new_dat: Self) {
        if let (Some(old_ast_rc), Some(new_ast_rc)) =
            (self.compiled_ast.as_ref(), new_dat.compiled_ast.as_ref())
        {
            *old_ast_rc.borrow_mut() = new_ast_rc.borrow().clone();
        }

        self.windows = new_dat.windows;
        self.root_node = new_dat.root_node;
    }
}
