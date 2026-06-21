use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct RuntimePaths {
    pub log_file: String,
    pub log_dir: String,
    pub ipc_socket_file: String,
    pub config_dir: String,
}
