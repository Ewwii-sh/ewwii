use crate::diag_error::DiagError;
use crate::{
    daemon_response::DaemonResponseSender,
    display_backend::DisplayBackend,
    error_handling_ctx,
    gtk::prelude::{Cast, ContainerExt, CssProviderExt, GtkWindowExt, MonitorExt, StyleContextExt, WidgetExt},
    paths::EwwPaths,
    widgets::window::Window,
    // dynval::DynVal,
    widgets::{
        build_widget::build_gtk_widget, 
        build_widget::WidgetInput,
        widget_definitions::WidgetRegistry,
    },
    window::{
        coords::Coords,
        monitor::MonitorIdentifier,
        window_geometry::{AnchorPoint, WindowGeometry},
    },
    window_arguments::WindowArguments,
    window_initiator::WindowInitiator,
    *,
};
use anyhow::{anyhow, bail};
use codespan_reporting::files::Files;
use ewwii_shared_util::Span;
use gdk::Monitor;
use glib::ObjectExt;
use gtk::{gdk, glib};
use iirhai::widgetnode::{WidgetNode, get_id_to_props_map};
use itertools::Itertools;
use once_cell::sync::Lazy;
use rhai::{Dynamic, Scope};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    rc::Rc,
};
use tokio::sync::mpsc::UnboundedSender;

/// A command for the ewwii daemon.
/// While these are mostly generated from ewwii CLI commands (see [`opts::ActionWithServer`]),
/// they may also be generated from other places internally.
#[derive(Debug)]
pub enum DaemonCommand {
    NoOp,
    ReloadConfigAndCss(DaemonResponseSender),
    OpenInspector,
    // OpenMany {
    //     windows: Vec<(String, String)>,
    //     // args: Vec<(String, VarName, DynVal)>,
    //     should_toggle: bool,
    //     sender: DaemonResponseSender,
    // },
    OpenWindow {
        window_name: String,
        instance_id: Option<String>,
        pos: Option<Coords>,
        size: Option<Coords>,
        anchor: Option<AnchorPoint>,
        screen: Option<MonitorIdentifier>,
        should_toggle: bool,
        duration: Option<std::time::Duration>,
        sender: DaemonResponseSender,
        // args: Option<Vec<(VarName, DynVal)>>,
    },
    CloseWindows {
        windows: Vec<String>,
        auto_reopen: bool,
        sender: DaemonResponseSender,
    },
    KillServer,
    CloseAll,
    PrintDebug(DaemonResponseSender),
    ListWindows(DaemonResponseSender),
    ListActiveWindows(DaemonResponseSender),
}

/// An opened window.
#[derive(Debug)]
pub struct EwwiiWindow {
    pub name: String,
    pub gtk_window: Window,
    pub destroy_event_handler_id: Option<glib::SignalHandlerId>,
}

impl EwwiiWindow {
    /// Close the GTK window and disconnect the destroy event-handler.
    ///
    /// You need to make sure that the scope get's properly cleaned from the state graph
    /// and that script-vars get cleaned up properly
    pub fn close(self) {
        log::info!("Closing gtk window {}", self.name);
        self.gtk_window.close();
        if let Some(handler_id) = self.destroy_event_handler_id {
            self.gtk_window.disconnect(handler_id);
        }
    }
}

pub struct App<B: DisplayBackend> {
    pub ewwii_config: config::EwwiiConfig,
    /// Map of all currently open windows to their unique IDs
    /// If no specific ID was specified whilst starting the window,
    /// it will be the same as the window name.
    /// Therefore, only one window of a given name can exist when not using IDs.
    pub open_windows: HashMap<String, EwwiiWindow>,
    pub instance_id_to_args: HashMap<String, WindowArguments>,
    /// Window names that are supposed to be open, but failed.
    /// When reloading the config, these should be opened again.
    pub failed_windows: HashSet<String>,
    pub css_provider: gtk::CssProvider,

    /// Sender to send [`DaemonCommand`]s
    pub app_evt_send: UnboundedSender<DaemonCommand>,

    /// Senders that will cancel a windows auto-close timer when started with --duration.
    pub window_close_timer_abort_senders: HashMap<String, futures::channel::oneshot::Sender<()>>,

    pub paths: EwwPaths,
    pub phantom: PhantomData<B>,
}

