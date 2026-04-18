use crate::{
    config::ewwii_config::{ConfigEngine, EWWII_CONFIG_PARSER},
    daemon_response::DaemonResponseSender,
    display_backend::DisplayBackend,
    error_handling_ctx,
    gtk4::prelude::{
        Cast, CastNone, DisplayExt, GtkWindowExt, ListModelExt, MonitorExt, NativeExt, ObjectExt,
        WidgetExt,
    },
    paths::EwwiiPaths,
    // dynval::DynVal,
    widgets::{
        build_widget::build_gtk_widget, build_widget::WidgetInput,
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
use anyhow::anyhow;
use ewwii_plugin_api as epapi;
use ewwii_rhai_impl::updates::api::VarWatcherAPI;
use gdk::Monitor;
use gtk4::Window;
use gtk4::{gdk, glib};
use itertools::Itertools;
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    path::PathBuf,
    rc::Rc,
    sync::Mutex,
};
use tokio::sync::mpsc::UnboundedSender;

fn register_active_plugin(lib: libloading::Library, id: String, version: String) -> Result<()> {
    let mut plugins =
        plugin::ACTIVE_PLUGINS.write().map_err(|_| anyhow!("Plugin registry is poisoned!"))?;

    if plugins.iter().any(|p| p.id == id) {
        return Err(anyhow!("Plugin with ID {} is already loaded", id));
    }

    plugins.push(plugin::ActivePlugin { library: lib, id, version });

    Ok(())
}

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
    ShowState(DaemonResponseSender),
    ListWindows(DaemonResponseSender),
    ListActiveWindows(DaemonResponseSender),
    ListPlugins(DaemonResponseSender),
    WidgetControl {
        action: crate::opts::WidgetControlAction,
        sender: DaemonResponseSender,
    },
    Update {
        mappings: HashMap<String, String>,
        sender: DaemonResponseSender,
    },
    CallRhaiFns {
        calls: Vec<String>,
        sender: DaemonResponseSender,
    },
}

/// An opened window.
pub struct EwwiiWindow {
    pub name: String,
    pub gtk_window: Window,
    pub delete_event_handler_id: Option<glib::SignalHandlerId>,
    pub destroy_event_handler_id: Option<glib::SignalHandlerId>,
}

impl std::fmt::Debug for EwwiiWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EwwiiWindow")
            .field("name", &self.name)
            .field("gtk_window", &"<GtkWindow>")
            .field("widget_reg_store", &"<WidgetRegistry>")
            .field("delete_event_handler_id", &self.delete_event_handler_id)
            .field("destroy_event_handler_id", &self.destroy_event_handler_id)
            .finish()
    }
}

impl EwwiiWindow {
    /// Close the GTK window and disconnect the destroy event-handler.
    ///
    /// You need to make sure that the window gets propery cleaned from gtk4!
    pub fn close(self) {
        log::info!("Closing gtk window {}", self.name);

        for handler_id_opt in [self.destroy_event_handler_id, self.delete_event_handler_id] {
            if let Some(handler_id) = handler_id_opt {
                self.gtk_window.disconnect(handler_id);
            }
        }

        self.gtk_window.close();
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
    pub css_provider: gtk4::CssProvider,
    pub reloading: bool,

    /// Sender to send [`DaemonCommand`]s
    pub app_evt_send: UnboundedSender<DaemonCommand>,

    /// Senders that will cancel a windows auto-close timer when started with --duration.
    pub window_close_timer_abort_senders: HashMap<String, futures::channel::oneshot::Sender<()>>,

    /// The dynamic gtk widget registery
    pub widget_reg_store: Rc<Mutex<Option<WidgetRegistry>>>,

    pub paths: EwwiiPaths,
    pub gtk_main_loop: gtk4::glib::MainLoop,
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
        let monitors_model = display.monitors();
        let n_monitors = monitors_model.n_items();

        let all_monitors_set = (0..n_monitors).all(|i| {
            if let Some(obj) = monitors_model.item(i) {
                // Downcast GObject to Monitor
                let monitor: Monitor = obj.downcast().unwrap();
                monitor.model().is_some()
            } else {
                false
            }
        });
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
                gtk4::Window::set_interactive_debugging(true);
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
                    self.open_window(&WindowArguments {
                        instance_id,
                        window_name,
                        pos,
                        size,
                        monitor,
                        anchor,
                        duration,
                    })
                };

