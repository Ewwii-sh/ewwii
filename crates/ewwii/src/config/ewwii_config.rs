// WHY THE HECK IS YUCK SO HARD TO REPLACE?
// I am losing my sanity replacing it!
// I wonder how honorificabilitudinitatibus will I feel after replacing yuck...
use anyhow::{bail, Context, Result};
use std::collections::HashMap;

use crate::{
    // ipc_server,
    // error_handling_ctx,
    paths::EwwPaths,
    window::backend_window_options::BackendWindowOptionsDef,
};

use iirhai::{parser::ParseConfig, widgetnode::WidgetNode};

use rhai::Map;

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
    root_node: Option<WidgetNode>,
}

#[derive(Debug, Clone)]
pub struct WindowDefinition {
    pub name: String,
    pub props: Map,
    pub backend_options: BackendWindowOptionsDef,
    pub root_widget: WidgetNode,
}

impl EwwiiConfig {
    /// Load an [`EwwiiConfig`] from the config dir of the given [`crate::EwwPaths`], reading the main config file.
    pub fn read_from_dir(eww_paths: &EwwPaths) -> Result<Self> {
        let rhai_path = eww_paths.get_rhai_path();
        if !rhai_path.exists() {
            bail!("The configuration file `{}` does not exist", rhai_path.display());
        }

        // get the iirhai widget tree
        let mut config_parser = ParseConfig::new();
        let config_tree = config_parser.parse_widget_from_file(rhai_path)?;

        let mut window_definitions = HashMap::new();

        if let WidgetNode::Enter(ref children) = config_tree {
            for node in children {
                if let WidgetNode::DefWindow { name, props, node } = node {
                    let win_def = WindowDefinition {
                        name: name.clone(),
                        props: props.clone(),
                        backend_options: BackendWindowOptionsDef::from_map(&props)?,
                        root_widget: *node.clone(),
                    };
                    window_definitions.insert(name.clone(), win_def);
                }
            }
        } else {
            bail!("Expected root node to be `Enter`, but got something else.");
        }

        Ok(EwwiiConfig { windows: window_definitions, root_node: Some(config_tree) })
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

    pub fn get_root_node(&self) -> Result<WidgetNode> {
        self.root_node.clone().ok_or_else(|| anyhow::anyhow!("root_node is missing"))
    }
}
