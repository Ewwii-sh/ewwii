/*
    ┏━━━━━━━━━━━━━━━━━━━━━┓
    ┃ Reference structure ┃
    ┗━━━━━━━━━━━━━━━━━━━━━┛

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
    pub struct ListenScriptVar {
        pub name: VarName,
        pub command: String,
        pub initial_value: DynVal,
        pub command_span: Span,
        pub name_span: Span,
    }
*/

use super::{ReactiveVarStore, SHUTDOWN_REGISTRY};
use ewwii_shared_util::general_helper::*;
use rhai::Map;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::sync::watch;

pub fn handle_listen(var_name: String, props: Map, store: ReactiveVarStore, tx: tokio::sync::mpsc::UnboundedSender<String>) {
    let cmd = match get_string_prop(&props, "cmd", Some("")) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Listen {} missing cmd property: {}", var_name, e);
            return;
        }
    };

    // No need to do this as we apply the initial value before parsing
    // Handle initial value
    // if let Ok(initial) = get_string_prop(&props, "initial", None) {
    //     log::debug!("[{}] initial value: {}", var_name, initial);
    //     store.write().unwrap().insert(var_name.clone(), initial.clone());
    //     let _ = tx.send(var_name.clone());
    // }

    let store = store.clone();
    let tx = tx.clone();

    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg(&cmd)
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start listener process");

    let stdout = BufReader::new(child.stdout.take().unwrap());

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
    SHUTDOWN_REGISTRY.lock().unwrap().push(shutdown_tx.clone());

    tokio::spawn(async move {
        let mut last_value: Option<String> = None;
        let mut lines = stdout.lines();

        loop {
            tokio::select! {
                maybe_line = lines.next_line() => {
                    match maybe_line {
                        Ok(Some(line)) => {
                            let val = line.trim().to_string();
                            if Some(&val) != last_value.as_ref() {
                                last_value = Some(val.clone());
                                log::debug!("[{}] listened value: {}", var_name, val);
                                store.write().unwrap().insert(var_name.clone(), val);
                                let _ = tx.send(var_name.clone());
                            } else {
                                log::trace!("[{}] value unchanged, skipping tx", var_name);
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            log::error!("[{}] error reading line: {}", var_name, e);
                            break;
                        }
                    }
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        log::info!("[{}] stopping listener task", var_name);
                        break;
                    }
                }
            }
        }
    });
}