                sender.respond_with_result(result)?;
            }
            DaemonCommand::CloseWindows { windows, auto_reopen, sender } => {
                let errors = windows
                    .iter()
                    .map(|window| self.close_window(window, auto_reopen))
                    .filter_map(Result::err);
                // Ignore sending errors, as the channel might already be closed
                let _ = sender.respond_with_error_list(errors);
            }
            DaemonCommand::ListWindows(sender) => {
                let output = self.ewwii_config.get_windows().keys().join("\n");
                sender.send_success(output)?
            }
            DaemonCommand::ListActiveWindows(sender) => {
                let output = self
                    .open_windows
                    .iter()
                    .map(|(id, window)| format!("{id}: {}", window.name))
                    .join("\n");
                sender.send_success(output)?
            }
            DaemonCommand::ListPlugins(sender) => {
                let plugins_guard = match plugin::ACTIVE_PLUGINS.read() {
                    Ok(guard) => guard,
                    Err(_) => {
                        sender.send_failure("Failed to acquire plugin lock".to_string())?;
                        return Ok(());
                    }
                };

                let output: String =
                    plugins_guard.iter().map(|p| format!("{} (v{})", p.id, p.version)).join("\n");

                sender.send_success(output)?
            }
            DaemonCommand::PrintDebug(sender) => {
                let output = format!("{:#?}", &self);
                sender.send_success(output)?
            }
            DaemonCommand::ShowState(sender) => {
                let output =
                    format!("{:#?}", ewwii_rhai_impl::updates::api::VarWatcherAPI::state());
                sender.send_success(output)?
            }
            DaemonCommand::Update { mappings, sender } => {
                match self.update_variables(mappings) {
                    Ok(_) => sender.send_success(String::new())?,
                    Err(e) => sender.send_failure(e.to_string())?,
                };
            }
            DaemonCommand::WidgetControl { action, sender } => {
                match self.perform_widget_control(action) {
                    Ok(s) => sender.send_success(s)?,
                    Err(e) => sender.send_failure(e.to_string())?,
                };
            }
            DaemonCommand::CallRhaiFns { calls, sender } => {
                match self.call_rhai_fns(calls) {
                    Ok(_) => sender.send_success(String::new())?,
                    Err(e) => sender.send_failure(e.to_string())?,
                };
            }
        }
        Ok(())
    }

    /// Fully stop ewwii:
    /// close all windows, kill the poll/listen state handler, quit the gtk appliaction and send the exit instruction to the lifecycle manager
    fn stop_application(&mut self) {
        ewwii_rhai_impl::updates::kill_state_change_handler();
        for (_, window) in self.open_windows.drain() {
            window.close();
        }
        self.gtk_main_loop.quit();
        let _ = crate::application_lifecycle::send_exit();
    }

    /// Close a window
    fn close_window(&mut self, instance_id: &str, auto_reopen: bool) -> Result<()> {
        if let Some(old_abort_send) = self.window_close_timer_abort_senders.remove(instance_id) {
            _ = old_abort_send.send(());
        }
        let ewwii_window = self.open_windows.remove(instance_id).with_context(|| {
            format!("Tried to close window with id '{instance_id}', but no such window was open")
        })?;

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

        // stop poll/listen handlers if no windows are open
        if self.open_windows.is_empty() || self.reloading {
            log::trace!("Killing ewwii state change handler.");
            ewwii_rhai_impl::updates::kill_state_change_handler();
            VarWatcherAPI::clear_all()
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
            assert_eq!(
                window_def.name, window_name,
                "window definition name did not equal the called window"
            );

            let initiator = WindowInitiator::new(&window_def, window_args)?;

            if self.open_windows.is_empty() || self.reloading {
                // Start the global variables
                let signals_vec = ewwii_rhai_impl::updates::retreive_signals(
                    self.ewwii_config.get_root_node()?.as_ref(),
                );

                ewwii_rhai_impl::updates::handle_state_changes(signals_vec);
            }

            let root_widget = {
                // builds the widget and populates widget registry
                let mut maybe_registry = self.widget_reg_store.lock().unwrap();
                let registry = maybe_registry.get_or_insert_with(WidgetRegistry::new);
                build_gtk_widget(&WidgetInput::Window(window_def), registry)?
            };

            root_widget.add_css_class(window_name);

            let monitor = get_gdk_monitor(initiator.monitor.clone())?;
            let mut ewwii_window = initialize_window::<B>(&initiator, monitor, root_widget)?;

            let gtk_close_handler = {
                let app_evt_sender = self.app_evt_send.clone();
                let instance_id = instance_id.to_string();
                // we don't care about the actual error response from the daemon as this is mostly just a fallback.
                // Generally, this should get disconnected before the gtk window gets destroyed.
                // This callback is triggered in 2 cases:
                // - When the monitor of this window gets disconnected
                // - When the window is closed manually.
                move |auto_reopen| {
                    let (response_sender, _) = daemon_response::create_pair();
                    let command = DaemonCommand::CloseWindows {
                        windows: vec![instance_id.clone()],
                        auto_reopen,
                        sender: response_sender,
                    };
                    if let Err(err) = app_evt_sender.send(command) {
                        log::error!("Error sending close window command: {}", err);
                    }
                }
            };

            let closed_by_user = Rc::new(Cell::new(false));

            // handling users close request
            ewwii_window.delete_event_handler_id =
                Some(ewwii_window.gtk_window.connect_close_request({
                    let handler = gtk_close_handler.clone();
                    let closed_by_user = closed_by_user.clone();
                    move |_| {
                        handler(false); // -- false: don't reopen window to respect users intent
                        closed_by_user.set(true);
                        glib::Propagation::Proceed
                    }
                }));

            // handling destory request
            ewwii_window.destroy_event_handler_id =
                Some(ewwii_window.gtk_window.connect_destroy({
                    let handler = gtk_close_handler.clone();
                    let closed_by_user = closed_by_user.clone();
                    move |_| {
                        if !closed_by_user.get() {
                            handler(true);
                        }
                    }
                }));

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

                if let Some(old_abort_send) = self
                    .window_close_timer_abort_senders
                    .insert(instance_id.to_string(), abort_send)
                {
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

        self.reloading = true;
        let result = (|| -> Result<()> {
            self.ewwii_config.replace_data(config);

            let open_window_ids: Vec<String> = self
                .open_windows
                .keys()
                .cloned()
                .chain(self.failed_windows.iter().cloned())
                .dedup()
                .collect();

            for instance_id in &open_window_ids {
                let window_arguments = self.instance_id_to_args.get(instance_id).with_context(|| {
                    format!("Cannot reopen window, initial parameters were not saved correctly for {instance_id}")
                })?;
                self.open_window(&window_arguments.clone())?;
            }

            Ok(())
        })();
        self.reloading = false;

        result
    }

    /// Load a given CSS string into the gtk css provider
    pub fn load_css(&mut self, _file_id: usize, css: &str) -> Result<()> {
        self.css_provider.load_from_string(&css);

        Ok(())
    }

    /// Perform widget control based on the action
    pub fn perform_widget_control(
        &mut self,
        action: crate::opts::WidgetControlAction,
    ) -> Result<String> {
        match action {
            crate::opts::WidgetControlAction::Remove { names } => {
                if let Ok(mut maybe_registry) = self.widget_reg_store.lock() {
                    if let Some(widget_registry) = maybe_registry.as_mut() {
                        for name in names {
                            widget_registry.remove_widget_by_name(&name);
                        }
                    } else {
                        log::error!("Widget registry is empty");
                    }
                } else {
                    log::error!("Failed to acquire lock on widget registry");
                }
            }
            crate::opts::WidgetControlAction::Create { rhai_codes, parent_name } => {
                for rhai_code in rhai_codes {
                    let widget_node = EWWII_CONFIG_PARSER.with(|p| {
                        let mut parser = p.borrow_mut();
                        match parser.as_mut().unwrap() {
                            ConfigEngine::Default(rhai) => rhai.eval_code_snippet(&rhai_code),
                            ConfigEngine::Custom(_) => Err(anyhow::anyhow!(
                                "Dynamic widget creation is only supported with the Rhai config engine"
                            )),
                        }
                    })?;
                    let wid = ewwii_shared_utils::ast::hash_props(widget_node.props().ok_or_else(
                        || anyhow::anyhow!("Failed to retreive the properties of this widget."),
                    )?);

                    if let Ok(mut maybe_registry) = self.widget_reg_store.lock() {
                        if let Some(widget_registry) = maybe_registry.as_mut() {
                            let pid =
                                widget_registry.get_widget_id_by_name(&parent_name).ok_or_else(
                                    || anyhow::anyhow!("Widget '{}' not found", parent_name),
                                )?;
                            widget_registry.create_widget(&widget_node, wid, pid)?;
                        } else {
                            log::error!("Widget registry is empty");
                        }
                    } else {
                        log::error!("Failed to acquire lock on widget registry");
                    }
                }
            }
            crate::opts::WidgetControlAction::PropertyGet { property, widget_name } => {
                if let Ok(mut maybe_registry) = self.widget_reg_store.lock() {
                    if let Some(widget_registry) = maybe_registry.as_mut() {
                        let property_value = widget_registry
                            .get_property_by_name(&widget_name, &property)
                            .ok_or_else(|| {
                                anyhow::anyhow!("Property '{}' not found or wrong type", property)
                            })?;

                        return Ok(property_value);
                    } else {
                        log::error!("Widget registry is empty");
                    }
                } else {
                    log::error!("Failed to acquire lock on widget registry");
                }
            }
            crate::opts::WidgetControlAction::PropertyUpdate {
                property_and_value,
                widget_name,
            } => {
                if let Ok(mut maybe_registry) = self.widget_reg_store.lock() {
                    if let Some(widget_registry) = maybe_registry.as_mut() {
                        for (key, value) in &property_and_value {
                            widget_registry.update_property_by_name(
                                &widget_name,
                                (key.clone(), value.clone()),
                            );
                        }
                    } else {
                        log::error!("Widget registry is empty");
                    }
                } else {
                    log::error!("Failed to acquire lock on widget registry");
                }
            }
            crate::opts::WidgetControlAction::AddClass { class, widget_name } => {
                if let Ok(mut maybe_registry) = self.widget_reg_store.lock() {
                    if let Some(widget_registry) = maybe_registry.as_mut() {
                        widget_registry.update_class_of_widget_by_name(&widget_name, &class, false);
                    } else {
                        log::error!("Widget registry is empty");
                    }
                } else {
                    log::error!("Failed to acquire lock on widget registry");
                }
            }
            crate::opts::WidgetControlAction::RemoveClass { class, widget_name } => {
                if let Ok(mut maybe_registry) = self.widget_reg_store.lock() {
                    if let Some(widget_registry) = maybe_registry.as_mut() {
                        widget_registry.update_class_of_widget_by_name(&widget_name, &class, true);
                    } else {
                        log::error!("Widget registry is empty");
                    }
                } else {
                    log::error!("Failed to acquire lock on widget registry");
                }
            }
        }

        Ok(String::new())
    }

    /// Update variables based on the mappings provided
    pub fn update_variables(&mut self, mappings: HashMap<String, String>) -> Result<()> {
        for (variable, value) in mappings {
            VarWatcherAPI::update_with_broadcast(&variable, value);
        }

        Ok(())
    }

    pub fn call_rhai_fns(&self, calls: Vec<String>) -> Result<()> {
        EWWII_CONFIG_PARSER.with(move |p| -> anyhow::Result<()> {
            let parser = p.borrow();
            match parser.as_ref().unwrap() {
                ConfigEngine::Default(rhai) => {
                    for fn_call in calls {
                        rhai.call_rhai_fn(&fn_call, None)?;
                    }
                    Ok(())
                }
                ConfigEngine::Custom(_) => Err(anyhow::anyhow!(
                    "Calling rhai functions is only supported with the Rhai config engine"
                )),
            }
        })?;

        Ok(())
    }

    pub fn load_ewwii_plugins(&mut self, plugin_paths: Vec<PathBuf>) -> Result<()> {
        // In case no plugins were passed
        if plugin_paths.is_empty() {
            return Ok(());
        }

        for plugin_path in plugin_paths {
            // SAFETY: This is necessary to load plugins
            unsafe {
                let lib = match libloading::Library::new(&plugin_path) {
                    Ok(l) => l,
                    Err(e) => {
                        log::error!("Failed to load plugin: {}", e);
                        continue;
                    }
                };

                // Create the plugin and receive metadata
                let create_result: Result<
                    libloading::Symbol<unsafe extern "C" fn() -> epapi::PluginInfo>,
                    _,
                > = lib.get(b"ewwii_plugin_create");

                let create = match create_result {
                    Ok(symbol) => symbol,
                    Err(e) => {
                        log::error!("Missing ewwii_plugin_create in {:?}: {}", plugin_path, e);
                        continue;
                    }
                };

                let info = create();

                // crash if plugin id is empty
                if info.id.is_empty() {
                    log::error!("Plugin registration failed: Plugin ID cannot be empty");
                    continue;
                }

                log::debug!("Loading plugin: {} v{}", info.id, info.version);

                let file_stem = plugin_path.file_stem().unwrap().to_str().unwrap();
                let unique_id = format!("{}::{}", file_stem, info.id);

                // Keep the library alive and register it before initialization
                register_active_plugin(lib, unique_id.clone(), info.version.to_string())?;

                {
                    let plugins = plugin::ACTIVE_PLUGINS.read().unwrap();
                    let plugin_entry = plugins.iter().find(|p| p.id == unique_id).unwrap();

                    // Initializing plugins with metadata
                    let init: libloading::Symbol<unsafe extern "C" fn(*const u8, usize)> =
                        plugin_entry.library.get(b"ewwii_plugin_init")?;

                    // Pass the ID back to the plugin
                    let id_bytes = unique_id.as_bytes();
                    init(id_bytes.as_ptr(), id_bytes.len());
                }
            }
        }

        Ok(())
    }
}

fn initialize_window<B: DisplayBackend>(
    window_init: &WindowInitiator,
    monitor: Monitor,
    root_widget: gtk4::Widget,
) -> Result<EwwiiWindow> {
    let monitor_geometry = monitor.geometry();
    let (actual_window_rect, x, y) = match window_init.geometry {
        Some(geometry) => {
            let rect = get_window_rectangle(geometry, monitor_geometry);
            (Some(rect), rect.x(), rect.y())
        }
        _ => (None, 0, 0),
    };
    let window = B::initialize_window(window_init, monitor_geometry, x, y).with_context(|| {
        format!("monitor {} is unavailable", window_init.monitor.clone().unwrap())
    })?;

    window.set_title(Some(&format!("Ewwii - {}", window_init.name)));
    // window.set_position(gtk4::WindowPosition::None);
    // window.set_gravity(gdk::Gravity::Center);

    if let Some(actual_window_rect) = actual_window_rect {
        window.set_size_request(actual_window_rect.width(), actual_window_rect.height());
        window.set_default_size(actual_window_rect.width(), actual_window_rect.height());
    }
    window.set_decorated(false);
    // window.set_skip_taskbar_hint(true);
    // window.set_skip_pager_hint(true);

    // run on_screen_changed to set the visual correctly initially.
    // on_screen_changed(&window, None);
    // window.connect_screen_changed(on_screen_changed);

    window.set_child(Some(&root_widget));

    gtk4::prelude::WidgetExt::realize(&window);

    #[cfg(feature = "x11")]
    if B::IS_X11 {
        if let Some(geometry) = window_init.geometry {
            let (conn, _) = x11rb::rust_connection::RustConnection::connect(None)?;
            let x11_conn = Rc::new(conn);

            let gdk_surface =
                window.surface().context("Couldn't get gdk window from gtk window")?;

            let win_xid = gdk_surface
                .downcast_ref::<gdk4_x11::X11Surface>()
                .context("Failed to get x11 window for gtk window")?
                .xid() as u32;

            use x11rb::protocol::xproto::*;
            x11_conn
                .clone()
                .change_window_attributes(
                    win_xid,
                    &ChangeWindowAttributesAux::new().event_mask(EventMask::STRUCTURE_NOTIFY),
                )
                .unwrap();

            let _ = apply_window_position(x11_conn.clone(), geometry, monitor_geometry, &window);
            if window_init.backend_options.x11.window_type
                != crate::window::backend_window_options::X11WindowType::Normal
            {
                let window_clone = window.clone();
                let conn_clone = x11_conn.clone();

                use x11rb::connection::Connection;

                glib::MainContext::default().spawn_local(async move {
                    loop {
                        if let Ok(event) = conn_clone.poll_for_event() {
                            if let Some(x11rb::protocol::Event::ConfigureNotify(_ev)) = event {
                                let _ = apply_window_position(
                                    conn_clone.clone(),
                                    geometry,
                                    monitor_geometry,
                                    &window_clone,
                                );
                            }
                        }
                        glib::timeout_future(std::time::Duration::from_millis(10)).await;
                    }
                });
            }
        }
        display_backend::set_xprops(&window, monitor, window_init)?;
    }

    window.present();

    Ok(EwwiiWindow {
        name: window_init.name.clone(),
        gtk_window: window,
        delete_event_handler_id: None,
        destroy_event_handler_id: None,
    })
}

/// Apply the provided window-positioning rules to the window.
#[cfg(feature = "x11")]
fn apply_window_position(
    conn: Rc<x11rb::rust_connection::RustConnection>,
    mut window_geometry: WindowGeometry,
    monitor_geometry: gdk::Rectangle,
    window: &Window,
) -> Result<()> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt, Window as XWindow};

    let gdk_surface = window.surface().context("Failed to get gdk surface from gtk window")?;

    if let Some(x11_surface) = gdk_surface.downcast_ref::<gdk4_x11::X11Surface>() {
        window_geometry.size =
            crate::window::window_geometry::Coords::from_pixels(window.default_size());
        let actual_window_rect = get_window_rectangle(window_geometry, monitor_geometry);

        let xid = x11_surface.xid();

        let aux = ConfigureWindowAux::new()
            .x(actual_window_rect.x() as i32)
            .y(actual_window_rect.y() as i32);

        conn.as_ref().configure_window(xid as XWindow, &aux)?;
        conn.as_ref().flush()?;
    }

    Ok(())
}

