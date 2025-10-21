use anyhow::Result;
use clap::{Parser, Subcommand};
// use shared_utils::VarName;
use serde::{Deserialize, Serialize};

use crate::{
    app,
    daemon_response::{self, DaemonResponse, DaemonResponseSender},
    dynval::DynVal,
    window::{coords::Coords, monitor::MonitorIdentifier, window_geometry::AnchorPoint},
};

use std::collections::HashMap;

/// Struct that gets generated from `RawOpt`.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Opt {
    pub force_wayland: bool,
    pub log_debug: bool,
    pub show_logs: bool,
    pub restart: bool,
    pub config_path: Option<std::path::PathBuf>,
    pub action: Action,
    pub no_daemonize: bool,
}

#[derive(Parser, Debug, Serialize, Deserialize, PartialEq)]
#[clap(author = "Byson94")]
#[clap(version, about)]
pub(super) struct RawOpt {
    /// Write out debug logs. (To read the logs, run `ewwii logs`).
    #[arg(long = "debug", global = true)]
    log_debug: bool,

    /// Force ewwii to use wayland. This is a no-op if ewwii was compiled without wayland support.
    #[arg(long = "force-wayland", global = true)]
    force_wayland: bool,

    /// override path to configuration directory (directory that contains ewwii.rhai and eww.(s)css)
    #[arg(short, long, global = true)]
    config: Option<std::path::PathBuf>,

    /// Watch the log output after executing the command
    #[arg(long = "logs", global = true)]
    show_logs: bool,

    /// Avoid daemonizing ewwii.
    #[arg(long = "no-daemonize", global = true)]
    no_daemonize: bool,

    /// Restart the daemon completely before running the command
    #[arg(long = "restart", global = true)]
    restart: bool,

    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug, Serialize, Deserialize, PartialEq)]
pub enum Action {
    /// Generate a shell completion script
    ShellCompletions {
        #[arg(short, long)]
        #[serde(with = "serde_shell")]
        shell: clap_complete::shells::Shell,
    },

    /// Start the Ewwii daemon.
    #[command(name = "daemon", alias = "d")]
    Daemon {
        #[arg(long)]
        with_plugin: Option<String>,
    },

    #[command(flatten)]
    ClientOnly(ActionClientOnly),

    #[command(flatten)]
    WithServer(ActionWithServer),
}

#[derive(Subcommand, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionClientOnly {
    /// Print and watch the ewwii logs
    #[command(name = "logs")]
    Logs,
}

#[derive(Subcommand, Debug, Serialize, Deserialize, PartialEq)]
pub enum ActionWithServer {
    /// Ping the ewwii server, checking if it is reachable.
    #[clap(name = "ping")]
    Ping,

    /// Open the GTK debugger
    #[command(name = "inspector", alias = "debugger")]
    OpenInspector,

    /// Open a window
    #[clap(name = "open", alias = "o")]
    OpenWindow {
        /// Name of the window you want to open.
        window_name: String,

        // The id of the window instance
        #[arg(long)]
        id: Option<String>,

        /// The identifier of the monitor the window should open on
        #[arg(long)]
        screen: Option<MonitorIdentifier>,

        /// The position of the window, where it should open. (i.e.: 200x100)
        #[arg(short, long)]
        pos: Option<Coords>,

        /// The size of the window to open (i.e.: 200x100)
        #[arg(short, long)]
        size: Option<Coords>,

        /// Sidepoint of the window, formatted like "top right"
        #[arg(short, long)]
        anchor: Option<AnchorPoint>,

        /// If the window is already open, close it instead
        #[arg(long = "toggle")]
        should_toggle: bool,

        /// Automatically close the window after a specified amount of time, i.e.: 1s
        #[arg(long, value_parser=parse_duration)]
        duration: Option<std::time::Duration>,
        // /// Define a variable for the window, i.e.: `--arg "var_name=value"`
        // #[arg(long = "arg", value_parser = parse_var_update_arg)]
        // args: Option<Vec<(VarName, DynVal)>>,
    },

    // TODO
    // /// Open multiple windows at once.
    // /// NOTE: This will in the future be part of ewwii open, and will then be removed.
    // #[command(name = "open-many")]
    // OpenMany {
    //     /// List the windows to open, optionally including their id, i.e.: `--window "window_name:window_id"`
    //     #[arg(value_parser = parse_window_config_and_id)]
    //     windows: Vec<(String, String)>,

    //     /// Define a variable for the window, i.e.: `--arg "window_id:var_name=value"`
    //     // #[arg(long = "arg", value_parser = parse_window_id_args)]
    //     // args: Vec<(String, VarName, DynVal)>,

    //     /// If a window is already open, close it instead
    //     #[arg(long = "toggle")]
    //     should_toggle: bool,
    // },
    /// Close the given windows
    #[command(name = "close", alias = "c")]
    CloseWindows { windows: Vec<String> },

