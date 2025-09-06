/*
    This is where we update the variables in ewwii

    Since we can poll and listen in ewwii, it is what we will
    use to get variable updates in ewwii.

    Updates are important because it helps keeping rhai dynamic.
    Even though rhai is static by nature (only evaluates once),
    we can trigger a re-evaluation every time a variable updates
    as a workaround to this limitation.

    Other than the poll and listen, we also handle the updates of
    the internal built in signals (the functions that return data)
    which is also known as "magic variables" in eww.
*/

mod listen;
mod poll;

use crate::ast::WidgetNode;
use listen::handle_listen;
use once_cell::sync::Lazy;
use poll::handle_poll;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc, sync::RwLock};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch;

pub type ReactiveVarStore = Arc<RwLock<HashMap<String, String>>>;
pub static SHUTDOWN_REGISTRY: Lazy<Mutex<Vec<watch::Sender<bool>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
    pub static TX_REGISTRY: Lazy<Mutex<Vec<UnboundedSender<String>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub fn handle_state_changes(
    root_node: &WidgetNode,
    store: ReactiveVarStore,
) {
    // Enter node is the WidgetNode of Enter()
    // it is the very root of every config.

    if let WidgetNode::Enter(children) = root_node {
        for child in children {
            match child {
                WidgetNode::Poll { var, props } => {
                    handle_poll(var.to_string(), props, store.clone());
                }
                WidgetNode::Listen { var, props } => {
                    handle_listen(var.to_string(), props, store.clone());
                }
                _ => {}
            }
        }
    } else {
        log::warn!("Expected Enter() as root node for config");
    }
}

pub fn register_tx(tx: UnboundedSender<String>) {
    let mut registry = TX_REGISTRY.lock().unwrap();
    registry.push(tx);
}

pub fn kill_state_change_handler() {
    let registry = SHUTDOWN_REGISTRY.lock().unwrap();
    for sender in registry.iter() {
        let _ = sender.send(true);
    }
    log::debug!("All state change handlers requested to stop");
}

pub(super) fn broadcast_update(update: String) {
    let registry = TX_REGISTRY.lock().unwrap();
    for tx in registry.iter() {
        // ignore errors if receiver is gone
        let _ = tx.send(update.clone());
    }
}