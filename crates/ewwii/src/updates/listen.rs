use super::{api::VarWatcherAPI, SHUTDOWN_REGISTRY};
use ewwii_shared_utils::prop::PropertyMap;
use ewwii_shared_utils::prop_utils::*;
use nix::libc;
use nix::{
    sys::signal,
    unistd::{setpgid, Pid},
};
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::signal as tokio_signal;
use tokio::sync::mpsc;
use tokio::sync::watch;

pub async fn stream_cmd_lines(
    shell: String,
    cmd: String,
    tx: mpsc::Sender<String>,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    let mut child = unsafe {
        Command::new(shell)
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .pre_exec(|| {
                let _ = setpgid(Pid::from_raw(0), Pid::from_raw(0));

                #[cfg(target_os = "linux")]
                {
                    if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) != 0 {
                        log::error!(
                            "prctl PR_SET_PDEATHSIG failed: {}",
                            std::io::Error::last_os_error()
                        );
                    }
                }

                #[cfg(target_os = "freebsd")]
                {
                    use libc::{c_int, c_void};
                    const PROC_PDEATHSIG_CTL: c_int = 11;
                    let sig: c_int = libc::SIGTERM;
                    if libc::procctl(
                        libc::P_PID,
                        0,
                        PROC_PDEATHSIG_CTL,
                        &sig as *const _ as *mut c_void,
                    ) != 0
                    {
                        log::error!(
                            "procctl PROC_PDEATHSIG_CTL failed: {}",
                            std::io::Error::last_os_error()
                        );
                    }
                }

                #[cfg(target_os = "macos")]
                {
                    log::warn!("Parent-death signal is not supported on macOS");
                }

                Ok(())
            })
            .spawn()
            .expect("failed to start listener process")
    };

    let mut stdout_lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut stderr_lines = BufReader::new(child.stderr.take().unwrap()).lines();

    loop {
        tokio::select! {
            maybe_line = stdout_lines.next_line() => {
                match maybe_line {
                    Ok(Some(line)) => {
                        let val = line.trim().to_string();
                        // Stop forwarding if the receiver was dropped
                        if tx.send(val).await.is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        log::error!("stream_cmd_lines: error reading stdout: {}", e);
                        break;
                    }
                }
            }
            maybe_err = stderr_lines.next_line() => {
                if let Ok(Some(line)) = maybe_err {
                    log::warn!("stream_cmd_lines stderr: {}", line);
                }
            }
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    break;
                }
            }
        }
    }

    let _ = terminate_child(child).await;
}

pub fn handle_listen(var_name: String, props: &PropertyMap, shell: String) {
    let cmd = match get_string_prop(props, "cmd", Some("")) {
        Ok(c) => unwrap_static("cmd", c),
        Err(e) => {
            log::warn!("Listen {} cmd property either missing or invalid: {}", var_name, e);
            return;
        }
    };

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    SHUTDOWN_REGISTRY.lock().unwrap().push(shutdown_tx.clone());

    // Task to catch SIGINT and SIGTERM
    tokio::spawn({
        let shutdown_tx = shutdown_tx.clone();
        async move {
            let mut sigterm_stream =
                tokio_signal::unix::signal(tokio_signal::unix::SignalKind::terminate()).unwrap();

            tokio::select! {
                _ = tokio_signal::ctrl_c() => {
                    log::trace!("Received SIGINT");
                }
                _ = sigterm_stream.recv() => {
                    log::trace!("Received SIGTERM");
                }
            }
            let _ = shutdown_tx.send(true);
        }
    });

    tokio::spawn(async move {
        let (tx, mut rx) = mpsc::channel::<String>(32);

        // Spawn the generic streamer
        tokio::spawn(stream_cmd_lines(shell, cmd, tx, shutdown_rx));

        // Handle dedup + broadcast in this task
        let mut last_value: Option<String> = None;
        while let Some(val) = rx.recv().await {
            if Some(&val) != last_value.as_ref() {
                last_value = Some(val.clone());
                log::debug!("[{}] listened value: {}", var_name, val);
                VarWatcherAPI::update_with_broadcast(&var_name, val);
            } else {
                log::trace!("[{}] value unchanged, skipping tx", var_name);
            }
        }
    });
}

async fn terminate_child(mut child: tokio::process::Child) {
    if let Some(id) = child.id() {
        log::debug!("Killing process with id {}", id);
        let _ = signal::killpg(Pid::from_raw(id as i32), signal::SIGTERM);
        tokio::select! {
            _ = child.wait() => { },
            _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
                let _ = child.kill().await;
            }
        };
    } else {
        let _ = child.kill().await;
    }
}
