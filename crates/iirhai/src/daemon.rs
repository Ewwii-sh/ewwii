use std::path::PathBuf;
use std::sync::Arc;
use tokio::{
    signal,
    net::{UnixListener, unix::{OwnedWriteHalf, OwnedReadHalf}},
    io::{AsyncWriteExt, BufReader, AsyncBufReadExt},
    sync::Mutex,
};
// use anyhow::Result;
use log::info;
use serde_json::Value;

// use crate::{
//     parser::ParseConfig,
//     ipc_manager::IpcManager,
// };

pub struct IIRhaiDaemon {
    socket_path: PathBuf,
    clients: Arc<Mutex<Vec<OwnedWriteHalf>>>,
}

enum DaemonState {
    Idle,
    Running,
}

impl IIRhaiDaemon {
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn run_ewwii_server(&self) -> anyhow::Result<()> {
        let _ = std::fs::remove_file(&self.socket_path); // cleanup stale socket

        let listener = UnixListener::bind(&self.socket_path)?;
        info!("IIRhai Daemon listening on {:?}", &self.socket_path);

        let daemon_state = Arc::new(Mutex::new(DaemonState::Idle));
        loop {
            tokio::select! {
                Ok((stream, _)) = listener.accept() => {
                    let state_clone = Arc::clone(&daemon_state);
                    let clients = Arc::clone(&self.clients);
                    tokio::spawn(async move {
                        let (reader, writer) = stream.into_split();

                            // lock clients safely
                            {
                                let mut guard = clients.lock().await;
                                guard.push(writer);
                            } // mutex dropped here
                        if let Err(e) = Self::handle_commands(reader, state_clone, clients.clone()).await {
                            eprintln!("Command handler error: {e}");
                        }
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

    async fn handle_commands(stream_read: OwnedReadHalf, state: Arc<Mutex<DaemonState>>, clients: Arc<Mutex<Vec<OwnedWriteHalf>>>,) -> anyhow::Result<()> {
        let mut buf_reader = BufReader::new(stream_read);

        loop {
            let mut line = String::new();
            if buf_reader.read_line(&mut line).await? == 0 {
                break Ok(()); // client disconnected
            }

            let msg: Value = serde_json::from_str(&line)?;
            match msg["cmd"].as_str() {
                Some("start") => {
                    let mut state_guard = state.lock().await;
                    if let DaemonState::Idle = *state_guard {
                        // transition to running
                        *state_guard = DaemonState::Running;
                    } else if let DaemonState::Running = *state_guard {
                        break Ok(());
                    }

                    drop(state_guard);

                    // Send greeting
                    Self::send_if_running_static("started\n", &clients, &state).await;
                },
                Some(&_) => {
                    // invalid command
                },
                None => {
                    // Invalid command
                }
            }
        }
    }

    async fn send_if_running_static(
        msg: &str,
        clients: &Arc<Mutex<Vec<OwnedWriteHalf>>>,
        state: &Arc<Mutex<DaemonState>>,
    ) {
        if let DaemonState::Running = *state.lock().await {
            let mut guard = clients.lock().await;
            let mut i = 0;
            while i < guard.len() {
                match guard[i].write_all(msg.as_bytes()).await {
                    Ok(_) => i += 1,
                    Err(e) => {
                        eprintln!("Client write error: {e}");
                        guard.remove(i);
                    }
                }
            }
        }
    }
}

// async fn get_widget_json(config_path: PathBuf) -> anyhow::Result<Value> {
//     let mut parser = ParseConfig::new();
//     let widget_tree = parser.parse_widget_from_file(config_path)?;

//     let ipc_manager = IpcManager::new(widget_tree.clone());
//     let json_tree = ipc_manager.transpile_to_json();

//     Ok(json_tree)
// }