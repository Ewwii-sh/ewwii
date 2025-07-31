// WHY THE HECK IS YUCK SO HARD TO REPLACE?
// I am losing my sanity replacing it!
// I wonder how honorificabilitudinitatibus will I feel after replacing yuck...
use anyhow::{bail, Context, Result};
use std::{collections::HashMap, path::PathBuf};

use crate::{
    ipc_server,
    // error_handling_ctx,
    paths::EwwPaths,
    window::backend_window_options::BackendWindowOptionsDef,
};

use iirhai::{parser::ParseConfig, widgetnode::WidgetNode};

use rhai::Map;

use tokio::{net::UnixStream, runtime::Runtime, sync::mpsc};

/// Load an [`EwwiiConfig`] from the config dir of the given [`crate::EwwPaths`],
/// resetting and applying the global YuckFiles object in [`crate::error_handling_ctx`].
pub fn read_from_ewwii_paths(eww_paths: &EwwPaths) -> Result<EwwiiConfig> {
    EwwiiConfig::read_from_dir(eww_paths)
}

/// Ewwii configuration structure.
#[derive(Debug, Clone, Default)]
pub struct EwwiiConfig {
    windows: HashMap<String, WindowDefinition>,
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

        // Create mpsc channel
        let (tx, rx) = mpsc::channel::<String>(32);

        // Gets iirhai ipc socket file
        let paths = EwwPaths::default()?;
        let iirhai_socket_file = paths.get_iirhai_ipc_socket_file().to_path_buf();

        // starts iirhai ipc server
        // a tokio runtime is used because we are calling async function
        let tokio_rt = Runtime::new().unwrap();
        let result = tokio_rt.block_on(ipc_server::run_iirhai_server(&iirhai_socket_file));

        match result {
            Ok(()) => {
                // starting the reader and passing the sender
                tokio_rt.spawn(run_ipc_reader(iirhai_socket_file, tx));

                tokio_rt.spawn(iirhai_consumer(rx));

                let mut window_definitions = HashMap::new();

                if let WidgetNode::Enter(children) = config_tree {
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

                Ok(EwwiiConfig { windows: window_definitions })
            }
            Err(_) => bail!("Failed to run the iirhai IPC server."),
        }
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
}

// channel that reads messages from 'run_ipc_reader'
async fn iirhai_consumer(mut rx: mpsc::Receiver<String>) {
    while let Some(message) = rx.recv().await {
        println!("[iirhai] Received: {}", message);

        // TODO: parse tree & call GTK update logic
    }
}

async fn run_ipc_reader(socket_path: PathBuf, tx: mpsc::Sender<String>) -> Result<()> {
    let stream = UnixStream::connect(&socket_path).await?;
    let (mut stream_read, _) = tokio::io::split(stream);

    loop {
        let line = ipc_server::read_iirhai_line_from_stream(&mut stream_read).await?;
        tx.send(line).await.unwrap(); // Forward the message
    }
}
