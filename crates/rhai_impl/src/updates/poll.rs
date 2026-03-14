use super::{variable::VarWatcherAPI, SHUTDOWN_REGISTRY};
use rhai::Map;
use shared_utils::prop_utils::*;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::watch;
use tokio::time::sleep;

pub fn handle_poll(var_name: String, props: &Map, shell: String) {
    let interval = get_duration_prop(props, "interval", Some(Duration::from_secs(1)));
    let interval = interval.expect("Error parsing interval property of poll");

    let cmd = match get_string_prop(props, "cmd", Some("")) {
        Ok(c) => unwrap_static("cmd", c),
        Err(e) => {
            log::warn!("Poll {} cmd property either missing or invalid: {}", var_name, e);
            return;
        }
    };

    const DEFAULT_SKIP: bool = true;
    let skip_unchanged = match get_bool_prop(props, "skip_unchanged", Some(DEFAULT_SKIP)) {
        Ok(p) => unwrap_static("skip_unchanged", p),
        Err(e) => {
            log::warn!("Failed to parse skip_unchanged property of poll {}: {}", var_name, e);
            DEFAULT_SKIP
        }
    };

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
    SHUTDOWN_REGISTRY.lock().unwrap().push(shutdown_tx.clone());

    tokio::spawn(async move {
        let mut child = match Command::new(&shell)
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

            let output_line = reader.next_line().await;
            if let Ok(Some(stdout_line)) = output_line {
                let stdout_trimmed = stdout_line.trim().to_string();
                if Some(&stdout_trimmed) != last_value.as_ref() {
                    last_value = Some(stdout_trimmed.clone());
                    log::debug!("[{}] polled value: {}", var_name, stdout_trimmed);

                    VarWatcherAPI::update_with_broadcast(&var_name, stdout_trimmed);
                } else if skip_unchanged {
                    log::trace!("[{}] value unchanged, skipping broadcast", var_name);
                } else {
                    log::trace!("[{}] value unchanged, skipping disabled", var_name);

                    VarWatcherAPI::broadcast(&var_name);
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
