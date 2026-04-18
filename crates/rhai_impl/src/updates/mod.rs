/*
    This is where we update the variables in ewwii

    Since we can poll and listen in ewwii, it is what we will
    use to get variable updates in ewwii.

    Updates are important because it helps keeping rhai dynamic.
    Even though rhai is static by nature (only evaluates once),
    we can trigger a re-evaluation every time a variable updates
    as a workaround to this limitation.
*/

mod listen;
mod poll;
pub mod api;

use ewwii_shared_utils::ast::WidgetNode;
use listen::handle_listen;
use once_cell::sync::Lazy;
use poll::handle_poll;
use std::process::Command;
use std::sync::Mutex;
use tokio::sync::watch;
use api::VarWatcherAPI;
use ewwii_shared_utils::prop::PropertyMap;

pub static SHUTDOWN_REGISTRY: Lazy<Mutex<Vec<watch::Sender<bool>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub fn get_prefered_shell() -> String {
    // Check Dash and prefer if dash is installed.
    let dash_installed: bool =
        Command::new("which").arg("dash").output().map(|o| o.status.success()).unwrap_or(false);

    let shell = if dash_installed { String::from("/bin/dash") } else { String::from("/bin/sh") };

    shell
}

pub enum SignalType {
    Poll,
    Listen
}

pub struct SignalProps {
    pub name: String,
    pub props: PropertyMap,
    pub signal_type: SignalType,
}

pub fn retreive_signals(root_node: &WidgetNode) -> Vec<SignalProps> {
    let mut signals: Vec<SignalProps> = Vec::new();

    if let WidgetNode::Tree(children) = root_node {
        for child in children {
            match child {
                WidgetNode::Poll { var, props } => {
                    let signal = SignalProps {
                        name: var.to_string(),
                        props: props.clone(),
                        signal_type: SignalType::Poll,
                    };

                    signals.push(signal);
                }
                WidgetNode::Listen { var, props } => {
                    let signal = SignalProps {
                        name: var.to_string(),
                        props: props.clone(),
                        signal_type: SignalType::Listen,
                    };

                    signals.push(signal);
                }
                _ => {}
            }
        }
    } else {
        log::warn!("Expected Enter() as root node for config");
    }

    signals
}

pub fn handle_state_changes(signals: Vec<SignalProps>) {
    let shell = get_prefered_shell();

    for signal in signals {
        match signal.signal_type {
            SignalType::Poll => {
                VarWatcherAPI::register(&signal.name, String::new());
                handle_poll(signal.name, &signal.props, shell.clone());
            },
            SignalType::Listen => {
                VarWatcherAPI::register(&signal.name, String::new());
                handle_listen(signal.name, &signal.props, shell.clone());
            }
        }
    }
}

pub fn kill_state_change_handler() {
    let registry = SHUTDOWN_REGISTRY.lock().unwrap();
    for sender in registry.iter() {
        let _ = sender.send(true);
    }
    log::debug!("All state change handlers requested to stop");
}
