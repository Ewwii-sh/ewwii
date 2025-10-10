use crate::plugin::PluginRequest;
use crate::{
    daemon_response::DaemonResponseSender,
    display_backend::DisplayBackend,
    error_handling_ctx,
    gtk4::prelude::{
        Cast, CastNone, DisplayExt, GtkWindowExt, ListModelExt, MonitorExt, NativeExt, ObjectExt,
        StyleContextExt, WidgetExt,
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
use anyhow::{anyhow, bail};
use gdk::Monitor;
use gtk4::Window;
use gtk4::{gdk, glib};
use itertools::Itertools;
use once_cell::sync::OnceCell;
use rhai::Dynamic;
use rhai_impl::ast::WidgetNode;
use rhai_impl::parser::ParseConfig;
use serde::{de::Error as SerdeError, Deserialize, Deserializer};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    marker::PhantomData,
    rc::Rc,
    sync::Mutex,
};
use tokio::sync::mpsc::UnboundedSender;

static ACTIVE_PLUGIN: OnceCell<libloading::Library> = OnceCell::new();

fn set_active_plugin(lib: libloading::Library) -> Result<()> {
    ACTIVE_PLUGIN.set(lib).map_err(|_| anyhow!("Plugin already set"))
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
    TriggerUpdateUI {
        inject_vars: Option<HashMap<String, String>>,
        should_preserve_state: bool,
        sender: DaemonResponseSender,
    },
    CallRhaiFns {
        calls: Vec<String>,
        sender: DaemonResponseSender,
    },
    EngineOverride {
        config: String,
        print: bool,
        sender: DaemonResponseSender,
    },
    SetPlugin {
        file_path: String,
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

    /// Sender to send [`DaemonCommand`]s
    pub app_evt_send: UnboundedSender<DaemonCommand>,

    /// Senders that will cancel a windows auto-close timer when started with --duration.
    pub window_close_timer_abort_senders: HashMap<String, futures::channel::oneshot::Sender<()>>,

    /// The dynamic gtk widget registery
    pub widget_reg_store: Rc<Mutex<Option<WidgetRegistry>>>,

    // The cached store of poll/listen handlers
    pub pl_handler_store: rhai_impl::updates::ReactiveVarStore,

    pub rt_engine_config: EngineConfValues,
    pub config_parser: Rc<RefCell<ParseConfig>>,

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

                let mut parser_ref = self.config_parser.borrow_mut();
                let config_result = config::read_from_ewwii_paths(&self.paths, &mut *parser_ref);

                drop(parser_ref);

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
            DaemonCommand::PrintDebug(sender) => {
                let output = format!("{:#?}", &self);
                sender.send_success(output)?
            }
            DaemonCommand::ShowState(sender) => {
                let output = format!("{:#?}", &self.pl_handler_store.read().unwrap());
                sender.send_success(output)?
            }
            DaemonCommand::TriggerUpdateUI { inject_vars, should_preserve_state, sender } => {
                match self.trigger_ui_update_with(inject_vars, should_preserve_state) {
                    Ok(_) => sender.send_success(String::new())?,
                    Err(e) => sender.send_failure(e.to_string())?,
                };
            }
            DaemonCommand::CallRhaiFns { calls, sender } => {
                match self.call_rhai_fns(calls) {
                    Ok(_) => sender.send_success(String::new())?,
                    Err(e) => sender.send_failure(e.to_string())?,
                };
            }
            DaemonCommand::EngineOverride { config, print, sender } => {
                match self.set_engine_overrides(config) {
                    Ok(_) => {
                        if print {
                            sender.send_success(format!("{:#?}", self.rt_engine_config))?
                        } else {
                            sender.send_success(String::new())?
                        }
                    }
                    Err(e) => sender.send_failure(e.to_string())?,
                };
            }
            DaemonCommand::SetPlugin { file_path, sender } => {
                match self.set_ewwii_plugin(file_path) {
                    Ok(_) => sender.send_success(String::from("OK"))?,
                    Err(e) => sender.send_failure(e.to_string())?,
                }
            }
        }
        Ok(())
    }

    /// Fully stop ewwii:
    /// close all windows, kill the poll/listen state handler, quit the gtk appliaction and send the exit instruction to the lifecycle manager
    fn stop_application(&mut self) {
        rhai_impl::updates::kill_state_change_handler();
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
        if self.open_windows.is_empty() {
            rhai_impl::updates::kill_state_change_handler();
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

            // Should hold the id and the props of a widget
            // It is critical for supporting dynamic updates
            let new_wdgt_registry =
                WidgetRegistry::new(Some(self.ewwii_config.get_root_node()?.as_ref()));

            // There should be an optimization here.
            // like, we should deextend the map when a window
            // is gone from scope? :/
            if let Ok(mut maybe_registry) = self.widget_reg_store.lock() {
                match maybe_registry.as_mut() {
                    Some(existing_registry) => {
                        existing_registry.widgets.extend(new_wdgt_registry.widgets);

                        existing_registry.stored_widget_node = new_wdgt_registry.stored_widget_node;
                    }
                    None => {
                        *maybe_registry = Some(new_wdgt_registry);
                    }
                }
            } else {
                log::error!("Failed to acquire lock on widget registry");
            }

            let root_widget = {
                let mut maybe_registry = self.widget_reg_store.lock().unwrap();
                let registry = maybe_registry
                    .as_mut()
                    .ok_or_else(|| anyhow::anyhow!("WidgetRegistry is not initialized"))?;
                build_gtk_widget(&WidgetInput::Window(window_def), registry)?
            };

            root_widget.style_context().add_class(window_name);

            let monitor = get_gdk_monitor(initiator.monitor.clone())?;
            let mut ewwii_window = initialize_window::<B>(&initiator, monitor, root_widget)?;

            // Start the poll/listen only once per startup
            // at the start, the open_windows will be empty because
            // it is only later down in open_window(..) that we register
            // the current `instance_id` in the open_windows variable.
            //
            // But since we are doing this hacky method, I wonder
            // if we can move this piece of code somewhere else.
            // I just cant find the perfect place where it can live
            // so I guess that I will just let it stay right here.
            let config_path = self.paths.get_rhai_path();
            let compiled_ast = self.ewwii_config.get_owned_compiled_ast();

            {
                let mut stored_parser = self.config_parser.borrow_mut();
                stored_parser.set_opt_level(get_opt_level_from(
                    self.rt_engine_config.optimization_level.unwrap_or(1),
                ));
            }

            let stored_parser_clone = self.config_parser.clone();
            if self.open_windows.is_empty() {
                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
                let widget_reg_store = self.widget_reg_store.clone();

                // Will automatically mutate pl_handler_store
                rhai_impl::updates::handle_state_changes(
                    self.ewwii_config.get_root_node()?.as_ref(),
                    tx,
                    self.pl_handler_store.clone(),
                );

                let store = self.pl_handler_store.clone();
                let b_interval = self.rt_engine_config.batching_interval;

                glib::MainContext::default().spawn_local(async move {
                    let mut pending_updates = HashSet::new();

                    while let Some(var_name) = rx.recv().await {
                        pending_updates.insert(var_name);

                        match b_interval {
                            Some(i) => glib::timeout_future(Duration::from_millis(i)).await,
                            None => {
                                // short batching interval (1 frame)
                                glib::timeout_future(Duration::from_millis(16)).await
                            }
                        }

                        while let Ok(next_var) = rx.try_recv() {
                            pending_updates.insert(next_var);
                        }

                        let vars = store.read().unwrap().clone();
                        let mut parser_rc = stored_parser_clone.borrow_mut();
                        match generate_new_widgetnode(
                            &vars,
                            &config_path,
                            compiled_ast.as_deref(),
                            &mut *parser_rc,
                        )
                        .await
                        {
                            Ok(new_widget) => {
                                if let Ok(mut maybe_registry) = widget_reg_store.lock() {
                                    if let Some(registry) = maybe_registry.as_mut() {
                                        let _ = registry.update_widget_tree(new_widget);
                                    } else {
                                        log::error!("WidgetRegistry is None inside async loop");
                                    }
                                } else {
                                    log::error!("Failed to acquire lock on widget registry");
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to generate new widgetnode: {:#}", e);
                            }
                        }

                        pending_updates.clear();
                    }

                    log::debug!("Receiver loop exited");
                });
            } else {
                // else, just update the window from the current store.
                let widget_reg_store = self.widget_reg_store.clone();
                let store = self.pl_handler_store.clone();

                glib::MainContext::default().spawn_local(async move {
                    let vars = store.read().unwrap().clone();
                    let mut parser_rc = stored_parser_clone.borrow_mut();
                    match generate_new_widgetnode(
                        &vars,
                        &config_path,
                        compiled_ast.as_deref(),
                        &mut *parser_rc,
                    )
                    .await
                    {
                        Ok(new_widget) => {
                            if let Ok(mut maybe_registry) = widget_reg_store.lock() {
                                if let Some(registry) = maybe_registry.as_mut() {
                                    let _ = registry.update_widget_tree(new_widget);
                                } else {
                                    log::error!("WidgetRegistry is None inside async loop");
                                }
                            } else {
                                log::error!("Failed to acquire lock on widget registry");
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to generate new widgetnode: {:#}", e);
                        }
                    }
                });
            }

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

        self.ewwii_config = config;

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
    }

    /// Load a given CSS string into the gtk css provider, returning a nicely formatted [`DiagError`] when GTK errors out
    pub fn load_css(&mut self, _file_id: usize, css: &str) -> Result<()> {
        self.css_provider.load_from_data(&css);

        Ok(())
    }

    /// Trigger a UI update with the given flags.
    /// Even if there are no flags, the UI will still be updated.
    pub fn trigger_ui_update_with(
        &self,
        inject_vars: Option<HashMap<String, String>>,
        should_preserve_state: bool,
    ) -> Result<()> {
        let compiled_ast = self.ewwii_config.get_owned_compiled_ast();
        let config_path = self.paths.get_rhai_path();

        if !config_path.exists() {
            bail!("The configuration file `{}` does not exist", config_path.display());
        }

        let mut reeval_parser = self.config_parser.borrow_mut();
        let rhai_code = reeval_parser.code_from_file(&config_path)?;

        let mut scope = ParseConfig::initial_poll_listen_scope(&rhai_code)?;

        let all_vars = self.pl_handler_store.read().unwrap().clone();

        for (name, val) in all_vars {
            scope.set_value(name.clone(), Dynamic::from(val.clone()));
        }

        if let Some(vars) = inject_vars {
            for (name, val) in vars {
                scope.set_value(name.clone(), Dynamic::from(val.clone()));

                // Preserving the new state.
                // ---
                // This is esstentially storing the inject variables
                // in the poll/listen variable store (or the `pl_handler_store` in self)
                if should_preserve_state {
                    self.pl_handler_store.write().unwrap().insert(name, val);
                }
            }
        }

        let new_root_widget = reeval_parser.eval_code_with(
            &rhai_code,
            Some(scope),
            compiled_ast.as_deref(),
            config_path.to_str(),
        )?;

        if let Ok(mut maybe_registry) = self.widget_reg_store.lock() {
            if let Some(widget_registry) = maybe_registry.as_mut() {
                let _ = widget_registry.update_widget_tree(new_root_widget);
            } else {
                log::error!("Widget registry is empty");
            }
        } else {
            log::error!("Failed to acquire lock on widget registry");
        }

        Ok(())
    }

    pub fn call_rhai_fns(&self, calls: Vec<String>) -> Result<()> {
        let compiled_ast = self.ewwii_config.get_owned_compiled_ast();
        let config_path = self.paths.get_rhai_path();

        let mut reeval_parser = self.config_parser.borrow_mut();
        let rhai_code = reeval_parser.code_from_file(&config_path)?;

        let mut scope = ParseConfig::initial_poll_listen_scope(&rhai_code)?;

        // unwrap Rc<AST>
        let ast_ref: &rhai::AST =
            compiled_ast.as_ref().ok_or_else(|| anyhow!("AST not compiled yet"))?.as_ref();

        for fn_call in calls {
            reeval_parser.call_rhai_fn(ast_ref, &fn_call, Some(&mut scope))?;
        }

        Ok(())
    }

    pub fn set_engine_overrides(&mut self, config: String) -> Result<()> {
        let parsed_config: EngineConfValues = serde_json::from_str(&config)?;
        self.rt_engine_config = self.rt_engine_config.merge(parsed_config);

        Ok(())
    }

    pub fn set_ewwii_plugin(&mut self, file_path: String) -> Result<()> {
        if ACTIVE_PLUGIN.get().is_some() {
            anyhow::bail!("A plugin is already loaded");
        }

        let lib = unsafe {
            libloading::Library::new(file_path)
                .map_err(|e| anyhow!("Failed to load plugin: {}", e))?
        };

        let (tx, rx): (
            std::sync::mpsc::Sender<PluginRequest>,
            std::sync::mpsc::Receiver<PluginRequest>,
        ) = std::sync::mpsc::channel();

        unsafe {
            // Each plugin exposes: extern "C" fn create_plugin() -> Box<dyn Plugin>
            let constructor: libloading::Symbol<
                unsafe extern "C" fn() -> Box<dyn ewwii_plugin_api::Plugin>,
            > = lib
                .get(b"create_plugin")
                .map_err(|e| anyhow!("Failed to find create_plugin: {}", e))?;

            let plugin = constructor(); // instantiate plugin
            let host = crate::plugin::EwwiiImpl { requestor: tx.clone() };
            plugin.init(&host); // call init immediately

            set_active_plugin(lib)?; // keep library alive
        }

        let cp = self.config_parser.clone();
        let wgs = self.widget_reg_store.clone();

        glib::MainContext::default().spawn_local(async move {
            while let Ok(req) = rx.recv() {
                match req {
                    PluginRequest::RhaiEngineAct(func) => {
                        let mut cp = cp.borrow_mut();
                        cp.action_with_engine(func);
                    }
                    PluginRequest::ListWidgetIds(res_tx) => {
                        let wgs_guard = wgs.lock().unwrap();
                        if let Some(wgs_brw) = wgs_guard.as_ref() {
                            let output: Vec<u64> = wgs_brw.widgets.keys().cloned().collect();

                            if let Err(e) = res_tx.send(output) {
                                log::error!("Failed to send window list to host: {}", e);
                            }
                        }
                    }
                    PluginRequest::WidgetRegistryAct(func) => {
                        let mut wgs_guard = wgs.lock().unwrap();
                        if let Some(ref mut registry) = *wgs_guard {
                            let repr_map: HashMap<u64, &mut gtk4::Widget> = registry
                                .widgets
                                .iter_mut()
                                .map(|(id, entry)| (*id, &mut entry.widget))
                                .collect();

                            func(&mut ewwii_plugin_api::widget_backend::WidgetRegistryRepr {
                                widgets: repr_map,
                            });
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub struct EngineConfValues {
    pub batching_interval: Option<u64>,
    #[serde(default, deserialize_with = "validate_optimization_level")]
    pub optimization_level: Option<u8>,
}

impl EngineConfValues {
    pub fn default() -> Self {
        Self {
            batching_interval: Some(16), // 16 ms
            optimization_level: Some(1), // 1 = simple
        }
    }

    pub fn merge(&self, val: Self) -> Self {
        // could be cleaner
        Self {
            batching_interval: Some(
                val.batching_interval.unwrap_or(self.batching_interval.unwrap_or(16)),
            ),
            optimization_level: Some(
                val.optimization_level.unwrap_or(self.optimization_level.unwrap_or(1)),
            ),
        }
    }
}

fn validate_optimization_level<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<u8>::deserialize(deserializer)?;
    if let Some(value) = opt {
        match value {
            0 | 1 | 2 => Ok(Some(value)),
            _ => Err(D::Error::custom("optimization_level must be 0, 1, or 2")),
        }
    } else {
        Ok(None)
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

    window.show();

    Ok(EwwiiWindow {
        name: window_init.name.clone(),
        gtk_window: window,
        delete_event_handler_id: None,
        destroy_event_handler_id: None,
    })
}

async fn generate_new_widgetnode(
    all_vars: &HashMap<String, String>,
    code_path: &Path,
    compiled_ast: Option<&rhai::AST>,
    parser: &mut ParseConfig,
) -> Result<WidgetNode> {
    let rhai_code = parser.code_from_file(&code_path)?;

    let mut scope = ParseConfig::initial_poll_listen_scope(&rhai_code)?;
    for (name, val) in all_vars {
        scope.set_value(name.clone(), Dynamic::from(val.clone()));
    }

    if !code_path.exists() {
        bail!("The configuration file `{}` does not exist", code_path.display());
    }

    let new_root_widget =
        parser.eval_code_with(&rhai_code, Some(scope), compiled_ast, code_path.to_str())?;

    Ok(new_root_widget)
}

fn get_opt_level_from(n: u8) -> rhai::OptimizationLevel {
    match n {
        0 => rhai::OptimizationLevel::None,
        1 => rhai::OptimizationLevel::Simple,
        2 => rhai::OptimizationLevel::Full,
        _ => rhai::OptimizationLevel::Simple,
    }
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
