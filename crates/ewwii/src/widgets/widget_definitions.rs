#![allow(clippy::option_map_unit_fn)]

use crate::util;
use crate::widgets::build_widget::{build_gtk_widget, WidgetInput};
use anyhow::{anyhow, bail, Result};
use gtk4::gdk::DragAction;
use gtk4::glib::translate::FromGlib;
use gtk4::{self, prelude::*};
use gtk4::{gdk, glib, pango};
use gtk4::{
    DragSource, DropTarget, EventControllerKey, EventControllerLegacy, EventControllerMotion,
    EventControllerScroll, GestureClick,
};
use rhai::Map;
use rhai_impl::ast::{get_id_to_widget_info, hash_props_and_type, WidgetNode};

use super::widget_definitions_helper::*;
use shared_utils::extract_props::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    // cmp::Ordering,
    rc::Rc,
    time::Duration,
};

// custom widgets
// use crate::widgets::{circular_progressbar::CircProg, transform::Transform};

/// Connect a gtk signal handler inside of this macro to ensure that when the same code gets run multiple times,
/// the previously connected singal handler first gets disconnected.
/// Can take an optional condition.
/// If the condition is false, we disconnect the handler without running the connect_expr,
/// thus not connecting a new handler unless the condition is met.
macro_rules! connect_signal_handler {
    // TODO:
    // Fully replace all of the places that uses this with a singular
    // controller. The property inside that controller can be updated
    // through interior mutability similar to have they are working in eventbox.
    ($widget:ident, if $cond:expr, $connect_expr:expr) => {{
        const KEY:&str = std::concat!("signal-handler:", std::line!());
        unsafe {
            let old = $widget.data::<gtk4::glib::SignalHandlerId>(KEY);

            if let Some(old) = old {
                 let a = old.as_ref().as_raw();
                 $widget.disconnect(gtk4::glib::SignalHandlerId::from_glib(a));
            }

            $widget.set_data::<gtk4::glib::SignalHandlerId>(KEY, $connect_expr);
        }
    }};
    ($widget:ident, $connect_expr:expr) => {{
        connect_signal_handler!($widget, if true, $connect_expr)
    }};
}

pub type UpdateFn = Box<dyn Fn(&Map)>;

pub struct WidgetEntry {
    pub widget: gtk4::Widget,
    pub update_fn: UpdateFn,
}

pub struct WidgetRegistry {
    pub widgets: HashMap<u64, WidgetEntry>,
    pub stored_widget_node: Option<WidgetNode>,
}

pub enum PatchGtkWidget<'a> {
    Create(&'a WidgetNode, u64, u64), // node, widget_id, parent_id
    Update(u64, Map),                 // widget_id, props
    Remove(u64),                      // widget_id
}

impl WidgetRegistry {
    pub fn new(wn: Option<&WidgetNode>) -> Self {
        Self { widgets: HashMap::new(), stored_widget_node: wn.cloned() }
    }

    pub fn update_widget_tree(&mut self, new_tree: WidgetNode) -> Result<()> {
        let old_tree = self.stored_widget_node.take();
        let patches = Self::diff_trees(old_tree.as_ref(), &new_tree);

        for patch_req in patches {
            match patch_req {
                PatchGtkWidget::Create(wdgt_node, wdgt_id, parent_id) => {
                    self.create_widget(wdgt_node, wdgt_id, parent_id)
                        .expect("failed to create new gtk widget");
                }
                PatchGtkWidget::Update(widget_id, new_props) => {
                    self.update_props(widget_id, new_props);
                }
                PatchGtkWidget::Remove(widget_id) => self.remove_widget(widget_id),
            }
        }

        self.stored_widget_node = Some(new_tree);
        Ok(())
    }

    pub fn diff_trees<'a>(
        old: Option<&'a WidgetNode>,
        new: &'a WidgetNode,
    ) -> Vec<PatchGtkWidget<'a>> {
        let mut patch = Vec::new();

        let mut old_map = HashMap::new();
        if let Some(old_node) = old {
            let _ = get_id_to_widget_info(old_node, &mut old_map, None);
        }

        let mut new_map = HashMap::new();
        let _ = get_id_to_widget_info(new, &mut new_map, None);

        // Updates and creations
        for (id, new_info) in &new_map {
            match old_map.get(id) {
                Some(old_info) if props_differ(&old_info.props, &new_info.props) => {
                    patch.push(PatchGtkWidget::Update(*id, new_info.props.clone()));
                }
                None => {
                    patch.push(PatchGtkWidget::Create(
                        &new_info.node,
                        *id,
                        new_info.parent_id.expect(&format!(
                            "Parent ID must exist. Widget type: {}",
                            &new_info.widget_type
                        )),
                    ));
                }
                _ => {}
            }
        }

        // Removals
        for (id, _) in &old_map {
            if !new_map.contains_key(id) {
                patch.push(PatchGtkWidget::Remove(*id));
            }
        }

        patch
    }

    pub fn create_widget(
        &mut self,
        widget_node: &WidgetNode,
        widget_id: u64,
        parent_id: u64,
    ) -> Result<()> {
        log::trace!("Creating '{}'", widget_id);
        if let Some(parent) = self.widgets.get(&parent_id) {
            let parent_widget = &parent.widget.clone();

            // find old siblings if the widget already exists
            let (prev_sibling, next_sibling) = if let Some(old_entry) = self.widgets.get(&widget_id)
            {
                (old_entry.widget.prev_sibling(), old_entry.widget.next_sibling())
            } else {
                (None, None)
            };

            // check if widget already exists
            if let Some(old_entry) = self.widgets.remove(&widget_id) {
                // obliterate that widget....
                // how dare it try to create duplication...
                old_entry.widget.unparent();
            }

            // build_gtk_widget also inserts info into widgetentry
            // self is passed for that reason.
            let gtk_widget = build_gtk_widget(&WidgetInput::BorrowedNode(widget_node), self)?;

            // insert into container if it's a Box
            if let Ok(box_container) = parent_widget.clone().dynamic_cast::<gtk4::Box>() {
                box_container.append(&gtk_widget);

                // reordering widgets
                if let Some(prev) = prev_sibling {
                    box_container.reorder_child_after(&gtk_widget, Some(&prev));
                } else if let Some(next) = next_sibling {
                    // move before next sibling: reorder after its prev (None if first)
                    let next_prev = next.prev_sibling();
                    box_container.reorder_child_after(&gtk_widget, next_prev.as_ref());
                } // else: only child, already appended, do nothing
            } else if let Ok(overlay) = parent_widget.clone().dynamic_cast::<gtk4::Overlay>() {
                // TODO: Handle changing main widget
                overlay.add_overlay(&gtk_widget);
            } else {
                // fallback:
                //
                // Assumes that every other container like widget
                // expects only one singular child.
                gtk_widget.set_parent(parent_widget);
            }
        }

        Ok(())
    }

    pub fn update_props(&self, widget_id: u64, new_props: Map) {
        if let Some(entry) = self.widgets.get(&widget_id) {
            (entry.update_fn)(&new_props);
        }
    }

    pub fn remove_widget(&mut self, widget_id: u64) {
        log::trace!("Removing '{}'", widget_id);
        if let Some(entry) = self.widgets.remove(&widget_id) {
            entry.widget.unparent();
        }
    }
}