    /// Reload the configuration
    #[command(name = "reload", alias = "r")]
    Reload,

    /// Kill the ewwii daemon
    #[command(name = "kill", alias = "k")]
    KillServer,

    /// Close all windows, without killing the daemon
    #[command(name = "close-all", alias = "ca")]
    CloseAll,

    /// Prints all the variables in the registery
    #[command(name = "state")]
    ShowState,
    /// List the names of active windows
    #[command(name = "list-windows")]
    ListWindows,

    /// Show active window IDs, formatted linewise `<window_id>: <window_name>`
    #[command(name = "active-windows")]
    ListActiveWindows,

    /// Print out the widget structure as seen by ewwii.
    ///
    /// This may be useful if you are facing issues with how ewwii is interpreting your configuration,
    /// and to provide additional context to the ewwii developers if you are filing a bug.
    #[command(name = "debug")]
    ShowDebug,
    // /// Print out the scope graph structure in graphviz dot format.
    // #[command(name = "graph")]
    // ShowGraph,
    /// Update the widgets of a particular window. Poll/Listen variables will be cleared
    #[command(name = "update", alias = "u")]
    TriggerUpdateUI {
        /// Inject variables while updating the UI
        ///
        /// Format: --inject foo="val1" baz="val2"
        /// Only variables used by the widget tree will affect the UI.
        #[arg(long = "inject", short = 'i', value_parser = parse_inject_var_map)]
        inject_vars: Option<HashMap<String, String>>,

        /// Preserve the new updates. Only meaningful if used with inject.
        #[arg(long = "preserve", short = 'p')]
        should_preserve_state: bool,
    },

    /// Call rhai functions. (NOTE: All poll/listen will default to their initial value)
    #[command(name = "call-fns")]
    CallRhaiFns {
        /// Rhai functions to call. Format: call-fns "fn_name1(args)" "fn_name2(args)"
        #[arg(required = true)]
        calls: Vec<String>,
    },

    /// Override the default runtime engine settings
    #[command(name = "engine-override")]
    EngineOverride {
        /// Configuration in JSON format
        config_json: String,

        /// Weather to print the current engine settings
        #[arg(long = "sprint", short = 'p')]
        print: bool,
    },

    /// Set a plugin (.so) to the ewwii binary
    #[command(name = "set-plugin")]
    SetPlugin {
        /// The .so file to load
        #[arg(value_parser = absolute_file_path_parser)]
        file_path: String,
    },
}

impl Opt {
    pub fn from_env() -> Self {
        let raw: RawOpt = RawOpt::parse();
        raw.into()
    }
}

impl From<RawOpt> for Opt {
    fn from(other: RawOpt) -> Self {
        let RawOpt { log_debug, force_wayland, config, show_logs, no_daemonize, restart, action } =
            other;
        Opt {
            log_debug,
            force_wayland,
            show_logs,
            restart,
            config_path: config,
            action,
            no_daemonize,
        }
    }
}

/// Parse a window-name:window-id pair of the form `name:id` or `name` into a tuple of `(name, id)`.
// fn parse_window_config_and_id(s: &str) -> Result<(String, String)> {
//     let (name, id) = s.split_once(':').unwrap_or((s, s));

//     Ok((name.to_string(), id.to_string()))
// }

/// Parse a window-id specific variable value declaration with the syntax `window-id:variable_name="new_value"`
/// into a tuple of `(id, variable_name, new_value)`.
// fn parse_window_id_args(s: &str) -> Result<(String, VarName, DynVal)> {
//     // Parse the = first so we know if an id has not been given
//     let (name, value) = parse_var_update_arg(s)?;

//     let (id, var_name) = name.0.split_once(':').unwrap_or(("", &name.0));

//     Ok((id.to_string(), var_name.into(), value))
// }

/// Split the input string at `=`, parsing the value into a [`DynVal`].
// fn parse_var_update_arg(s: &str) -> Result<(VarName, DynVal)> {
//     let (name, value) = s
//         .split_once('=')
//         .with_context(|| format!("arguments must be in the shape `variable_name=\"new_value\"`, but got: {}", s))?;
//     Ok((name.into(), DynVal::from_string(value.to_owned())))
// }

impl ActionWithServer {
    pub fn can_start_daemon(&self) -> bool {
        // matches!(self, ActionWithServer::OpenWindow { .. } | ActionWithServer::OpenMany { .. })
        matches!(self, ActionWithServer::OpenWindow { .. })
    }

