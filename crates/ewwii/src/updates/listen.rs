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

use super::ReactiveVarStore;
use ewwii_shared_util::general_helper::*;
use rhai::Map;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::time::sleep;

pub fn handle_listen(var_name: String, props: Map, store: ReactiveVarStore, tx: tokio::sync::mpsc::UnboundedSender<String>) {
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

    let mut child =
        Command::new("/bin/sh").arg("-c").arg(&cmd).stdout(Stdio::piped()).spawn().expect("failed to start listener process");

    let stdout = BufReader::new(child.stdout.take().unwrap());

    tokio::spawn(async move {
        let mut last_value: Option<String> = None;
        let mut lines = stdout.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let val = line.trim().to_string();
            if Some(&val) != last_value.as_ref() {
                log::debug!("[{}] listened value: {}", var_name, val);
                store.write().unwrap().insert(var_name.clone(), val);
                let _ = tx.send(var_name.clone());
            } else {
                log::trace!("[{}] value unchanged, skipping tx", var_name);
            }
        }
    });
}