impl<B: DisplayBackend> std::fmt::Debug for App<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("ewwii_config", &self.ewwii_config)
            .field("open_windows", &self.open_windows)
            .field("failed_windows", &self.failed_windows)
            .field("window_arguments", &self.instance_id_to_args)
            .field("paths", &self.paths)
            .finish()
    }
}

/// Wait until the .model() is available for all monitors (or there is a timeout)
async fn wait_for_monitor_model() {
    let display = gdk::Display::default().expect("could not get default display");
    let start = std::time::Instant::now();
    loop {
        let all_monitors_set =
            (0..display.n_monitors()).all(|i| display.monitor(i).and_then(|monitor| monitor.model()).is_some());
        if all_monitors_set {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
        if std::time::Instant::now() - start > Duration::from_millis(500) {
            log::warn!("Timed out waiting for monitor model to be set");
            break;
        }
    }
}

impl<B: DisplayBackend> App<B> {
    /// Handle a [`DaemonCommand`] event, logging any errors that occur.
    pub async fn handle_command(&mut self, event: DaemonCommand) {
        if let Err(err) = self.try_handle_command(event).await {
            error_handling_ctx::print_error(err);
        }
    }

    /// Try to handle a [`DaemonCommand`] event.
    async fn try_handle_command(&mut self, event: DaemonCommand) -> Result<()> {
        log::debug!("Handling event: {:?}", &event);
        match event {
            DaemonCommand::NoOp => {}
            DaemonCommand::OpenInspector => {
                gtk::Window::set_interactive_debugging(true);
            }
            DaemonCommand::ReloadConfigAndCss(sender) => {
                // Wait for all monitor models to be set. When a new monitor gets added, this
                // might not immediately be the case. And if we were to wait inside the
                // connect_monitor_added callback, model() never gets set. So instead we wait here.
                wait_for_monitor_model().await;
                let mut errors = Vec::new();

                let config_result = config::read_from_ewwii_paths(&self.paths);
                if let Err(e) = config_result.and_then(|new_config| self.load_config(new_config)) {
                    errors.push(e)
                }
                match crate::config::scss::parse_scss_from_config(self.paths.get_config_dir()) {
                    Ok((file_id, css)) => {
                        if let Err(e) = self.load_css(file_id, &css) {
                            errors.push(anyhow!(e));
                        }
                    }
                    Err(e) => {
                        errors.push(e);
                    }
                }

                sender.respond_with_error_list(errors)?;
            }
            DaemonCommand::KillServer => {
                log::info!("Received kill command, stopping server!");
                self.stop_application();
            }
            DaemonCommand::CloseAll => {
                log::info!("Received close command, closing all windows");
                for window_name in self.open_windows.keys().cloned().collect::<Vec<String>>() {
                    self.close_window(&window_name, false)?;
                }
            }
            // DaemonCommand::OpenMany { windows, should_toggle, sender } => {
            //     let errors = windows
            //         .iter()
            //         .map(|w| {
            //             let (config_name, id) = w;
            //             if should_toggle && self.open_windows.contains_key(id) {
            //                 self.close_window(id, false)
            //             } else {
            //                 log::debug!("Config: {}, id: {}", config_name, id);
            //                 let window_args = args
            //                     .iter()
            //                     .filter(|(win_id, ..)| win_id.is_empty() || win_id == id)
            //                     .map(|(_, n, v)| (n.clone(), v.clone()))
            //                     .collect();
            //                 self.open_window(&WindowArguments::new_from_args(id.to_string(), config_name.clone())?)
            //             }
            //         })
            //         .filter_map(Result::err);
            //     sender.respond_with_error_list(errors)?;
            // }
            DaemonCommand::OpenWindow {
                window_name,
                instance_id,
                pos,
                size,
                anchor,
                screen: monitor,
                should_toggle,
                duration,
                sender,
                // args,
            } => {
                let instance_id = instance_id.unwrap_or_else(|| window_name.clone());

                let is_open = self.open_windows.contains_key(&instance_id);

                let result = if should_toggle && is_open {
                    self.close_window(&instance_id, false)
                } else {
                    self.open_window(&WindowArguments { instance_id, window_name, pos, size, monitor, anchor, duration })
                };

                sender.respond_with_result(result)?;
            }
            DaemonCommand::CloseWindows { windows, auto_reopen, sender } => {
                let errors = windows.iter().map(|window| self.close_window(window, auto_reopen)).filter_map(Result::err);
                // Ignore sending errors, as the channel might already be closed
                let _ = sender.respond_with_error_list(errors);
            }
            DaemonCommand::ListWindows(sender) => {
                let output = self.ewwii_config.get_windows().keys().join("\n");
                sender.send_success(output)?
            }
            DaemonCommand::ListActiveWindows(sender) => {
                let output = self.open_windows.iter().map(|(id, window)| format!("{id}: {}", window.name)).join("\n");
                sender.send_success(output)?
            }
            DaemonCommand::PrintDebug(sender) => {
                let output = format!("{:#?}", &self);
                sender.send_success(output)?
            }
        }
        Ok(())
    }

    /// Fully stop ewwii:
    /// close all windows, stop the script_var_handler, quit the gtk appliaction and send the exit instruction to the lifecycle manager
    fn stop_application(&mut self) {
        for (_, window) in self.open_windows.drain() {
            window.close();
        }
        gtk::main_quit();
        let _ = crate::application_lifecycle::send_exit();
    }

    /// Close a window and do all the required cleanups in the scope_graph and script_var_handler
    fn close_window(&mut self, instance_id: &str, auto_reopen: bool) -> Result<()> {
        if let Some(old_abort_send) = self.window_close_timer_abort_senders.remove(instance_id) {
            _ = old_abort_send.send(());
        }
        let ewwii_window = self
            .open_windows
            .remove(instance_id)
            .with_context(|| format!("Tried to close window with id '{instance_id}', but no such window was open"))?;

        // let scope_index = ewwii_window.scope_index;
        ewwii_window.close();

        if auto_reopen {
            self.failed_windows.insert(instance_id.to_string());
            // There might be an alternative monitor available already, so try to re-open it immediately.
            // This can happen for example when a monitor gets disconnected and another connected,
            // and the connection event happens before the disconnect.
            if let Some(window_arguments) = self.instance_id_to_args.get(instance_id) {
                let _ = self.open_window(&window_arguments.clone());
            }
        } else {
            self.instance_id_to_args.remove(instance_id);
        }

        Ok(())
    }

    fn open_window(&mut self, window_args: &WindowArguments) -> Result<()> {
        let instance_id = &window_args.instance_id;
        self.failed_windows.remove(instance_id);
        log::info!("Opening window {} as '{}'", window_args.window_name, instance_id);

        // if an instance of this is already running, close it
        if self.open_windows.contains_key(instance_id) {
            self.close_window(instance_id, false)?;
        }

        self.instance_id_to_args.insert(instance_id.to_string(), window_args.clone());

        let open_result: Result<_> = (|| {
            let window_name: &str = &window_args.window_name;

            let window_def = self.ewwii_config.get_window(window_name)?.clone();
            assert_eq!(window_def.name, window_name, "window definition name did not equal the called window");

            let initiator = WindowInitiator::new(&window_def, window_args)?;

            /// Holds the id and the props of a widget
            /// It is crutual for supporting dynamic updates
            let mut widget_reg_store = WidgetRegistry::new(); 
            // note for future me: ^ this might need cloning.

            let root_widget = build_gtk_widget(
                WidgetInput::Window(window_def), 
                &mut widget_reg_store
            )?;

            root_widget.style_context().add_class(window_name);

            let monitor = get_gdk_monitor(initiator.monitor.clone())?;
            let mut ewwii_window = initialize_window::<B>(&initiator, monitor, root_widget)?;
            ewwii_window.gtk_window.style_context().add_class(window_name);

            // listening/polling
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
            let (update_sender, update_receiver) = glib::MainContext::channel(0.into());
            let config_path = self.paths.get_rhai_path();
            let store = iirhai::updates::handle_state_changes(self.ewwii_config.get_root_node()?, tx);

            glib::MainContext::default().spawn_local(async move {
                while let Some(var_name) = rx.recv().await {
                    log::debug!("Received update for var: {}", var_name);
                    let vars = store.read().unwrap().clone();
                    
                    match generate_new_widgetnode(&vars, &config_path).await {
                        Ok(new_widget) => {
                            if let Err(e) = update_sender.send(new_widget) {
                                log::warn!("Failed to send new widget update: {:?}", e);
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to generate new widgetnode: {:?}", e);
                        }
                    }
                }
                log::debug!("Receiver loop exited");
            });

            ewwii_window.destroy_event_handler_id = Some(ewwii_window.gtk_window.connect_destroy({
                let app_evt_sender = self.app_evt_send.clone();
                let instance_id = instance_id.to_string();
                move |_| {
                    // we don't care about the actual error response from the daemon as this is mostly just a fallback.
                    // Generally, this should get disconnected before the gtk window gets destroyed.
                    // This callback is triggered in 2 cases:
                    // - When the monitor of this window gets disconnected
                    // - When the window is closed manually.
                    // We don't distinguish here and assume the window should be reopened once a monitor
                    // becomes available again
                    let (response_sender, _) = daemon_response::create_pair();
                    let command = DaemonCommand::CloseWindows {
                        windows: vec![instance_id.clone()],
                        auto_reopen: true,
                        sender: response_sender,
                    };
                    if let Err(err) = app_evt_sender.send(command) {
                        log::error!("Error sending close window command to daemon after gtk window destroy event: {}", err);
                    }
                }
            }));

            update_receiver.attach(None, move |new_root_widget| {
                let mut id_to_prop = HashMap::new();
                let _ = get_id_to_props_map(&new_root_widget, &mut id_to_prop);

                widget_reg_store.update_prop_changes(id_to_prop);

                // for child in container_for_task.children() {
                //     container_for_task.remove(&child);
                // }

                // match new_root_widget {
                //     Ok(node) => {
                //         let gtk_widget: gtk::Widget =
                //             build_gtk_widget(WidgetInput::Node(node)).expect("Unable to create the gtk widget.");
                //         container_for_task.add(&gtk_widget);
                //         container_for_task.show_all();
                //     }
                //     Err(err) => {
                //         eprintln!("Widget render failed: {:?}", err);
                //     }
                // }

                glib::ControlFlow::Continue
            });

            let duration = window_args.duration;
            if let Some(duration) = duration {
                let app_evt_sender = self.app_evt_send.clone();

                let (abort_send, abort_recv) = futures::channel::oneshot::channel();

                glib::MainContext::default().spawn_local({
                    let instance_id = instance_id.to_string();
                    async move {
                        tokio::select! {
                            _ = glib::timeout_future(duration) => {
                                let (response_sender, mut response_recv) = daemon_response::create_pair();
                                let command = DaemonCommand::CloseWindows { windows: vec![instance_id.clone()], auto_reopen: false, sender: response_sender };
                                if let Err(err) = app_evt_sender.send(command) {
                                    log::error!("Error sending close window command to daemon after gtk window destroy event: {}", err);
                                }
                                _ = response_recv.recv().await;
                            }
                            _ = abort_recv => {}
                        }
                    }
                });

                if let Some(old_abort_send) = self.window_close_timer_abort_senders.insert(instance_id.to_string(), abort_send) {
                    _ = old_abort_send.send(());
                }
            }

            self.open_windows.insert(instance_id.to_string(), ewwii_window);
            Ok(())
        })();

        if let Err(err) = open_result {
            self.failed_windows.insert(instance_id.to_string());
            Err(err).with_context(|| format!("failed to open window `{}`", instance_id))
        } else {
            Ok(())
        }
    }

    /// Load the given configuration, reloading all script-vars and attempting to reopen all windows that where opened.
    pub fn load_config(&mut self, config: config::EwwiiConfig) -> Result<()> {
        log::info!("Reloading windows");
        log::trace!("loading config: {:#?}", config);

        self.ewwii_config = config;

        let open_window_ids: Vec<String> =
            self.open_windows.keys().cloned().chain(self.failed_windows.iter().cloned()).dedup().collect();
        for instance_id in &open_window_ids {
            let window_arguments = self.instance_id_to_args.get(instance_id).with_context(|| {
                format!("Cannot reopen window, initial parameters were not saved correctly for {instance_id}")
            })?;
            self.open_window(&window_arguments.clone())?;
        }
        Ok(())
    }

    /// Load a given CSS string into the gtk css provider, returning a nicely formatted [`DiagError`] when GTK errors out
    pub fn load_css(&mut self, file_id: usize, css: &str) -> Result<()> {
        if let Err(err) = self.css_provider.load_from_data(css.as_bytes()) {
            static PATTERN: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"[^:]*:(\d+):(\d+)(.*)$").unwrap());
            let nice_error_option: Option<_> = (|| {
                let captures = PATTERN.captures(err.message())?;
                let line = captures.get(1).unwrap().as_str().parse::<usize>().ok()?;
                let msg = captures.get(3).unwrap().as_str();
                let db = error_handling_ctx::FILE_DATABASE.read().ok()?;
                let line_range = db.line_range(file_id, line - 1).ok()?;
                let span = Span(line_range.start, line_range.end - 1, file_id);
                Some(DiagError(gen_diagnostic!(msg, span)))
            })();
            match nice_error_option {
                Some(error) => Err(anyhow!(error)),
                None => Err(anyhow!("CSS error: {}", err.message())),
            }
        } else {
            Ok(())
        }
    }
}

