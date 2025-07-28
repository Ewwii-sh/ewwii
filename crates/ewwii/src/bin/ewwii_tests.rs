use anyhow::{Result, anyhow, bail};
use codespan_reporting::diagnostic::Severity;
use ewwii_shared_util::{AttrName, Spanned};
use std::path::PathBuf;
use std::path::Path;
use iirhai::parser::ParseConfig;
use std::hash::DefaultHasher;
use gtk::{
    gdk::prelude::Cast,
    prelude::{BoxExt, ContainerExt, WidgetExt},
    Orientation,
};
use itertools::Itertools;
use maplit::hashmap;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use tokio::sync::mpsc;
use tokio::runtime::Runtime;
use std::hash::Hash;
use std::hash::Hasher;

use rhai::Map;

use iirhai::widgetnode::WidgetNode;


struct BuilderArgs {
    pub window_defs: Rc<HashMap<String, WindowDefinition>>,
}

#[derive(Debug, Clone)]
struct EwwPaths {
    pub log_file: PathBuf,
    pub log_dir: PathBuf,
    pub ipc_socket_file: PathBuf,
    pub iirhai_ipc_socket_file: PathBuf,
    pub config_dir: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct EwwConfig {
    windows: HashMap<String, WindowDefinition>,
}

#[derive(Debug, Clone)]
struct WindowDefinition {
    name: String,
    props: Map,
    root_widget: WidgetNode,
}

impl EwwPaths {
    pub fn from_config_dir<P: AsRef<Path>>(config_dir: P) -> Result<Self> {
        let config_dir = config_dir.as_ref();
        if config_dir.is_file() {
            bail!("Please provide the path to the config directory, not a file within it")
        }

        if !config_dir.exists() {
            bail!("Configuration directory {} does not exist", config_dir.display());
        }

        let config_dir = config_dir.canonicalize()?;

        let mut hasher = DefaultHasher::new();
        format!("{}", config_dir.display()).hash(&mut hasher);
        // daemon_id is a hash of the config dir path to ensure that, given a normal XDG_RUNTIME_DIR,
        // the absolute path to the socket stays under the 108 bytes limit. (see #387, man 7 unix)
        let daemon_id = format!("{:x}", hasher.finish());

        let ipc_socket_file = std::env::var("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"))
            .join(format!("ewwii-server_{}", daemon_id));

        let iirhai_ipc_socket_file = std::env::var("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"))
            .join(format!("ewwii_iirhai-server_{}", daemon_id));

        // 100 as the limit isn't quite 108 everywhere (i.e 104 on BSD or mac)
        if format!("{}", ipc_socket_file.display()).len() > 100 {
            log::warn!("The IPC socket file's absolute path exceeds 100 bytes, the socket may fail to create.");
        }

        if format!("{}", iirhai_ipc_socket_file.display()).len() > 100 {
            log::warn!("The iirhai IPC socket file's absolute path exceeds 100 bytes, the socket may fail to create.");
        }

        let log_dir = std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap()).join(".cache"))
            .join("ewwii");

        if !log_dir.exists() {
            log::info!("Creating log dir");
            std::fs::create_dir_all(&log_dir)?;
        }

        Ok(EwwPaths { config_dir, log_file: log_dir.join(format!("eww_{}.log", daemon_id)), log_dir, ipc_socket_file, iirhai_ipc_socket_file })
    }

    pub fn default() -> Result<Self> {
        let config_dir = "examples/eww-bar";
        Self::from_config_dir(config_dir)
    }

    pub fn get_log_file(&self) -> &Path {
        self.log_file.as_path()
    }

    pub fn get_log_dir(&self) -> &Path {
        self.log_dir.as_path()
    }

    pub fn get_ipc_socket_file(&self) -> &Path {
        self.ipc_socket_file.as_path()
    }

    pub fn get_iirhai_ipc_socket_file(&self) -> &Path {
        self.iirhai_ipc_socket_file.as_path()
    }

    pub fn get_config_dir(&self) -> &Path {
        self.config_dir.as_path()
    }

    // Modified this code with rhai (the new yuck replacer in ewwii)
    pub fn get_rhai_path(&self) -> PathBuf {
        self.config_dir.join("ewwii.rhai")
    }
}

impl EwwConfig {
fn read_from_dir(eww_paths: &EwwPaths) -> Result<Self> {
    let rhai_path = eww_paths.get_rhai_path();
    if !rhai_path.exists() {
        bail!("The configuration file `{}` does not exist", rhai_path.display());
    }

    // get the iirhai widget tree
    let mut config_parser = ParseConfig::new();
    let config_tree = config_parser.parse_widget_from_file(rhai_path)?;

    // Create mpsc channel
    let (tx, rx) = mpsc::channel::<String>(32);

    // Gets iirhai ipc socket file
    let paths = EwwPaths::default()?;
    let iirhai_socket_file = paths.get_iirhai_ipc_socket_file().to_path_buf();

    // starts iirhai ipc server
    // a tokio runtime is used because we are calling async function
    let tokio_rt = Runtime::new().unwrap();

    // starting the reader and passing the sender
    let mut window_definitions = HashMap::new();

    if let WidgetNode::Enter(children) = config_tree {
        for node in children {
            if let WidgetNode::DefWindow { name, props, node } = node {
                let win_def = WindowDefinition {
                    name: name.clone(),
                    props: props.clone(),
                    root_widget: *node.clone()
                };
                window_definitions.insert(name.clone(), win_def);
            }
        }
    } else {
        bail!("Expected root node to be `Enter`, but got something else.");
    }

    Ok(EwwConfig {
        windows: window_definitions,
    })
}}

fn build_gtk_widget(window_defs: Rc<HashMap<String, WindowDefinition>>) {
    let def = window_defs.values().next().ok_or_else(|| anyhow!("No WindowDefinition passed to build_gtk_widget()"));

    let root_node = &def.unwrap().root_widget;

    // build_gtk_widget_from_node(root_node)
    println!("{:#?}", root_node);
}

// fn build_gtk_widget_from_node(root_node: WidgetNode) {

// }


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir: &Path = Path::new("examples/eww-bar/");
    let eww_paths = EwwPaths::default()?;

    // println!("{:#?}", eww_paths);

    let window_defs = EwwConfig::read_from_dir(&eww_paths)?;

    // println!("{:#?}", window_defs.windows);

    build_gtk_widget(window_defs.windows.into());

    Ok(())
}