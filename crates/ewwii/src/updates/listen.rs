use super::{api::VarWatcherAPI, SHUTDOWN_REGISTRY};
use ewwii_shared_utils::prop::PropertyMap;
use ewwii_shared_utils::prop_utils::*;
use nix::libc;
use nix::{
    sys::signal::{self, Signal},
    unistd::{setpgid, Pid},
};
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::sync::watch;

pub async fn stream_cmd_lines<F>(
    shell: String,
    cmd: String,
    mut shutdown_rx: watch::Receiver<bool>,
    mut on_line: F,
) where
    F: FnMut(&str) + Send + 'static,
{
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

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take();

    // offload stderr in bg thread so it doesn't slow stdout
    if let Some(stderr_stream) = stderr {
        tokio::spawn(async move {
            let mut err_buf = String::new();
            let mut reader = BufReader::new(stderr_stream);
            while let Ok(bytes_read) = reader.read_line(&mut err_buf).await {
                if bytes_read == 0 { break; }
                log::warn!("stream_cmd_lines stderr: {}", err_buf.trim_end());
                err_buf.clear();
            }
        });
    }

    let mut reader = BufReader::with_capacity(16 * 1024, stdout);
    let mut line_buf = String::with_capacity(256);

    loop {
        line_buf.clear();

        tokio::select! {
            biased;
            res = reader.read_line(&mut line_buf) => {
                match res {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let trimmed = line_buf.trim();
                        if !trimmed.is_empty() {
                            on_line(trimmed);
                        }
                    }
                    Err(e) => {
                        log::error!("stream_cmd_lines read error: {e}");
                        break;
                    }
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
    const CMD_KEY: &str = "cmd";

    let cmd_prop = soft_retreive_prop(props, CMD_KEY, "");
    let cmd = match get_string_prop(&cmd_prop, CMD_KEY) {
        Ok(c) => unwrap_static(CMD_KEY, c),
        Err(e) => {
            log::warn!("Listen {} cmd property invalid: {}", var_name, e);
            return;
        }
    };

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    SHUTDOWN_REGISTRY.lock().unwrap().push(shutdown_tx);

    tokio::spawn(async move {
        let mut last_value = String::new();

        stream_cmd_lines(shell, cmd, shutdown_rx, move |line| {
            if line != last_value {
                last_value.clear();
                last_value.push_str(line);

                log::debug!("[{var_name}] listened value: {line}");
                VarWatcherAPI::update_with_broadcast(&var_name, line.to_string());
            }
        })
        .await;
    });
}

async fn terminate_child(mut child: tokio::process::Child) {
    if let Some(id) = child.id() {
        let _ = signal::killpg(Pid::from_raw(id as i32), Signal::SIGTERM);
        tokio::select! {
            _ = child.wait() => { },
            _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {
                let _ = child.kill().await;
            }
        };
    } else {
        let _ = child.kill().await;
    }
}
