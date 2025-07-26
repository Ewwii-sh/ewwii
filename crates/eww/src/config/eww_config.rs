// TODO: Yuck found, have to replace with lua


// WHY THE HECK IS YUCK SO HARD TO REPLACE?
// I am losing my sanity replacing it!
// I wonder how honorificabilitudinitatibus will I feel after replacing yuck...
use anyhow::{bail, Result};
use eww_shared_util::VarName;
use std::{
    collections::HashMap,
    path::PathBuf,
};

use yuck::{
    config::{
        script_var_definition::ScriptVarDefinition, 
        widget_definition::WidgetDefinition,
        window_definition::WindowDefinition, Config,
    },
};

use simplexpr::dynval::DynVal;

use crate::{
    error_handling_ctx, 
    file_database::FileDatabase, 
    paths::EwwPaths, 
    ipc_server,
};

use tokio::{
    runtime::Runtime,
    net::UnixStream
};

/// Load an [`EwwConfig`] from the config dir of the given [`crate::EwwPaths`],
/// resetting and applying the global YuckFiles object in [`crate::error_handling_ctx`].
pub fn read_from_eww_paths(eww_paths: &EwwPaths) -> Result<EwwConfig> {
    error_handling_ctx::clear_files();
    EwwConfig::read_from_dir(&mut error_handling_ctx::FILE_DATABASE.write().unwrap(), eww_paths)
}

/// Eww configuration structure.
#[derive(Debug, Clone, Default)]
pub struct EwwConfig {
    widgets: HashMap<String, WidgetDefinition>,
    windows: HashMap<String, WindowDefinition>,
    initial_variables: HashMap<VarName, DynVal>,
    script_vars: HashMap<VarName, ScriptVarDefinition>,

    // map of variables to all pollvars which refer to them in their run-while-expression
    run_while_mentions: HashMap<VarName, Vec<VarName>>,
}

impl EwwConfig {
    /// Load an [`EwwConfig`] from the config dir of the given [`crate::EwwPaths`], reading the main config file.
    pub fn read_from_dir(files: &mut FileDatabase, eww_paths: &EwwPaths) -> Result<Self> {
        let rhai_path = eww_paths.get_rhai_path();
        if !rhai_path.exists() {
            bail!("The configuration file `{}` does not exist", rhai_path.display());
        }
        let config = Config::generate_from_main_file(files, rhai_path.clone())?;

        // Gets iirhai ipc socket file
        let paths = EwwPaths::default()?;
        let iirhai_socket_file = paths.get_iirhai_ipc_socket_file().to_path_buf();

        // starts iirhai ipc server
        // a tokio runtime is used because we are calling async function
        let tokio_rt = Runtime::new().unwrap();
        let result = tokio_rt.block_on(ipc_server::run_iirhai_server(iirhai_socket_file, rhai_path));

        match result {
            Ok(()) => tokio_rt.block_on(run_ipc_reader(iirhai_socket_file)),
            Err(e) => bail!("Failed to run the iirhai IPC server.")
        };
    }
}

async fn run_ipc_reader(socket_path: PathBuf) -> Result<()> {
    let stream = UnixStream::connect(&socket_path).await?;
    let (mut stream_read, _) = tokio::io::split(stream);

    loop {
        let line = ipc_server::read_iirhai_line_from_stream(&mut stream_read).await?;
        // pass this to mpsc::channel used by ewwii later
    }
}