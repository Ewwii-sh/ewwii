use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::{
    signal,
    net::{UnixListener, unix::OwnedWriteHalf},
    io::AsyncWriteExt,
};
// use anyhow::Result;
use log::info;
use serde_json::Value;

use crate::{
    parser::ParseConfig,
    ipc_manager::IpcManager,
};

pub struct IIRhaiDaemon {
    socket_path: PathBuf,
    clients: Arc<Mutex<Vec<OwnedWriteHalf>>>,
    config_path: PathBuf,
}

impl IIRhaiDaemon {
    pub fn new(socket_path: PathBuf, config_path: PathBuf) -> Self {
        Self {
            socket_path,
            config_path,
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn run_ewwii_server(&self) -> anyhow::Result<()> {
        let _ = std::fs::remove_file(&self.socket_path); // cleanup stale socket

        let listener = UnixListener::bind(&self.socket_path)?;
        info!("IIRhai Daemon listening on {:?}", &self.socket_path);

        loop {
            tokio::select! {
                Ok((stream, _)) = listener.accept() => {
                    let config_path = self.config_path.clone();
                    let clients = self.clients.clone();
                    tokio::spawn(async move {
                        let (_reader, mut writer) = stream.into_split();

                        let json_tree = get_widget_json(config_path).await;

                        // Send greeting
                        match json_tree {
                            Ok(val) => {
                                let msg = format!("{}\n", serde_json::to_string(&val).unwrap());

                                if let Err(e) = writer.write_all(msg.as_bytes()).await {
                                    eprintln!("Failed to write JSON to client: {e}");
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to get widget JSON: {e}");
                            }
                        }

                        clients.lock().unwrap().push(writer);
                    });
                }

                _ = signal::ctrl_c() => {
                    info!("Daemon received shutdown signal.");
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn print_ipc_message(&self, msg: &str) {
        let mut clients = self.clients.lock().unwrap();
        let mut i = 0;
        while i < clients.len() {
            match clients[i].write_all(msg.as_bytes()).await {
                Ok(_) => i += 1,
                Err(e) => {
                    eprintln!("Client write error: {e}");
                    clients.remove(i);
                }
            }
        }
    }
}

async fn get_widget_json(config_path: PathBuf) -> anyhow::Result<Value> {
    let mut parser = ParseConfig::new();
    let widget_tree = parser.parse_widget_from_file(config_path)?;

    let ipc_manager = IpcManager::new(widget_tree.clone());
    let json_tree = ipc_manager.transpile_to_json();

    Ok(json_tree)
}