fn initialize_window<B: DisplayBackend>(
    window_init: &WindowInitiator,
    monitor: Monitor,
    root_widget: gtk::Widget,
) -> Result<EwwiiWindow> {
    let monitor_geometry = monitor.geometry();
    let (actual_window_rect, x, y) = match window_init.geometry {
        Some(geometry) => {
            let rect = get_window_rectangle(geometry, monitor_geometry);
            (Some(rect), rect.x(), rect.y())
        }
        _ => (None, 0, 0),
    };
    let window = B::initialize_window(window_init, monitor_geometry, x, y)
        .with_context(|| format!("monitor {} is unavailable", window_init.monitor.clone().unwrap()))?;

    window.set_title(&format!("Ewwii - {}", window_init.name));
    window.set_position(gtk::WindowPosition::None);
    window.set_gravity(gdk::Gravity::Center);

    if let Some(actual_window_rect) = actual_window_rect {
        window.set_size_request(actual_window_rect.width(), actual_window_rect.height());
        window.set_default_size(actual_window_rect.width(), actual_window_rect.height());
    }
    window.set_decorated(false);
    window.set_skip_taskbar_hint(true);
    window.set_skip_pager_hint(true);

    // run on_screen_changed to set the visual correctly initially.
    on_screen_changed(&window, None);
    window.connect_screen_changed(on_screen_changed);

    window.add(&root_widget);

    window.realize();

    #[cfg(feature = "x11")]
    if B::IS_X11 {
        if let Some(geometry) = window_init.geometry {
            let _ = apply_window_position(geometry, monitor_geometry, &window);
            if window_init.backend_options.x11.window_type != crate::window::backend_window_options::X11WindowType::Normal {
                let last_pos = Rc::new(RefCell::new(None));
                window.connect_configure_event({
                    let last_pos = last_pos.clone();
                    move |window, _| {
                        let gdk_window = window.window().unwrap();
                        let current_origin = gdk_window.origin();

                        let mut last = last_pos.borrow_mut();
                        if Some((current_origin.1, current_origin.2)) != *last {
                            *last = Some((current_origin.1, current_origin.2));
                            let _ = apply_window_position(geometry, monitor_geometry, window);
                        }

                        false
                    }
                });
            }
        }
        display_backend::set_xprops(&window, monitor, window_init)?;
    }

    window.show_all();

    Ok(EwwiiWindow { 
        name: window_init.name.clone(), 
        gtk_window: window, 
        destroy_event_handler_id: None 
    })
}

