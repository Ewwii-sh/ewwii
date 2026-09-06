use super::{api::VarWatcherAPI, SHUTDOWN_REGISTRY};
use ewwii_shared_utils::prop::PropertyMap;
use ewwii_shared_utils::prop_utils::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::watch;

pub fn handle_poll(var_name: String, props: &PropertyMap, shell: String) {
    const INTERVAL_KEY: &str = "interval";
    const CMD_KEY: &str = "cmd";
    const SKIP_KEY: &str = "skip_unchanged";

    let interval_prop = soft_retreive_prop(props, INTERVAL_KEY, "1s");
    let cmd_prop = soft_retreive_prop(props, CMD_KEY, "");

    let interval_duration = match get_duration_prop(&interval_prop, INTERVAL_KEY) {
        Ok(d) => d,
        Err(e) => {
            log::error!("Error parsing interval property of poll {var_name}: {e}");
            return;
        }
    };

    let cmd = match get_string_prop(&cmd_prop, CMD_KEY) {
        Ok(c) => unwrap_static(CMD_KEY, c),
        Err(e) => {
            log::warn!("Poll {} cmd property either missing or invalid: {}", var_name, e);
            return;
        }
    };

    const DEFAULT_SKIP: bool = true;
    let skip_prop = soft_retreive_prop_bool(props, SKIP_KEY, DEFAULT_SKIP);
    let skip_unchanged = match get_bool_prop(&skip_prop, SKIP_KEY) {
        Ok(p) => unwrap_static(SKIP_KEY, p),
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
        let mut reader = BufReader::with_capacity(4096, stdout);

        // format command
        let formatted_cmd = format!("{cmd}\n");
        let cmd_bytes = formatted_cmd.as_bytes();

        // setup tokio interval
        let mut poll_interval = tokio::time::interval(interval_duration);
        poll_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut line_buf = String::with_capacity(128);
        let mut last_value = String::with_capacity(128);

        loop {
            tokio::select! {
                _ = poll_interval.tick() => {
                    if let Err(err) = stdin.write_all(cmd_bytes).await {
                        log::error!("[{var_name}] failed to write to shell stdin: {err}");
                        break;
                    }
                    if let Err(err) = stdin.flush().await {
                        log::error!("[{var_name}] failed to flush shell stdin: {err}");
                        break;
                    }

                    line_buf.clear();
                    match reader.read_line(&mut line_buf).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let trimmed = line_buf.trim();

                            if trimmed != last_value {
                                last_value.clear();
                                last_value.push_str(trimmed);
                                log::debug!("[{var_name}] polled value: {trimmed}");

                                VarWatcherAPI::update_with_broadcast(&var_name, trimmed.to_string());
                            } else if !skip_unchanged {
                                log::trace!("[{var_name}] value unchanged, forcing broadcast");
                                VarWatcherAPI::broadcast(&var_name);
                            }
                        }
                        Err(err) => {
                            log::warn!("[{var_name}] shell output read error: {err}");
                            break;
                        }
                    }
                }
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
