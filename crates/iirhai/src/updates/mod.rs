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

use crate::widgetnode::WidgetNode;
use listen::handle_listen;
use poll::handle_poll;
use std::{collections::HashMap, sync::Arc, sync::RwLock};
use tokio::sync::mpsc::UnboundedSender;

pub type ReactiveVarStore = Arc<RwLock<HashMap<String, String>>>;

pub fn handle_state_changes(enter_node: WidgetNode, tx: UnboundedSender<String>) -> ReactiveVarStore {
    // Enter node is the WidgetNode of Enter()
    // it is the very root of every config.
    let store: ReactiveVarStore = Arc::new(RwLock::new(HashMap::new()));

    if let WidgetNode::Enter(children) = enter_node {
        for child in children {
            match child {
                WidgetNode::Poll { var, props } => {
                    handle_poll(var, props, store.clone(), tx.clone());
                }
                WidgetNode::Listen { var, props } => {
                    handle_listen(var, props, store.clone(), tx.clone());
                }
                _ => {}
            }
        }
    } else {
        log::warn!("Expected Enter() as root node for config");
    }

    store
}