async fn generate_new_widgetnode(all_vars: &HashMap<String, String>, code_path: &Path) -> Result<WidgetNode> {
    let mut scope = Scope::new();
    for (name, val) in all_vars {
        scope.set_value(name.clone(), Dynamic::from(val.clone()));
    }

    if !code_path.exists() {
        bail!("The configuration file `{}` does not exist", code_path.display());
    }

    let mut reeval_parser = iirhai::parser::ParseConfig::new();
    let new_root_widget = reeval_parser.eval_file_with(code_path, scope, None);

    Ok(config::EwwiiConfig::get_windows_root_widget(new_root_widget?)?)
}

/// Apply the provided window-positioning rules to the window.
#[cfg(feature = "x11")]
fn apply_window_position(mut window_geometry: WindowGeometry, monitor_geometry: gdk::Rectangle, window: &Window) -> Result<()> {
    let gdk_window = window.window().context("Failed to get gdk window from gtk window")?;
    window_geometry.size = crate::window::window_geometry::Coords::from_pixels(window.size());
    let actual_window_rect = get_window_rectangle(window_geometry, monitor_geometry);

    let gdk_origin = gdk_window.origin();

    if actual_window_rect.x() != gdk_origin.1 || actual_window_rect.y() != gdk_origin.2 {
        gdk_window.move_(actual_window_rect.x(), actual_window_rect.y());
    }

    Ok(())
}