pub(super) fn build_gtk_box(
    props: &Map,
    children: &Vec<WidgetNode>,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Box> {
    // Parse initial props to create the widget:
    let orientation = props
        .get("orientation")
        .and_then(|v| v.clone().try_cast::<String>())
        .map(|s| parse_orientation(&s))
        .transpose()?
        .unwrap_or(gtk4::Orientation::Horizontal);

    let spacing =
        props.get("spacing").and_then(|v| v.clone().try_cast::<i64>()).unwrap_or(0) as i32;

    let space_evenly = get_bool_prop(&props, "space_evenly", Some(true))?;

    let gtk_widget = gtk4::Box::new(orientation, spacing);
    gtk_widget.set_homogeneous(space_evenly);

    for child in children {
        let child_widget = build_gtk_widget(&WidgetInput::BorrowedNode(child), widget_registry)?;
        gtk_widget.append(&child_widget);
    }

    let gtk_widget_clone = gtk_widget.clone();

    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        if let Some(orientation_str) =
            props.get("orientation").and_then(|v| v.clone().try_cast::<String>())
        {
            if let Ok(orientation) = parse_orientation(&orientation_str) {
                gtk_widget_clone.set_orientation(orientation);
            }
        }

        if let Some(spacing_val) = props.get("spacing").and_then(|v| v.clone().try_cast::<i64>()) {
            gtk_widget_clone.set_spacing(spacing_val as i32);
        }

        if let Ok(space_evenly) = get_bool_prop(props, "space_evenly", None) {
            gtk_widget_clone.set_homogeneous(space_evenly);
        }

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "Box");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { widget: gtk_widget.clone().upcast(), update_fn });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_overlay(
    props: &Map,
    children: &Vec<WidgetNode>,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Overlay> {
    let gtk_widget = gtk4::Overlay::new();

    let count = children.len();

    if count < 1 {
        bail!("overlay must contain at least one element");
    }

    let mut children = children
        .into_iter()
        .map(|child| build_gtk_widget(&WidgetInput::BorrowedNode(child), widget_registry));

    // we have more than one child, we can unwrap
    let first = children.next().unwrap()?;
    gtk_widget.set_child(Some(&first));
    for child in children {
        let child = child?;
        gtk_widget.add_overlay(&child);
    }

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_tooltip(
    props: &Map,
    children: &Vec<WidgetNode>,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Box> {
    let gtk_widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    gtk_widget.set_has_tooltip(true);

    let count = children.len();

    if count < 2 {
        bail!("tooltip must contain exactly 2 children");
    } else if count > 2 {
        bail!("tooltip must contain exactly 2 children, but got more");
    }

    let tooltip_node = children.get(0).cloned().ok_or_else(|| anyhow!("missing tooltip"))?;
    let content_node = children.get(1).cloned().ok_or_else(|| anyhow!("missing content"))?;

    // The visible child immediately
    let content_widget = build_gtk_widget(&WidgetInput::Node(content_node), widget_registry)?;
    gtk_widget.append(&content_widget);

    let tooltip_node = Rc::new(tooltip_node);
    let tooltip_widget = build_gtk_widget(
        &WidgetInput::BorrowedNode(Rc::clone(&tooltip_node).as_ref()),
        widget_registry,
    )
    .expect("Failed to build tooltip widget");

    gtk_widget.connect_query_tooltip(move |_widget, _x, _y, _keyboard_mode, tooltip| {
        tooltip.set_custom(Some(&tooltip_widget));
        true
    });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

struct EventBoxCtrlData {
    // hover controller data
    onhover_cmd: String,
    onhoverlost_cmd: String,
    hover_cursor: String,

    // gesture controller data
    onclick_cmd: String,
    onmiddleclick_cmd: String,
    onrightclick_cmd: String,

    // scroll controoler data
    onscroll_cmd: String,

    // drop controller data
    ondropped_cmd: String,
    dragvalue: String,
    dragtype: DragEntryType,

    // key controller data
    onkeypress_cmd: Option<String>,
    onkeyrelease_cmd: Option<String>,

    // other
    cmd_timeout: Duration,
}

pub(super) fn build_event_box(
    props: &Map,
    children: &Vec<WidgetNode>,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Box> {
    let orientation = props
        .get("orientation")
        .and_then(|v| v.clone().try_cast::<String>())
        .map(|s| parse_orientation(&s))
        .transpose()?
        .unwrap_or(gtk4::Orientation::Horizontal);

    let spacing =
        props.get("spacing").and_then(|v| v.clone().try_cast::<i64>()).unwrap_or(0) as i32;

    let space_evenly = get_bool_prop(&props, "space_evenly", Some(true))?;

    let gtk_widget = gtk4::Box::new(orientation, spacing);
    gtk_widget.set_homogeneous(space_evenly);

    // controllers
    let hover_controller = EventControllerMotion::new();
    let gesture_controller = GestureClick::new();
    let scroll_controller = EventControllerScroll::new(gtk4::EventControllerScrollFlags::BOTH_AXES);
    let legacy_controller = EventControllerLegacy::new();
    let drop_text_target = DropTarget::new(String::static_type(), gdk::DragAction::COPY);
    let drop_uri_target = DropTarget::new(String::static_type(), gdk::DragAction::COPY);
    let key_controller = EventControllerKey::new();

    // properties that can be updated
    let controller_data = Rc::new(RefCell::new(EventBoxCtrlData {
        onhover_cmd: String::new(),
        onhoverlost_cmd: String::new(),
        hover_cursor: String::new(),
        onclick_cmd: String::new(),
        onmiddleclick_cmd: String::new(),
        onrightclick_cmd: String::new(),
        onscroll_cmd: String::new(),
        ondropped_cmd: String::new(),
        dragvalue: String::new(),
        dragtype: DragEntryType::File,
        onkeypress_cmd: None,
        onkeyrelease_cmd: None,
        cmd_timeout: Duration::from_millis(200),
    }));

    // Support :hover selector and run command
    hover_controller.connect_enter(glib::clone!(
        #[weak]
        gtk_widget,
        #[strong]
        controller_data,
        move |_, x, y| {
            let controller = controller_data.borrow();

            gtk_widget.set_state_flags(gtk4::StateFlags::PRELIGHT, false);

            // set cursor
            if let Some(native) = gtk_widget.native() {
                if let Some(surface) = native.surface() {
                    // Create a cursor by name. You can supply a fallback cursor too.
                    if let Some(cursor) =
                        gtk4::gdk::Cursor::from_name(&controller.hover_cursor, None)
                    {
                        surface.set_cursor(Some(&cursor));
                    }
                }
            }

            run_command(controller.cmd_timeout, &controller.onhover_cmd, &[x, y]);
        }
    ));

    hover_controller.connect_leave(glib::clone!(
        #[weak]
        gtk_widget,
        #[strong]
        controller_data,
        move |_| {
            let controller = controller_data.borrow();

            gtk_widget.unset_state_flags(gtk4::StateFlags::PRELIGHT);

            // reset cursor
            if let Some(native) = gtk_widget.native() {
                if let Some(surface) = native.surface() {
                    // Reset to default
                    surface.set_cursor(None);
                }
            }

            run_command(controller.cmd_timeout, &controller.onhoverlost_cmd, &[] as &[&str]);
        }
    ));

    // Support :active selector and run command
    gesture_controller.connect_pressed(glib::clone!(
        #[weak]
        gtk_widget,
        move |_, _, _, _| {
            gtk_widget.set_state_flags(gtk4::StateFlags::ACTIVE, false);
        }
    ));

    // Scroll event handler to run command
    scroll_controller.connect_scroll(glib::clone!(
        #[strong]
        controller_data,
        move |_, dx, dy| {
            let controller = controller_data.borrow();

            if dy != 0.0 {
                run_command(
                    controller.cmd_timeout,
                    &controller.onscroll_cmd,
                    &[if dy < 0.0 { "up" } else { "down" }],
                );
            }

            if dx != 0.0 {
                run_command(
                    controller.cmd_timeout,
                    &controller.onscroll_cmd,
                    &[if dy < 0.0 { "left" } else { "right" }],
                );
            }

            glib::Propagation::Proceed
        }
    ));

    gesture_controller.connect_released(glib::clone!(
        #[weak]
        gtk_widget,
        move |_, _, _, _| {
            gtk_widget.unset_state_flags(gtk4::StateFlags::ACTIVE);
        }
    ));

    legacy_controller.connect_event(glib::clone!(
        #[strong]
        controller_data,
        move |_, event| {
            if event.event_type() == gtk4::gdk::EventType::ButtonPress {
                if let Some(button_event) = event.downcast_ref::<gtk4::gdk::ButtonEvent>() {
                    let button = button_event.button();
                    let controller = controller_data.borrow();
                    match button {
                        1 => run_command(
                            controller.cmd_timeout,
                            &controller.onclick_cmd,
                            &[] as &[&str],
                        ),
                        2 => run_command(
                            controller.cmd_timeout,
                            &controller.onmiddleclick_cmd,
                            &[] as &[&str],
                        ),
                        3 => run_command(
                            controller.cmd_timeout,
                            &controller.onrightclick_cmd,
                            &[] as &[&str],
                        ),
                        _ => {}
                    }
                }
            }
            glib::Propagation::Proceed
        }
    ));

    drop_uri_target.connect_drop(glib::clone!(
        #[strong]
        controller_data,
        move |_, value, _, _| {
            let controller = controller_data.borrow();
            if let Ok(uris) = value.get::<String>() {
                if let Some(first_uri) = uris.split_whitespace().next() {
                    run_command(
                        controller.cmd_timeout,
                        &controller.ondropped_cmd,
                        &[first_uri.to_string(), "file".to_string()],
                    );
                }
            }
            true
        }
    ));

    drop_text_target.connect_drop(glib::clone!(
        #[strong]
        controller_data,
        move |_, value, _, _| {
            let controller = controller_data.borrow();
            if let Ok(text) = value.get::<String>() {
                run_command(
                    controller.cmd_timeout,
                    &controller.ondropped_cmd,
                    &[text.to_string(), "text".to_string()],
                );
            }
            true
        }
    ));

    // drag source settings
    let drag_source = DragSource::new();
    drag_source.set_actions(DragAction::COPY | DragAction::MOVE);

    drag_source.connect_prepare(glib::clone!(
        #[strong]
        controller_data,
        move |_, _, _| {
            let controller = controller_data.borrow();

            match controller.dragtype {
                DragEntryType::File => Some(gdk::ContentProvider::for_value(&glib::Value::from(
                    &[controller.dragvalue.as_str()][..],
                ))),
                DragEntryType::Text => {
                    Some(gdk::ContentProvider::for_value(&glib::Value::from(&controller.dragvalue)))
                }
            }
        }
    ));

    // key controller events
    key_controller.connect_key_pressed(glib::clone!(
        #[strong]
        controller_data,
        move |_, _, code, _| {
            let controller = controller_data.borrow();
            if let Some(cmd) = &controller.onkeypress_cmd {
                run_command(controller.cmd_timeout, cmd, &[code]);
            }
            glib::Propagation::Proceed
        }
    ));

    key_controller.connect_key_released(glib::clone!(
        #[strong]
        controller_data,
        move |_, _, code, _| {
            let controller = controller_data.borrow();
            if let Some(cmd) = &controller.onkeyrelease_cmd {
                run_command(controller.cmd_timeout, cmd, &[code]);
            }
        }
    ));

    gtk_widget.add_controller(gesture_controller);
    gtk_widget.add_controller(hover_controller);
    gtk_widget.add_controller(scroll_controller);
    gtk_widget.add_controller(legacy_controller);
    gtk_widget.add_controller(drop_text_target);
    gtk_widget.add_controller(drop_uri_target);
    gtk_widget.add_controller(drag_source);
    gtk_widget.add_controller(key_controller);

    let apply_props = |props: &Map, controller_data: Rc<RefCell<EventBoxCtrlData>>, gtk_widget: &gtk4::Box| -> Result<()> {
        // timeout - timeout of the command. Default: "200ms"
        controller_data.borrow_mut().cmd_timeout =
            get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

        // onscroll - event to execute when the user scrolls with the mouse over the widget. The placeholder `{}` used in the command will be replaced with either `up` or `down`.
        if let Ok(onscroll) = get_string_prop(&props, "onscroll", None) {
            controller_data.borrow_mut().onscroll_cmd = onscroll;
        }

        // onhover - event to execute when the user hovers over the widget
        if let Ok(onhover) = get_string_prop(&props, "onhover", None) {
            controller_data.borrow_mut().onhover_cmd = onhover;
        }

        // onhoverlost - event to execute when the user loses hover over the widget
        if let Ok(onhoverlost) = get_string_prop(&props, "onhoverlost", None) {
            controller_data.borrow_mut().onhoverlost_cmd = onhoverlost;
        }

        // cursor - Cursor to show while hovering (see [gtk3-cursors](https://docs.gtk.org/gdk3/ctor.Cursor.new_from_name.html) for possible names)
        if let Ok(cursor) = get_string_prop(&props, "cursor", Some("default")) {
            controller_data.borrow_mut().hover_cursor = cursor;
        }

        // ondropped - Command to execute when something is dropped on top of this element. The placeholder `{}` used in the command will be replaced with the uri to the dropped thing.
        if let Ok(ondropped) = get_string_prop(&props, "ondropped", None) {
            controller_data.borrow_mut().ondropped_cmd = ondropped;
        }

        // dragtype - Type of value that should be dragged from this widget. Possible values: $dragtype
        if let Ok(dragtype) = get_string_prop(&props, "drag_type", Some("file")) {
            controller_data.borrow_mut().dragtype = parse_dragtype(&dragtype)?;
        }

        // dragvalue - URI that will be provided when dragging from this widget
        if let Ok(dragvalue) = get_string_prop(&props, "dragvalue", None) {
            controller_data.borrow_mut().dragvalue = dragvalue;
        }

        // onclick - command to run when the widget is clicked
        if let Ok(onclick) = get_string_prop(&props, "onclick", None) {
            controller_data.borrow_mut().onclick_cmd = onclick;
        }
        // onmiddleclick - command to run when the widget is middleclicked
        if let Ok(onmiddleclick) = get_string_prop(&props, "onmiddleclick", None) {
            controller_data.borrow_mut().onmiddleclick_cmd = onmiddleclick;
        }
        // onrightclick - command to run when the widget is rightclicked
        if let Ok(onrightclick) = get_string_prop(&props, "onrightclick", None) {
            controller_data.borrow_mut().onrightclick_cmd = onrightclick;
        }

        // onkeypress - command to to run when a key is pressed
        if let Ok(onkeypress) = get_string_prop(&props, "onkeypress", None) {
            controller_data.borrow_mut().onkeypress_cmd = Some(onkeypress);
        }

        // onkeyrelease - command to run when a key is released
        if let Ok(onkeyrelease) = get_string_prop(&props, "onkeyrelease", None) {
            controller_data.borrow_mut().onkeyrelease_cmd = Some(onkeyrelease);
        }

        if let Some(orientation_str) =
            props.get("orientation").and_then(|v| v.clone().try_cast::<String>())
        {
            if let Ok(orientation) = parse_orientation(&orientation_str) {
                gtk_widget.set_orientation(orientation);
            }
        }

        if let Some(spacing_val) = props.get("spacing").and_then(|v| v.clone().try_cast::<i64>()) {
            gtk_widget.set_spacing(spacing_val as i32);
        }

        if let Ok(space_evenly) = get_bool_prop(props, "space_evenly", None) {
            gtk_widget.set_homogeneous(space_evenly);
        }

        Ok(())
    };

    apply_props(&props, controller_data.clone(), &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, controller_data.clone(), &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let count = children.len();

    if count < 1 {
        bail!("event box must contain exactly one element");
    } else if count > 1 {
        bail!("event box must contain exactly one element, but got more");
    }

    let child = children.get(0).cloned().ok_or_else(|| anyhow!("missing child 0"))?;
    let child_widget = build_gtk_widget(&WidgetInput::Node(child), widget_registry)?;
    gtk_widget.append(&child_widget);
    child_widget.show();

    let id = hash_props_and_type(&props, "EventBox");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_stack(
    props: &Map,
    children: &Vec<WidgetNode>,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Stack> {
    let gtk_widget = gtk4::Stack::new();

    if children.is_empty() {
        return Err(anyhow!("stack must contain at least one element"));
    }

    let children = children
        .into_iter()
        .map(|child| build_gtk_widget(&WidgetInput::BorrowedNode(child), widget_registry));

    for (i, child) in children.enumerate() {
        let child = child?;
        gtk_widget.add_named(&child, Some(&i.to_string()));
        child.show();
    }

    let apply_props = |props: &Map, widget: &gtk4::Stack| -> Result<()> {
        // parsing the properties
        if let Ok(selected) = get_i32_prop(&props, "selected", None) {
            widget.set_visible_child_name(&selected.to_string());
        }

        let transition = get_string_prop(&props, "transition", Some("crossfade"))?;
        widget.set_transition_type(parse_stack_transition(&transition)?);

        // let same_size = get_bool_prop(&props, "same_size", Some(false))?;
        // widget.set_homogeneous(same_size);

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "Stack");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

// pub(super) fn build_transform(
//     props: &Map,
//     widget_registry: &mut WidgetRegistry,
// ) -> Result<Transform> {
//     let widget = Transform::new();

//     let apply_props = |props: &Map, widget: &Transform| -> Result<()> {
//         // rotate - the percentage to rotate
//         if let Ok(rotate) = get_f64_prop(&props, "rotate", None) {
//             widget.set_property("rotate", rotate);
//         }

//         // transform-origin-x - x coordinate of origin of transformation (px or %)
//         if let Ok(transform_origin_x) = get_string_prop(&props, "transform_origin_x", None) {
//             widget.set_property("transform-origin-x", transform_origin_x);
//         }

//         // transform-origin-y - y coordinate of origin of transformation (px or %)
//         if let Ok(transform_origin_y) = get_string_prop(&props, "transform_origin_y", None) {
//             widget.set_property("transform-origin-y", transform_origin_y);
//         }

//         // translate-x - the amount to translate in the x direction (px or %)
//         if let Ok(translate_x) = get_string_prop(&props, "translate_x", None) {
//             widget.set_property("translate-x", translate_x);
//         }

//         // translate-y - the amount to translate in the y direction (px or %)
//         if let Ok(translate_y) = get_string_prop(&props, "translate_y", None) {
//             widget.set_property("translate-y", translate_y);
//         }

//         // scale-x - the amount to scale in the x direction (px or %)
//         if let Ok(scale_x) = get_string_prop(&props, "scale_x", None) {
//             widget.set_property("scale-x", scale_x);
//         }

//         // scale-y - the amount to scale in the y direction (px or %)
//         if let Ok(scale_y) = get_string_prop(&props, "scale_y", None) {
//             widget.set_property("scale-y", scale_y);
//         }

//         Ok(())
//     };

//     apply_props(&props, &widget)?;

//     let widget_clone = widget.clone();
//     let update_fn: UpdateFn = Box::new(move |props: &Map| {
//         let _ = apply_props(props, &widget_clone);

//         // now re-apply generic widget attrs
//         if let Err(err) =
//             resolve_rhai_widget_attrs(&widget_clone.clone().upcast::<gtk4::Widget>(), &props)
//         {
//             eprintln!("Failed to update widget attrs: {:?}", err);
//         }
//     });

//     let id = hash_props_and_type(&props, "Transform");

//     widget_registry.widgets.insert(id, WidgetEntry { update_fn, widget: widget.clone().upcast() });

//     resolve_rhai_widget_attrs(&widget.clone().upcast::<gtk4::Widget>(), &props)?;

//     Ok(widget)
// }

// pub(super) fn build_circular_progress_bar(
//     props: &Map,
//     widget_registry: &mut WidgetRegistry,
// ) -> Result<CircProg> {
//     let widget = CircProg::new();

//     let apply_props = |props: &Map, widget: &CircProg| -> Result<()> {
//         if let Ok(value) = get_f64_prop(&props, "value", None) {
//             widget.set_property("value", value.clamp(0.0, 100.0));
//         }

//         if let Ok(start_at) = get_f64_prop(&props, "start_at", None) {
//             widget.set_property("start-at", start_at.clamp(0.0, 100.0));
//         }

//         if let Ok(thickness) = get_f64_prop(&props, "thickness", None) {
//             widget.set_property("thickness", thickness);
//         }

//         if let Ok(clockwise) = get_f64_prop(&props, "clockwise", None) {
//             widget.set_property("clockwise", clockwise);
//         }

//         Ok(())
//     };

//     apply_props(&props, &widget)?;

//     let widget_clone = widget.clone();
//     let update_fn: UpdateFn = Box::new(move |props: &Map| {
//         let _ = apply_props(props, &widget_clone);

//         // now re-apply generic widget attrs
//         if let Err(err) =
//             resolve_rhai_widget_attrs(&widget_clone.clone().upcast::<gtk4::Widget>(), &props)
//         {
//             eprintln!("Failed to update widget attrs: {:?}", err);
//         }
//     });

//     let id = hash_props_and_type(&props, "CircularProgressBar");

//     widget_registry.widgets.insert(id, WidgetEntry { update_fn, widget: widget.clone().upcast() });

//     resolve_rhai_widget_attrs(&widget.clone().upcast::<gtk4::Widget>(), &props)?;

//     Ok(widget)
// }

// pub(super) fn build_graph(
//     props: &Map,
//     widget_registry: &mut WidgetRegistry,
// ) -> Result<super::graph::Graph> {
//     let widget = super::graph::Graph::new();

//     let apply_props = |props: &Map, widget: &super::graph::Graph| -> Result<()> {
//         if let Ok(value) = get_f64_prop(&props, "value", None) {
//             if value.is_nan() || value.is_infinite() {
//                 return Err(anyhow!("Graph's value should never be NaN or infinite"));
//             }
//             widget.set_property("value", value);
//         }

//         if let Ok(thickness) = get_f64_prop(&props, "thickness", None) {
//             widget.set_property("thickness", thickness);
//         }

//         if let Ok(time_range) = get_duration_prop(&props, "time_range", None) {
//             widget.set_property("time-range", time_range.as_millis() as u64);
//         }

//         let min = get_f64_prop(&props, "min", Some(0.0)).ok();
//         let max = get_f64_prop(&props, "max", Some(100.0)).ok();

//         if let (Some(mi), Some(ma)) = (min, max) {
//             if mi > ma {
//                 return Err(anyhow!("Graph's min ({mi}) should never be higher than max ({ma})"));
//             }
//         }

//         if let Some(mi) = min {
//             widget.set_property("min", mi);
//         }

//         if let Some(ma) = max {
//             widget.set_property("max", ma);
//         }

//         if let Ok(dynamic) = get_bool_prop(&props, "dynamic", None) {
//             widget.set_property("dynamic", dynamic);
//         }

//         if let Ok(line_style) = get_string_prop(&props, "line_style", None) {
//             widget.set_property("line-style", line_style);
//         }

//         // flip-x - whether the x axis should go from high to low
//         if let Ok(flip_x) = get_bool_prop(&props, "flip_x", None) {
//             widget.set_property("flip-x", flip_x);
//         }

//         // flip-y - whether the y axis should go from high to low
//         if let Ok(flip_y) = get_bool_prop(&props, "flip_y", None) {
//             widget.set_property("flip-y", flip_y);
//         }

//         // vertical - if set to true, the x and y axes will be exchanged
//         if let Ok(vertical) = get_bool_prop(&props, "vertical", None) {
//             widget.set_property("vertical", vertical);
//         }

//         Ok(())
//     };

//     apply_props(&props, &widget)?;

//     let widget_clone = widget.clone();
//     let update_fn: UpdateFn = Box::new(move |props: &Map| {
//         let _ = apply_props(props, &widget_clone);

//         // now re-apply generic widget attrs
//         if let Err(err) =
//             resolve_rhai_widget_attrs(&widget_clone.clone().upcast::<gtk4::Widget>(), &props)
//         {
//             eprintln!("Failed to update widget attrs: {:?}", err);
//         }
//     });

//     let id = hash_props_and_type(&props, "Graph");

//     widget_registry.widgets.insert(id, WidgetEntry { update_fn, widget: widget.clone().upcast() });

//     resolve_rhai_widget_attrs(&widget.clone().upcast::<gtk4::Widget>(), &props)?;

//     Ok(widget)
// }

pub(super) fn build_gtk_progress(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ProgressBar> {
    let gtk_widget = gtk4::ProgressBar::new();

    let apply_props = |props: &Map, widget: &gtk4::ProgressBar| -> Result<()> {
        let orientation = props
            .get("orientation")
            .and_then(|v| v.clone().try_cast::<String>())
            .map(|s| parse_orientation(&s))
            .transpose()?
            .unwrap_or(gtk4::Orientation::Horizontal);

        widget.set_orientation(orientation);

        if let Ok(flipped) = get_bool_prop(&props, "flipped", Some(false)) {
            widget.set_inverted(flipped)
        }

        if let Ok(bar_value) = get_f64_prop(&props, "value", None) {
            widget.set_fraction(bar_value / 100f64)
        }

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "Progress");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_image(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Picture> {
    let gtk_widget = gtk4::Picture::new();

    let apply_props = |props: &Map, widget: &gtk4::Picture| -> Result<()> {
        let path = get_string_prop(&props, "path", None)?;
        let image_width = get_i32_prop(&props, "image_width", Some(-1))?;
        let image_height = get_i32_prop(&props, "image_height", Some(-1))?;
        let preserve_aspect_ratio = get_bool_prop(&props, "preserve_aspect_ratio", Some(true))?;
        let fill_svg = get_string_prop(&props, "fill_svg", Some(""))?;

        if !path.ends_with(".svg") && !fill_svg.is_empty() {
            log::warn!("Fill attribute ignored, file is not an svg image");
        }

        if path.ends_with(".gif") {
            let pixbuf_animation =
                gtk4::gdk_pixbuf::PixbufAnimation::from_file(std::path::PathBuf::from(path))?;
            let iter = pixbuf_animation.iter(None);

            let frame_pixbuf = iter.pixbuf();
            widget.set_pixbuf(Some(&frame_pixbuf));

            let widget_clone = widget.clone();

            if let Some(delay) = iter.delay_time() {
                glib::timeout_add_local(delay, move || {
                    let now = std::time::SystemTime::now();

                    if iter.advance(now) {
                        let frame_pixbuf = iter.pixbuf();
                        widget_clone.set_pixbuf(Some(&frame_pixbuf));
                    }

                    glib::ControlFlow::Continue
                });
            }
        } else {
            let pixbuf;
            // populate the pixel buffer
            if path.ends_with(".svg") && !fill_svg.is_empty() {
                let svg_data = std::fs::read_to_string(std::path::PathBuf::from(path.clone()))?;
                // The fastest way to add/change fill color
                let svg_data = if svg_data.contains("fill=") {
                    let reg = regex::Regex::new(r#"fill="[^"]*""#)?;
                    reg.replace(&svg_data, &format!("fill=\"{}\"", fill_svg))
                } else {
                    let reg = regex::Regex::new(r"<svg")?;
                    reg.replace(&svg_data, &format!("<svg fill=\"{}\"", fill_svg))
                };
                let stream = gtk4::gio::MemoryInputStream::from_bytes(&gtk4::glib::Bytes::from(
                    svg_data.as_bytes(),
                ));
                pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_stream_at_scale(
                    &stream,
                    image_width,
                    image_height,
                    preserve_aspect_ratio,
                    None::<&gtk4::gio::Cancellable>,
                )?;
                stream.close(None::<&gtk4::gio::Cancellable>)?;
            } else {
                pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_file_at_scale(
                    std::path::PathBuf::from(path),
                    image_width,
                    image_height,
                    preserve_aspect_ratio,
                )?;
            }
            widget.set_pixbuf(Some(&pixbuf));
        }

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "Image");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_icon(props: &Map, widget_registry: &mut WidgetRegistry) -> Result<gtk4::Image> {
    let gtk_widget = gtk4::Image::new();

    let apply_props = |props: &Map, widget: &gtk4::Image| -> Result<()> {
        let path = get_string_prop(&props, "path", None)?;
        let image_width = get_i32_prop(&props, "image_width", Some(-1))?;
        let image_height = get_i32_prop(&props, "image_height", Some(-1))?;
        let preserve_aspect_ratio = get_bool_prop(&props, "preserve_aspect_ratio", Some(true))?;
        let fill_svg = get_string_prop(&props, "fill_svg", Some(""))?;

        if !path.ends_with(".svg") && !fill_svg.is_empty() {
            log::warn!("Fill attribute ignored, file is not an svg image");
        }

        if path.ends_with(".gif") {
            let pixbuf_animation =
                gtk4::gdk_pixbuf::PixbufAnimation::from_file(std::path::PathBuf::from(path))?;
            let iter = pixbuf_animation.iter(None);

            let frame_pixbuf = iter.pixbuf();
            widget.set_from_pixbuf(Some(&frame_pixbuf));

            let widget_clone = widget.clone();

            if let Some(delay) = iter.delay_time() {
                glib::timeout_add_local(delay, move || {
                    let frame_pixbuf = iter.pixbuf();
                    widget_clone.set_from_pixbuf(Some(&frame_pixbuf));

                    glib::ControlFlow::Continue
                });
            }
        } else {
            let pixbuf;
            // populate the pixel buffer
            if path.ends_with(".svg") && !fill_svg.is_empty() {
                let svg_data = std::fs::read_to_string(std::path::PathBuf::from(path.clone()))?;
                // The fastest way to add/change fill color
                let svg_data = if svg_data.contains("fill=") {
                    let reg = regex::Regex::new(r#"fill="[^"]*""#)?;
                    reg.replace(&svg_data, &format!("fill=\"{}\"", fill_svg))
                } else {
                    let reg = regex::Regex::new(r"<svg")?;
                    reg.replace(&svg_data, &format!("<svg fill=\"{}\"", fill_svg))
                };
                let stream = gtk4::gio::MemoryInputStream::from_bytes(&gtk4::glib::Bytes::from(
                    svg_data.as_bytes(),
                ));
                pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_stream_at_scale(
                    &stream,
                    image_width,
                    image_height,
                    preserve_aspect_ratio,
                    None::<&gtk4::gio::Cancellable>,
                )?;
                stream.close(None::<&gtk4::gio::Cancellable>)?;
            } else {
                pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_file_at_scale(
                    std::path::PathBuf::from(path),
                    image_width,
                    image_height,
                    preserve_aspect_ratio,
                )?;
            }
            widget.set_from_pixbuf(Some(&pixbuf));
        }

        if let Ok(icon_name) = get_string_prop(&props, "icon", None) {
            widget.set_icon_name(Some(&icon_name));
        }

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "Image");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

#[derive(Clone)]
struct GtkButtonCtrlData {
    // button press
    onclick_cmd: String,
    onmiddleclick_cmd: String,
    onrightclick_cmd: String,

    // command timeout
    cmd_timeout: Duration,
}

pub(super) fn build_gtk_button(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Button> {
    let gtk_widget = gtk4::Button::new();

    let controller_data = Rc::new(RefCell::new(GtkButtonCtrlData {
        onclick_cmd: String::new(),
        onmiddleclick_cmd: String::new(),
        onrightclick_cmd: String::new(),
        cmd_timeout: Duration::from_millis(200),
    }));

    let key_controller = EventControllerKey::new();
    let legacy_controller = EventControllerLegacy::new();

    gtk_widget.connect_clicked(glib::clone!(
        #[weak]
        gtk_widget,
        move |_| {
            gtk_widget.emit_activate();
        }
    ));

    legacy_controller.connect_event(glib::clone!(
        #[strong]
        controller_data,
        move |_, event| {
            if event.event_type() == gtk4::gdk::EventType::ButtonPress {
                if let Some(button_event) = event.downcast_ref::<gtk4::gdk::ButtonEvent>() {
                    let button = button_event.button();
                    let controller = controller_data.borrow();
                    match button {
                        1 => run_command(
                            controller.cmd_timeout,
                            &controller.onclick_cmd,
                            &[] as &[&str],
                        ),
                        2 => run_command(
                            controller.cmd_timeout,
                            &controller.onmiddleclick_cmd,
                            &[] as &[&str],
                        ),
                        3 => run_command(
                            controller.cmd_timeout,
                            &controller.onrightclick_cmd,
                            &[] as &[&str],
                        ),
                        _ => {}
                    }
                }
            }
            gtk4::glib::Propagation::Proceed
        }
    ));

    key_controller.connect_key_released(glib::clone!(
        #[strong]
        controller_data,
        move |_, _, code, _| {
            let controller = controller_data.borrow();
            match code {
                // return
                36 => run_command(controller.cmd_timeout, &controller.onclick_cmd, &[] as &[&str]),
                // space
                65 => run_command(controller.cmd_timeout, &controller.onclick_cmd, &[] as &[&str]),
                _ => {}
            }
        }
    ));

    gtk_widget.add_controller(key_controller);
    gtk_widget.add_controller(legacy_controller);

    let apply_props = |props: &Map,
                       widget: &gtk4::Button,
                       controller_data: Rc<RefCell<GtkButtonCtrlData>>|
     -> Result<()> {
        controller_data.borrow_mut().cmd_timeout =
            get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

        if let Ok(onclick) = get_string_prop(&props, "onclick", None) {
            controller_data.borrow_mut().onclick_cmd = onclick;
        }
        if let Ok(onmiddleclick) = get_string_prop(&props, "onmiddleclick", None) {
            controller_data.borrow_mut().onmiddleclick_cmd = onmiddleclick;
        }
        if let Ok(onrightclick) = get_string_prop(&props, "onrightclick", None) {
            controller_data.borrow_mut().onrightclick_cmd = onrightclick;
        }

        if let Ok(button_label) = get_string_prop(&props, "label", None) {
            widget.set_label(&button_label);
        }

        Ok(())
    };

    apply_props(&props, &gtk_widget, controller_data.clone())?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone, controller_data.clone());

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "Button");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_label(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Label> {
    let gtk_widget = gtk4::Label::new(None);

    let apply_props = |props: &Map, widget: &gtk4::Label| -> Result<()> {
        let truncate = get_bool_prop(&props, "truncate", Some(false))?;
        let limit_width = get_i32_prop(&props, "limit_width", Some(i32::MAX))?;
        let truncate_left = get_bool_prop(&props, "truncate_left", Some(false))?;
        let show_truncated = get_bool_prop(&props, "show_truncated", Some(true))?;
        let unindent = get_bool_prop(&props, "unindent", Some(true))?;

        let has_text = props.get("text").is_some();
        let has_markup = props.get("markup").is_some();

        if has_text && has_markup {
            bail!("Cannot set both 'text' and 'markup' for a label");
        } else if has_text {
            let text = get_string_prop(&props, "text", None)?;
            let t = if show_truncated {
                if limit_width == i32::MAX {
                    widget.set_max_width_chars(-1);
                } else {
                    widget.set_max_width_chars(limit_width);
                }
                apply_ellipsize_settings(
                    &widget,
                    truncate,
                    limit_width,
                    truncate_left,
                    show_truncated,
                );
                text
            } else {
                widget.set_ellipsize(pango::EllipsizeMode::None);

                let limit_width = limit_width as usize;
                let char_count = text.chars().count();
                if char_count > limit_width {
                    if truncate_left {
                        text.chars().skip(char_count - limit_width).collect()
                    } else {
                        text.chars().take(limit_width).collect()
                    }
                } else {
                    text
                }
            };

            let unescaped =
                unescape::unescape(&t).ok_or_else(|| anyhow!("Failed to unescape..."))?;
            let final_text = if unindent { util::unindent(&unescaped) } else { unescaped };
            widget.set_text(&final_text);
        } else if has_markup {
            let markup = get_string_prop(&props, "markup", None)?;
            apply_ellipsize_settings(&widget, truncate, limit_width, truncate_left, show_truncated);
            widget.set_markup(&markup);
        } else {
            bail!("Either 'text' or 'markup' must be set");
        }

        if let Ok(wrap) = get_bool_prop(&props, "wrap", Some(false)) {
            widget.set_wrap(wrap);
        }

        // if let Ok(angle) = get_f64_prop(&props, "angle", Some(0.0)) {
        //     widget.set_angle(angle);
        // }

        let gravity = get_string_prop(&props, "gravity", Some("south"))?;
        widget.pango_context().set_base_gravity(parse_gravity(&gravity)?);

        if let Ok(xalign) = get_f64_prop(&props, "xalign", Some(0.5)) {
            widget.set_xalign(xalign as f32);
        }

        if let Ok(yalign) = get_f64_prop(&props, "yalign", Some(0.5)) {
            widget.set_yalign(yalign as f32);
        }

        let justify = get_string_prop(&props, "justify", Some("left"))?;
        widget.set_justify(parse_justification(&justify)?);

        let wrap_mode = get_string_prop(&props, "wrap_mode", Some("word"))?;
        widget.set_wrap_mode(parse_wrap_mode(&wrap_mode)?);

        if let Ok(lines) = get_i32_prop(&props, "lines", Some(-1)) {
            widget.set_lines(lines);
        }

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "Label");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_input(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Entry> {
    let gtk_widget = gtk4::Entry::new();

    let apply_props = |props: &Map, widget: &gtk4::Entry| -> Result<()> {
        if let Ok(value) = get_string_prop(&props, "value", None) {
            widget.set_text(&value);
        }

        let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

        if let Ok(onchange) = get_string_prop(&props, "onchange", None) {
            connect_signal_handler!(
                widget,
                widget.connect_changed(move |widget| {
                    run_command(timeout, &onchange, &[widget.text().to_string()]);
                })
            );
        }

        if let Ok(onaccept) = get_string_prop(&props, "onaccept", None) {
            connect_signal_handler!(
                widget,
                widget.connect_activate(move |widget| {
                    run_command(timeout, &onaccept, &[widget.text().to_string()]);
                })
            );
        }

        let password: bool = get_bool_prop(&props, "password", Some(false))?;
        widget.set_visibility(!password);

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "Input");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_calendar(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Calendar> {
    let gtk_widget = gtk4::Calendar::new();

    let apply_props = |props: &Map, widget: &gtk4::Calendar| -> Result<()> {
        // day - the selected day
        if let Ok(day) = get_f64_prop(&props, "day", None) {
            if !(1f64..=31f64).contains(&day) {
                log::warn!("Calendar day is not a number between 1 and 31");
            } else {
                widget.set_day(day as i32)
            }
        }

        // month - the selected month
        if let Ok(month) = get_f64_prop(&props, "month", None) {
            if !(1f64..=12f64).contains(&month) {
                log::warn!("Calendar month is not a number between 1 and 12");
            } else {
                widget.set_month(month as i32 - 1)
            }
        }

        // year - the selected year
        if let Ok(year) = get_f64_prop(&props, "year", None) {
            widget.set_year(year as i32)
        }

        // // show-details - show details
        // if let Ok(show_details) = get_bool_prop(&props, "show_details", None) {
        //     widget.set_show_details(show_details)
        // }

        // show-heading - show heading line
        if let Ok(show_heading) = get_bool_prop(&props, "show_heading", None) {
            widget.set_show_heading(show_heading)
        }

        // show-day-names - show names of days
        if let Ok(show_day_names) = get_bool_prop(&props, "show_day_names", None) {
            widget.set_show_day_names(show_day_names)
        }

        // show-week-numbers - show week numbers
        if let Ok(show_week_numbers) = get_bool_prop(&props, "show_week_numbers", None) {
            widget.set_show_week_numbers(show_week_numbers)
        }

        // timeout - timeout of the command. Default: "200ms"
        let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

        // onclick - command to run when the user selects a date. The `{0}` placeholder will be replaced by the selected day, `{1}` will be replaced by the month, and `{2}` by the year.
        if let Ok(onclick) = get_string_prop(&props, "onclick", None) {
            connect_signal_handler!(
                widget,
                widget.connect_day_selected(move |w| {
                    run_command(timeout, &onclick, &[w.day(), w.month(), w.year()])
                })
            );
        }
        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "Calendar");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_combo_box_text(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ComboBoxText> {
    let gtk_widget = gtk4::ComboBoxText::new();

    let apply_props = |props: &Map, widget: &gtk4::ComboBoxText| -> Result<()> {
        if let Ok(items) = get_vec_string_prop(&props, "items", None) {
            widget.remove_all();
            for i in items {
                widget.append_text(&i);
            }
        }

        let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;
        let onchange = get_string_prop(&props, "onchange", Some(""))?;

        connect_signal_handler!(
            widget,
            widget.connect_changed(move |widget| {
                run_command(
                    timeout,
                    &onchange,
                    &[widget.active_text().unwrap_or_else(|| "".into())],
                );
            })
        );

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "ComboBoxText");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_expander(
    props: &Map,
    children: &Vec<WidgetNode>,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Expander> {
    let gtk_widget = gtk4::Expander::new(None);

    let count = children.len();

    if count < 1 {
        bail!("expander must contain exactly one element");
    } else if count > 1 {
        bail!("expander must contain exactly one element, but got more");
    }

    let child = children.get(0).cloned().ok_or_else(|| anyhow!("missing child 0"))?;
    let child_widget = build_gtk_widget(&WidgetInput::Node(child), widget_registry)?;
    gtk_widget.set_child(Some(&child_widget));
    child_widget.show();

    let apply_props = |props: &Map, widget: &gtk4::Expander| -> Result<()> {
        if let Ok(name) = get_string_prop(&props, "name", None) {
            widget.set_label(Some(&name));
        }

        if let Ok(expanded) = get_bool_prop(&props, "expanded", None) {
            widget.set_expanded(expanded);
        }

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "Expander");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_revealer(
    props: &Map,
    children: &Vec<WidgetNode>,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Revealer> {
    let gtk_widget = gtk4::Revealer::new();

    let apply_props = |props: &Map, widget: &gtk4::Revealer| -> Result<()> {
        let transition = get_string_prop(&props, "transition", Some("crossfade"))?;
        widget.set_transition_type(parse_revealer_transition(&transition)?);

        if let Ok(reveal) = get_bool_prop(&props, "reveal", None) {
            widget.set_reveal_child(reveal);
        }

        let duration = get_duration_prop(&props, "duration", Some(Duration::from_millis(500)))?;

        widget.set_transition_duration(duration.as_millis() as u32);

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    match children.len() {
        0 => { /* maybe warn? */ }
        1 => {
            let child_widget =
                build_gtk_widget(&WidgetInput::Node(children[0].clone()), widget_registry)?;
            gtk_widget.set_child(Some(&child_widget));
        }
        n => {
            return Err(anyhow!("A revealer must only have a maximum of 1 child but got: {}", n));
        }
    }

    let id = hash_props_and_type(&props, "Revealer");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_checkbox(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::CheckButton> {
    let gtk_widget = gtk4::CheckButton::new();

    let apply_props = |props: &Map, widget: &gtk4::CheckButton| -> Result<()> {
        let checked = get_bool_prop(&props, "checked", Some(false))?;
        widget.set_active(checked);

        let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;
        let onchecked = get_string_prop(&props, "onchecked", Some(""))?;
        let onunchecked = get_string_prop(&props, "onchecked", Some(""))?;

        connect_signal_handler!(
            widget,
            widget.connect_toggled(move |widget| {
                run_command(
                    timeout,
                    if widget.is_active() { &onchecked } else { &onunchecked },
                    &[] as &[&str],
                );
            })
        );

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "Checkbox");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_color_button(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ColorButton> {
    let gtk_widget = gtk4::ColorButton::builder().build();

    let apply_props = |props: &Map, widget: &gtk4::ColorButton| -> Result<()> {
        // use-alpha - bool to wether or not use alpha
        if let Ok(use_alpha) = get_bool_prop(&props, "use_alpha", None) {
            widget.set_use_alpha(use_alpha);
        }

        // timeout - timeout of the command. Default: "200ms"
        let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

        // onchange - runs the code when the color was selected
        if let Ok(onchange) = get_string_prop(&props, "onchange", None) {
            connect_signal_handler!(
                widget,
                widget.connect_color_set(move |widget| {
                    run_command(timeout, &onchange, &[widget.rgba()]);
                })
            );
        }

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "ColorButton");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_color_chooser(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ColorChooserWidget> {
    let gtk_widget = gtk4::ColorChooserWidget::new();

    let apply_props = |props: &Map, widget: &gtk4::ColorChooserWidget| -> Result<()> {
        // use-alpha - bool to wether or not use alpha
        if let Ok(use_alpha) = get_bool_prop(&props, "use_alpha", None) {
            widget.set_use_alpha(use_alpha);
        }

        // timeout - timeout of the command. Default: "200ms"
        let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

        // onchange - runs the code when the color was selected
        if let Ok(onchange) = get_string_prop(&props, "onchange", None) {
            connect_signal_handler!(
                widget,
                widget.connect_color_activated(move |_a, color| {
                    run_command(timeout, &onchange, &[*color]);
                })
            );
        }

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "ColorChooser");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) struct RangeCtrlData {
    onchange_cmd: String,
    cmd_timeout: Duration,
    is_being_dragged: bool,
    last_set_value: Option<f64>,
}

pub(super) fn build_gtk_scale(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Scale> {
    let gtk_widget = gtk4::Scale::new(
        gtk4::Orientation::Horizontal,
        Some(&gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 1.0)),
    );

    // only allow changing the value via the value property if the user isn't currently dragging
    let scale_dat = Rc::new(RefCell::new(RangeCtrlData {
        onchange_cmd: String::new(),
        cmd_timeout: Duration::from_millis(16),
        is_being_dragged: false,
        last_set_value: None,
    }));
    let legacy_controller = EventControllerLegacy::new();

    // there might be better implementation but
    // this seems to be the one that works well.
    legacy_controller.connect_event(glib::clone!(
        #[strong]
        scale_dat,
        move |ctrl, event| {
            match event.event_type() {
                gtk4::gdk::EventType::ButtonPress => {
                    scale_dat.borrow_mut().is_being_dragged = true;
                }
                gtk4::gdk::EventType::ButtonRelease => {
                    let mut scale_dat_mut = scale_dat.borrow_mut();
                    scale_dat_mut.is_being_dragged = false;

                    if let Some(widget) = ctrl.widget() {
                        let value = widget.downcast_ref::<gtk4::Scale>().unwrap().value();

                        if scale_dat_mut.last_set_value.take() != Some(value) {
                            let cmd_timeout = scale_dat_mut.cmd_timeout;
                            let onchange_cmd = scale_dat_mut.onchange_cmd.clone();
                            drop(scale_dat_mut);

                            run_command(cmd_timeout, &onchange_cmd, &[value]);
                        }
                    }
                }
                _ => {}
            }
            glib::Propagation::Proceed
        }
    ));

    gtk_widget.add_controller(legacy_controller);

    // Reusable closure for applying props
    let apply_props =
        |props: &Map, widget: &gtk4::Scale, scale_dat: Rc<RefCell<RangeCtrlData>>| -> Result<()> {
            widget.set_inverted(get_bool_prop(props, "flipped", Some(false))?);

            if let Ok(marks) = get_string_prop(props, "marks", None) {
                widget.clear_marks();
                for mark in marks.split(',') {
                    widget.add_mark(mark.trim().parse()?, gtk4::PositionType::Bottom, None);
                }
            }

            widget.set_draw_value(get_bool_prop(props, "draw_value", Some(false))?);

            if let Ok(value_pos) = get_string_prop(props, "value_pos", None) {
                widget.set_value_pos(parse_position_type(&value_pos)?);
            }

            widget.set_round_digits(get_i32_prop(props, "round_digits", Some(0))?);

            resolve_range_attrs(props, widget.upcast_ref::<gtk4::Range>(), scale_dat)?;
            Ok(())
        };

    apply_props(&props, &gtk_widget, scale_dat.clone())?;

    let gtk_widget_clone = gtk_widget.clone();
    let scale_dat_clone = scale_dat.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone, scale_dat_clone.clone());

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "Slider");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_scrolledwindow(
    props: &Map,
    children: &Vec<WidgetNode>,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ScrolledWindow> {
    // I don't have single idea of what those two generics are supposed to be, but this works.
    let gtk_widget = gtk4::ScrolledWindow::new();

    let apply_props = |props: &Map, widget: &gtk4::ScrolledWindow| -> Result<()> {
        let hscroll = get_bool_prop(&props, "hscroll", Some(true))?;
        let vscroll = get_bool_prop(&props, "vscroll", Some(true))?;

        widget.set_policy(
            if hscroll { gtk4::PolicyType::Automatic } else { gtk4::PolicyType::Never },
            if vscroll { gtk4::PolicyType::Automatic } else { gtk4::PolicyType::Never },
        );

        if let Ok(natural_height_bool) = get_bool_prop(&props, "propagate_natural_height", None) {
            widget.set_propagate_natural_height(natural_height_bool);
        }

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    let gtk_widget_clone = gtk_widget.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let count = children.len();

    if count < 1 {
        bail!("scrolled window must contain exactly one element");
    } else if count > 1 {
        bail!("scrolled window contain exactly one element, but got more");
    }

    let child = children.get(0).cloned().ok_or_else(|| anyhow!("missing child 0"))?;
    let child_widget = build_gtk_widget(&WidgetInput::Node(child), widget_registry)?;
    gtk_widget.set_child(Some(&child_widget));
    child_widget.show();

    let id = hash_props_and_type(&props, "ScrolledWindow");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

// commented out because i dont think its needed...
// /// Deprecated attributes from top of widget hierarchy
// static DEPRECATED_ATTRS: Lazy<HashSet<&str>> =
//     Lazy::new(|| ["timeout", "onscroll", "onhover", "cursor"].iter().cloned().collect());

/// Code that applies css/scss to widgets.
pub(super) fn resolve_rhai_widget_attrs(gtk_widget: &gtk4::Widget, props: &Map) -> Result<()> {
    // // checking deprecated keys
    // // see eww issue #251 (https://github.com/elkowar/eww/issues/251)
    // for deprecated in DEPRECATED_ATTRS.iter() {
    //     if props.contains_key(*deprecated) {
    //         eprintln!("Warning: attribute `{}` is deprecated and ignored", deprecated);
    //     }
    // }

    // Handle visibility
    let visible = get_bool_prop(&props, "visible", Some(true))?;
    if visible {
        gtk_widget.show();
    } else {
        gtk_widget.hide();
    }

    // Handle classes
    if let Ok(class_str) = get_string_prop(&props, "class", None) {
        let style_context = gtk_widget.style_context();

        // remove all classes
        for class in gtk_widget.css_classes() {
            style_context.remove_class(&class);
        }

        // then apply the classes
        for class in class_str.split_whitespace() {
            style_context.add_class(class);
        }
    }

    if let Ok(style_str) = get_string_prop(&props, "style", None) {
        let css_provider = gtk4::CssProvider::new();
        let scss = format!("* {{ {} }}", style_str);
        css_provider.load_from_data(&grass::from_string(scss, &grass::Options::default())?);
        gtk_widget.style_context().add_provider(&css_provider, 950);
    }

    if let Ok(css_str) = get_string_prop(&props, "css", None) {
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(&grass::from_string(css_str, &grass::Options::default())?);
        gtk_widget.style_context().add_provider(&css_provider, 950);
    }

    if let Ok(valign) = get_string_prop(&props, "valign", None) {
        gtk_widget.set_valign(parse_align(&valign)?)
    }

    if let Ok(halign) = get_string_prop(&props, "halign", None) {
        gtk_widget.set_halign(parse_align(&halign)?)
    }

    let vexpand = get_bool_prop(&props, "vexpand", Some(false))?;
    gtk_widget.set_vexpand(vexpand);

    let hexpand = get_bool_prop(&props, "hexpand", Some(false))?;
    gtk_widget.set_hexpand(hexpand);

    let width = get_i32_prop(&props, "width", None).ok();
    let height = get_i32_prop(&props, "height", None).ok();

    match (width, height) {
        (Some(w), Some(h)) => gtk_widget.set_size_request(w, h),
        (Some(w), None) => gtk_widget.set_size_request(w, gtk_widget.allocated_height()),
        (None, Some(h)) => gtk_widget.set_size_request(gtk_widget.allocated_width(), h),
        (None, None) => {} // do nothing
    }

    let active = get_bool_prop(&props, "active", Some(true))?;
    gtk_widget.set_sensitive(active);

    if let Ok(tooltip) = get_string_prop(&props, "tooltip", None) {
        gtk_widget.set_tooltip_text(Some(&tooltip));
    }

    if let Ok(can_target) = get_bool_prop(&props, "can_target", None) {
        gtk_widget.set_can_target(can_target);
    }

    Ok(())
}

/// Shared rage atribute
pub(super) fn resolve_range_attrs(
    props: &Map,
    gtk_widget: &gtk4::Range,
    range_dat: Rc<RefCell<RangeCtrlData>>,
) -> Result<()> {
    // We keep track of the last value that has been set via gtk_widget.set_value (by a change in the value property).
    // We do this so we can detect if the new value came from a scripted change or from a user input from within the value_changed handler
    // and only run on_change when it's caused by manual user input
    if let Ok(value) = get_f64_prop(&props, "value", None) {
        if !range_dat.borrow().is_being_dragged {
            range_dat.borrow_mut().last_set_value = Some(value);
            gtk_widget.set_value(value);
        }
    }

    if let Ok(min) = get_f64_prop(&props, "min", None) {
        gtk_widget.adjustment().set_lower(min)
    }

    if let Ok(max) = get_f64_prop(&props, "max", None) {
        gtk_widget.adjustment().set_upper(max)
    }

    let onchange = get_string_prop(&props, "onchange", None).ok();
    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    if let Some(onchange) = onchange {
        range_dat.borrow_mut().onchange_cmd = onchange;
        range_dat.borrow_mut().cmd_timeout = timeout;
    }

    Ok(())
}
