use ewwii_shared_utils::prop::PropertyMap;
use ewwii_shared_utils::prop_utils::*;
use super::SHUTDOWN_REGISTRY;
use tokio::sync::watch;
use tokio::time::sleep;

pub fn handle_script(props: PropertyMap) {
    let every_sec = match get_duration_prop(&props, "every", None) {
        Ok(d) => Some(d),
        Err(e) => None,
    };
    let on_cmd = match get_string_prop(&props, "on", Some("")) {
        Ok(c) => Some(unwrap_static("on", c)),
        Err(e) => None,
    };
    let run = match get_callback_prop(&props, "run") {
        Ok(r) => r,
        Err(e) => {
            log::error!("Script requires 'run' property to function.");
            return;
        }
    };

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
    SHUTDOWN_REGISTRY.lock().unwrap().push(shutdown_tx.clone());

    match (every_sec, on_cmd) {
        (Some(interval), Some(cmd)) => {}
        (Some(interval), None) => {
            tokio::spawn(async move {
                loop {
                    // call the callback

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
        (None, Some(cmd)) => {}
        (None, None) => {
            log::error!("Either 'every' or 'on' needs to be provided for Script to work.");
            return;
        }
    }
}