fn on_screen_changed(window: &Window, _old_screen: Option<&gdk::Screen>) {
    let visual = gtk::prelude::GtkWindowExt::screen(window)
        .and_then(|screen| screen.rgba_visual().filter(|_| screen.is_composited()).or_else(|| screen.system_visual()));
    window.set_visual(visual.as_ref());
}

/// Get the monitor geometry of a given monitor, or the default if none is given
fn get_gdk_monitor(identifier: Option<MonitorIdentifier>) -> Result<Monitor> {
    let display = gdk::Display::default().expect("could not get default display");
    let monitor = match identifier {
        Some(ident) => {
            let mon = get_monitor_from_display(&display, &ident);
            mon.with_context(|| {
                let head = format!("Failed to get monitor {}\nThe available monitors are:", ident);
                let mut body = String::new();
                for m in 0..display.n_monitors() {
                    if let Some(model) = display.monitor(m).and_then(|x| x.model()) {
                        body.push_str(format!("\n\t[{}] {}", m, model).as_str());
                    }
                }
                format!("{}{}", head, body)
            })?
        }
        None => display
            .primary_monitor()
            .context("Failed to get primary monitor from GTK. Try explicitly specifying the monitor on your window.")?,
    };
    Ok(monitor)
}

/// Get the name of monitor plug for given monitor number
/// workaround gdk not providing this information on wayland in regular calls
/// gdk_screen_get_monitor_plug_name is deprecated but works fine for that case
fn get_monitor_plug_name(display: &gdk::Display, monitor_num: i32) -> Option<&str> {
    unsafe {
        use glib::translate::ToGlibPtr;
        let plug_name_pointer = gdk_sys::gdk_screen_get_monitor_plug_name(display.default_screen().to_glib_none().0, monitor_num);
        use std::ffi::CStr;
        CStr::from_ptr(plug_name_pointer).to_str().ok()
    }
}