    pub fn into_daemon_command(
        self,
    ) -> (app::DaemonCommand, Option<daemon_response::DaemonResponseReceiver>) {
        let command = match self {
            ActionWithServer::TriggerUpdateUI { inject_vars, should_preserve_state } => {
                return with_response_channel(|sender| app::DaemonCommand::TriggerUpdateUI {
                    inject_vars,
                    should_preserve_state,
                    sender,
                })
            }
            ActionWithServer::CallRhaiFns { calls } => {
                return with_response_channel(|sender| app::DaemonCommand::CallRhaiFns {
                    calls,
                    sender,
                })
            }
            ActionWithServer::OpenInspector => app::DaemonCommand::OpenInspector,

            ActionWithServer::KillServer => app::DaemonCommand::KillServer,
            ActionWithServer::CloseAll => app::DaemonCommand::CloseAll,
            ActionWithServer::Ping => {
                let (send, recv) = tokio::sync::mpsc::unbounded_channel();
                let _ = send.send(DaemonResponse::Success("pong".to_owned()));
                return (app::DaemonCommand::NoOp, Some(recv));
            }
            // ActionWithServer::OpenMany { windows, should_toggle } => {
            //     return with_response_channel(|sender| app::DaemonCommand::OpenMany { windows, should_toggle, sender });
            // }
            ActionWithServer::OpenWindow {
                window_name,
                id,
                pos,
                size,
                screen,
                anchor,
                should_toggle,
                duration,
            } => {
                return with_response_channel(|sender| app::DaemonCommand::OpenWindow {
                    window_name,
                    instance_id: id,
                    pos,
                    size,
                    anchor,
                    screen,
                    should_toggle,
                    duration,
                    sender,
                    // args,
                });
            }
            ActionWithServer::CloseWindows { windows } => {
                return with_response_channel(|sender| app::DaemonCommand::CloseWindows {
                    windows,
                    auto_reopen: false,
                    sender,
                });
            }
            ActionWithServer::Reload => {
                return with_response_channel(app::DaemonCommand::ReloadConfigAndCss)
            }
            ActionWithServer::ShowState => {
                return with_response_channel(app::DaemonCommand::ShowState)
            }
            ActionWithServer::ListWindows => {
                return with_response_channel(app::DaemonCommand::ListWindows)
            }
            ActionWithServer::ListActiveWindows => {
                return with_response_channel(app::DaemonCommand::ListActiveWindows)
            }
            ActionWithServer::ShowDebug => {
                return with_response_channel(app::DaemonCommand::PrintDebug)
            }
            ActionWithServer::EngineOverride { config_json, print } => {
                return with_response_channel(|sender| app::DaemonCommand::EngineOverride {
                    config: config_json,
                    print,
                    sender,
                })
            }
            ActionWithServer::SetPlugin { file_path } => {
                return with_response_channel(|sender| app::DaemonCommand::SetPlugin {
                    file_path,
                    sender,
                })
            }
        };
        (command, None)
    }
}

fn with_response_channel<O, F>(
    f: F,
) -> (O, Option<tokio::sync::mpsc::UnboundedReceiver<DaemonResponse>>)
where
    F: FnOnce(DaemonResponseSender) -> O,
{
    let (sender, recv) = daemon_response::create_pair();
    (f(sender), Some(recv))
}

fn parse_duration(s: &str) -> Result<std::time::Duration, crate::dynval::ConversionError> {
    DynVal::from_string(s.to_owned()).as_duration()
}

mod serde_shell {
    use std::str::FromStr as _;

    use clap_complete::Shell;
    use serde::{Deserialize as _, Deserializer, Serialize as _, Serializer};

    pub fn serialize<S: Serializer>(shell: &Shell, serializer: S) -> Result<S::Ok, S::Error> {
        shell.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Shell, D::Error> {
        let s = String::deserialize(deserializer)?;
        Shell::from_str(&s).map_err(serde::de::Error::custom)
    }
}

fn parse_inject_var_map(s: &str) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    let mut chars = s.chars().peekable();
    let mut key = String::new();
    let mut val = String::new();
    let mut in_key = true;
    let mut in_quotes = false;

    while let Some(&c) = chars.peek() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                chars.next();
            }
            '=' if in_key => {
                in_key = false;
                chars.next();
            }
            ',' if !in_key && !in_quotes => {
                map.insert(key.trim().to_string(), val.trim().to_string());
                key.clear();
                val.clear();
                in_key = true;
                chars.next();
            }
            _ => {
                if in_key {
                    key.push(c);
                } else {
                    val.push(c);
                }
                chars.next();
            }
        }
    }

    if !key.is_empty() {
        map.insert(key.trim().to_string(), val.trim().to_string());
    }
    Ok(map)
}

fn absolute_file_path_parser(s: &str) -> Result<String, String> {
    let p = std::path::Path::new(s);
    std::fs::canonicalize(p)
        .map_err(|e| format!("Failed to canonicalize '{}': {}", s, e))
        .map(|abs_path| abs_path.to_string_lossy().into_owned())
}
