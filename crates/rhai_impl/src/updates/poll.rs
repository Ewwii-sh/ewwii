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
use rhai::Map;
use shared_utils::extract_props::*;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::watch;
use tokio::time::sleep;

pub fn handle_poll(
    var_name: String,
    props: &Map,
    store: ReactiveVarStore,
    tx: tokio::sync::mpsc::UnboundedSender<String>,
) {
    // Parse polling interval
    let interval = get_duration_prop(props, "interval", Some(Duration::from_secs(1)));
    let interval = interval.expect("Error parsing interval property of poll");

    let cmd = match get_string_prop(props, "cmd", Some("")) {
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

    // Check Dash and prefer if dash is installed.

    let dash_installed: bool = Command::new("which")
        .arg("dash")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let shell = if dash_installed {
        "/bin/sh"
    } else {
        "/bin/dash"
    };


    tokio::spawn(async move {
        // Spawn a persistent shell
        let mut child = match Command::new(shell)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(err) => {
                log::error!("[{}] failed to spawn shell: {}", var_name, err);
                return;
            }
        };

        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");
        let mut reader = BufReader::new(stdout).lines();

        let mut last_value: Option<String> = None;

        loop {
            // Send command
            if let Err(err) = stdin.write_all(cmd.as_bytes()).await {
                log::error!("[{}] failed to write to shell stdin: {}", var_name, err);
                break;
            }
            if let Err(err) = stdin.write_all(b"\n").await {
                log::error!("[{}] failed to write newline to shell stdin: {}", var_name, err);
                break;
            }
            if let Err(err) = stdin.flush().await {
                log::error!("[{}] failed to flush shell stdin: {}", var_name, err);
                break;
            }

            // Read single line output
            let output_line = reader.next_line().await;
            if let Ok(Some(stdout_line)) = output_line {
                let stdout_trimmed = stdout_line.trim().to_string();
                if Some(&stdout_trimmed) != last_value.as_ref() {
                    last_value = Some(stdout_trimmed.clone());
                    log::debug!("[{}] polled value: {}", var_name, stdout_trimmed);
                    store.write().unwrap().insert(var_name.clone(), stdout_trimmed);
                    let _ = tx.send(var_name.clone());
                } else {
                    log::trace!("[{}] value unchanged, skipping tx", var_name);
                }
            } else {
                log::warn!("[{}] shell output ended or failed: {:?}", var_name, output_line);
                break;
            }

            tokio::select! {
                _ = sleep(interval) => {}
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        let _ = child.kill().await;
                        break;
                    }
                }
            }
        }
    });
}