// fn on_screen_changed(window: &Window, _old_screen: Option<&gdk::Screen>) {
//     let visual = window.screen().and_then(|screen| {
//         screen.rgba_visual().filter(|_| screen.is_composited()).or_else(|| screen.system_visual())
//     });
//     window.set_visual(visual.as_ref());
// }

/// Get the monitor geometry of a given monitor, or the default if none is given
fn get_gdk_monitor(identifier: Option<MonitorIdentifier>) -> Result<Monitor> {
    let display = gdk::Display::default().expect("could not get default display");
    let monitor = match identifier {
        Some(ident) => {
            let mon = get_monitor_from_display(&display, &ident);
            mon.with_context(|| {
                let head = format!("Failed to get monitor {}\nThe available monitors are:", ident);
                let mut body = String::new();
                let monitors = display.monitors();
                for i in 0..monitors.n_items() {
                    if let Some(monitor) = monitors.item(i).and_downcast::<gdk::Monitor>() {
                        if let Some(model) = monitor.model() {
                            body.push_str(format!("\n\t[{}] {}", i, model).as_str());
                        }
                    }
                }
                format!("{}{}", head, body)
            })?
        }
        None => {
            let monitors = display.monitors();
            if monitors.n_items() == 0 {
                anyhow::bail!("No monitors found on the display");
            }

            monitors
                .item(0)
                .and_downcast::<gdk::Monitor>()
                .context("Failed to get the primary monitor from the list of monitors")?
        }
    };
    Ok(monitor)
}

