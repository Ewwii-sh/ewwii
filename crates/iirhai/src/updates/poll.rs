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

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
    pub struct ListenScriptVar {
        pub name: VarName,
        pub command: String,
        pub initial_value: DynVal,
        pub command_span: Span,
        pub name_span: Span,
    }
*/

use std::time::Duration;
use tokio::time::sleep;
use tokio::process::Command;
use ewwii_shared_util::general_helper::*;
use rhai::Map;
use super::ReactiveVarStore;

pub fn handle_poll(
    var_name: String,
    props: Map,
    store: ReactiveVarStore,
    tx: tokio::sync::mpsc::UnboundedSender<String>,
) {
    // Parse polling interval
    let interval = get_duration_prop(&props, "interval", Some(Duration::from_secs(1)))
        .unwrap_or(Duration::from_secs(1));

    let cmd = match get_string_prop(&props, "cmd", Some("")) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Poll {} missing cmd property: {}", var_name, e);
            return;
        }
    };

    // Handle initial value
    if let Ok(initial) = get_string_prop(&props, "initial", None) {
        log::debug!("[{}] initial value: {}", var_name, initial);
        store.write().unwrap().insert(var_name.clone(), initial.clone());
        let _ = tx.send(var_name.clone());
    }

    let store = store.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        loop {
            match Command::new("/bin/sh")
                .arg("-c")
                .arg(&cmd)
                .output()
                .await
            {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        log::debug!("[{}] polled value: {}", var_name, stdout);
                        store.write().unwrap().insert(var_name.clone(), stdout);
                        let _ = tx.send(var_name.clone());
                    } else {
                        log::warn!("[{}] command failed: {:?}", var_name, output.status);
                    }
                }
                Err(err) => {
                    log::error!("[{}] failed to execute poll cmd: {}", var_name, err);
                }
            }

            sleep(interval).await;
        }
    });
}
