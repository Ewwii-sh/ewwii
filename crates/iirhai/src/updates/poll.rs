/*
    ┏━━━━━━━━━━━━━━━━━━━━━┓
    ┃ Reference structure ┃
    ┗━━━━━━━━━━━━━━━━━━━━━┛

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
    pub struct PollScriptVar {
        pub name: VarName,
        pub run_while_expr: SimplExpr,
        pub command: VarSource,
        pub initial_value: Option<DynVal>,
        pub interval: std::time::Duration,
        pub name_span: Span,
    }
*/

use super::{ReactiveVarStore, SHUTDOWN_REGISTRY};
use ewwii_shared_util::general_helper::*;
use rhai::Map;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::watch;
use tokio::time::sleep;

pub fn handle_poll(var_name: String, props: Map, store: ReactiveVarStore, tx: tokio::sync::mpsc::UnboundedSender<String>) {
    // Parse polling interval
    let interval = get_duration_prop(&props, "interval", Some(Duration::from_secs(1)));
    let interval = interval.expect("Error parsing interval property of poll");

    let cmd = match get_string_prop(&props, "cmd", Some("")) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Poll {} missing cmd property: {}", var_name, e);
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

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
    SHUTDOWN_REGISTRY.lock().unwrap().push(shutdown_tx.clone());

    tokio::spawn(async move {
        let mut last_value: Option<String> = None;

        loop {
            match Command::new("/bin/sh").arg("-c").arg(&cmd).output().await {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if Some(&stdout) != last_value.as_ref() {
                            last_value = Some(stdout.clone());

                            log::debug!("[{}] polled value: {}", var_name, stdout);
                            store.write().unwrap().insert(var_name.clone(), stdout);
                            let _ = tx.send(var_name.clone());
                        } else {
                            log::trace!("[{}] value unchanged, skipping tx", var_name);
                        }
                    } else {
                        log::warn!("[{}] command failed: {:?}", var_name, output.status);
                    }
                }
                Err(err) => {
                    log::error!("[{}] failed to execute poll cmd: {}", var_name, err);
                }
            }

            tokio::select! {
                _ = sleep(interval) => { continue; }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        log::debug!("[{}] stopping task", var_name);
                        break;
                    }
                }
            }
        }
    });
}