// /// Get the name of monitor plug for given monitor number
// /// workaround gdk not providing this information on wayland in regular calls
// /// gdk_screen_get_monitor_plug_name is deprecated but works fine for that case
// fn get_monitor_plug_name(display: &gdk::Display, monitor_num: i32) -> Option<&str> {
//     unsafe {
//         use glib::translate::ToGlibPtr;
//         let plug_name_pointer = gdk_sys::gdk_screen_get_monitor_plug_name(
//             display.default_screen().to_glib_none().0,
//             monitor_num,
//         );
//         use std::ffi::CStr;
//         CStr::from_ptr(plug_name_pointer).to_str().ok()
//     }
// }

/// Returns the [Monitor][gdk::Monitor] structure corresponding to the identifer.
/// Outside of x11, only [MonitorIdentifier::Numeric] is supported
pub fn get_monitor_from_display(
    display: &gdk::Display,
    identifier: &MonitorIdentifier,
) -> Option<gdk::Monitor> {
    let monitors = display.monitors();
    match identifier {
        MonitorIdentifier::List(list) => {
            for ident in list {
                if let Some(monitor) = get_monitor_from_display(display, ident) {
                    return Some(monitor);
                }
            }
            None
        }
        MonitorIdentifier::Primary => {
            if monitors.n_items() > 0 {
                monitors.item(0).and_downcast::<gdk::Monitor>()
            } else {
                None
            }
        }
        MonitorIdentifier::Numeric(num) => {
            if *num < monitors.n_items() as i32 {
                monitors.item(*num as u32).and_downcast::<gdk::Monitor>()
            } else {
                None
            }
        }
        MonitorIdentifier::Name(name) => {
            for i in 0..monitors.n_items() {
                if let Some(monitor) = monitors.item(i).and_downcast::<gdk::Monitor>() {
                    if let Some(model) = monitor.model() {
                        if model == *name {
                            return Some(monitor);
                        }
                    }
                }
            }
            None
        }
    }
}

pub fn get_window_rectangle(
    geometry: WindowGeometry,
    screen_rect: gdk::Rectangle,
) -> gdk::Rectangle {
    let (offset_x, offset_y) =
        geometry.offset.relative_to(screen_rect.width(), screen_rect.height());
    let (width, height) = geometry.size.relative_to(screen_rect.width(), screen_rect.height());
    let x = screen_rect.x()
        + offset_x
        + geometry.anchor_point.x.alignment_to_coordinate(width, screen_rect.width());
    let y = screen_rect.y()
        + offset_y
        + geometry.anchor_point.y.alignment_to_coordinate(height, screen_rect.height());
    gdk::Rectangle::new(x, y, width, height)
}
