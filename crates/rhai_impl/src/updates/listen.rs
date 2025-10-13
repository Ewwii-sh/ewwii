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
use nix::{
    sys::signal,
    unistd::{setpgid, Pid},
};
use rhai::Map;
use shared_utils::extract_props::*;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::signal as tokio_signal;
use tokio::sync::watch;

pub fn handle_listen(
    var_name: String,
    props: &Map,
    shell: String,
    store: ReactiveVarStore,
    tx: tokio::sync::mpsc::UnboundedSender<String>,
) {
    let cmd = match get_string_prop(props, "cmd", Some("")) {
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

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
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
        let mut child = unsafe {
            Command::new(&shell)
                .arg("-c")
                .arg(&cmd)
                // .kill_on_drop(true)
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
                        // Perhaps make it a TODO?
                        log::warn!("Parent-death signal is not supported on macOS system");
                    }

                    Ok(())
                })
                .spawn()
                .expect("failed to start listener process")
        };

        let mut stdout_lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let mut stderr_lines = BufReader::new(child.stderr.take().unwrap()).lines();

        let mut last_value: Option<String> = None;

        loop {
            tokio::select! {
                maybe_line = stdout_lines.next_line() => {
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
                        Ok(None) => break, // EOF
                        Err(e) => {
                            log::error!("[{}] error reading line: {}", var_name, e);
                            break;
                        }
                    }
                }
                maybe_err_line = stderr_lines.next_line() => {
                    if let Ok(Some(line)) = maybe_err_line {
                        log::warn!("stderr of `{}`: {}", var_name, line);
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
