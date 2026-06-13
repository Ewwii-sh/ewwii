use super::listen::stream_cmd_lines;
use super::SHUTDOWN_REGISTRY;
use crate::config::ConfigEngine;
use ewwii_shared_utils::prop::PropertyMap;
use ewwii_shared_utils::prop_utils::*;
use gtk4::glib;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio::time::sleep;

pub fn handle_script(parser: &ConfigEngine, props: &PropertyMap, shell: String) {
    const EVERY_KEY: &str = "every";
    const ON_KEY: &str = "on";
    const RUN_KEY: &str = "run";

    let every_prop = soft_retreive_prop(props, EVERY_KEY, "");
    let on_prop = soft_retreive_prop(props, ON_KEY, "");

    let every_sec = match get_duration_prop(&every_prop, EVERY_KEY) {
        Ok(d) => Some(d),
        Err(_) => None,
    };
    let on_cmd = match get_string_prop(&on_prop, ON_KEY) {
        Ok(c) => Some(unwrap_static(ON_KEY, c)),
        Err(_) => None,
    };
    let run = match get_callback_prop(&props, RUN_KEY) {
        Ok(r) => r,
        Err(_) => {
            log::error!("Script requires 'run' property to function.");
            return;
        }
    };

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
    SHUTDOWN_REGISTRY.lock().unwrap().push(shutdown_tx.clone());

    let parser: ConfigEngine = parser.clone();

    match (every_sec, on_cmd) {
        (Some(interval), None) => {
            glib::MainContext::default().spawn_local(async move {
                loop {
                    parser.handle_callback(&run);

                    tokio::select! {
                        _ = sleep(interval) => {}
                        _ = shutdown_rx.changed() => {
                            if *shutdown_rx.borrow() {
                                break;
                            }
                        }
                    }
                }
            });
        }
        (None, Some(cmd)) => {
            glib::MainContext::default().spawn_local(async move {
                let (tx, mut rx) = mpsc::channel::<String>(32);
                tokio::spawn(stream_cmd_lines(shell, cmd, tx, shutdown_rx));

                while let Some(_) = rx.recv().await {
                    parser.handle_callback(&run);
                }
            });
        }
        (Some(_), Some(_)) => {
            log::error!("Either 'every' or 'on' needs to be provided: got both.");
            return;
        }
        (None, None) => {
            log::error!("Either 'every' or 'on' needs to be provided for Script to work.");
            return;
        }
    }
}