/// Returns the [Monitor][gdk::Monitor] structure corresponding to the identifer.
/// Outside of x11, only [MonitorIdentifier::Numeric] is supported
pub fn get_monitor_from_display(display: &gdk::Display, identifier: &MonitorIdentifier) -> Option<gdk::Monitor> {
    match identifier {
        MonitorIdentifier::List(list) => {
            for ident in list {
                if let Some(monitor) = get_monitor_from_display(display, ident) {
                    return Some(monitor);
                }
            }
            None
        }
        MonitorIdentifier::Primary => display.primary_monitor(),
        MonitorIdentifier::Numeric(num) => display.monitor(*num),
        MonitorIdentifier::Name(name) => {
            for m in 0..display.n_monitors() {
                if let Some(model) = display.monitor(m).and_then(|x| x.model()) {
                    if model == *name || Some(name.as_str()) == get_monitor_plug_name(display, m) {
                        return display.monitor(m);
                    }
                }
            }
            None
        }
    }
}

pub fn get_window_rectangle(geometry: WindowGeometry, screen_rect: gdk::Rectangle) -> gdk::Rectangle {
    let (offset_x, offset_y) = geometry.offset.relative_to(screen_rect.width(), screen_rect.height());
    let (width, height) = geometry.size.relative_to(screen_rect.width(), screen_rect.height());
    let x = screen_rect.x() + offset_x + geometry.anchor_point.x.alignment_to_coordinate(width, screen_rect.width());
    let y = screen_rect.y() + offset_y + geometry.anchor_point.y.alignment_to_coordinate(height, screen_rect.height());
    gdk::Rectangle::new(x, y, width, height)
}
