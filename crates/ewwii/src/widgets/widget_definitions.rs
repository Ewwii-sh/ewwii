#![allow(clippy::option_map_unit_fn)]

use crate::util;
use crate::widgets::build_widget::{build_gtk_widget, WidgetInput};
use crate::{bind_property, apply_property, apply_property_watch};
use anyhow::{anyhow, bail, Result};
use gtk4::gdk::DragAction;
use gtk4::{self, prelude::*};
use gtk4::{gdk, glib, pango};
use gtk4::{
    DragSource, DropTarget, EventControllerKey, EventControllerLegacy, EventControllerMotion,
    EventControllerScroll, GestureClick,
};
use rhai::Map;
use rhai_impl::ast::{hash_props_and_type, WidgetNode};

use super::widget_definitions_helper::*;
use shared_utils::prop_utils::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    // cmp::Ordering,
    rc::Rc,
    time::Duration,
};

// custom widgets
// use crate::widgets::{circular_progressbar::CircProg, transform::Transform};
use crate::widgets::circular_progressbar::CircProg;
use crate::widgets::graph::{Graph, RenderType};

pub struct WidgetRegistry {
    pub widgets: HashMap<u64, gtk4::Widget>,
    pub window_widgets: HashMap<String, Vec<u64>>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self { widgets: HashMap::new(), window_widgets: HashMap::new() }
    }

    pub fn create_widget(
        &mut self,
        widget_node: &WidgetNode,
        widget_id: u64,
        parent_id: u64,
    ) -> Result<()> {
        log::trace!("Creating '{}'", widget_id);
        if let Some(parent) = self.widgets.get(&parent_id) {
            let parent_widget = parent.clone();

            // find old siblings if the widget already exists
            let (prev_sibling, next_sibling) = if let Some(old_widget) = self.widgets.get(&widget_id)
            {
                (old_widget.prev_sibling(), old_widget.next_sibling())
            } else {
                (None, None)
            };

            // check if widget already exists
            if let Some(old_widget) = self.widgets.remove(&widget_id) {
                // obliterate that widget....
                // how dare it try to create duplication...
                old_widget.unparent();
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
                gtk_widget.set_parent(&parent_widget);
            }
        }

        Ok(())
    }

    pub fn remove_widget(&mut self, widget_id: u64) {
        log::trace!("Removing '{}'", widget_id);
        if let Some(widget) = self.widgets.remove(&widget_id) {
            widget.unparent();
        }
    }

    pub fn remove_widget_by_name(&mut self, name: &str) -> bool {
        if let Some((&id, _)) =
            self.widgets.iter().find(|(_, widget)| widget.widget_name().as_str() == name)
        {
            if let Some(widget) = self.widgets.remove(&id) {
                widget.unparent();
                log::info!("Deleted widget '{}' on command.", name);
                return true;
            }
        }

        log::warn!("Widget '{}' not found", name);
        false
    }

    pub fn get_widget_id_by_name(&self, name: &str) -> Option<u64> {
        self.widgets
            .iter()
            .find(|(_, widget)| widget.widget_name().as_str() == name)
            .map(|(&id, _)| id)
    }

    pub fn get_property_by_name(&self, widget_name: &str, property: &str) -> Option<String> {
        let widget = self.widgets
            .values()
            .find(|widget| widget.widget_name() == widget_name)?;

        let value: glib::Value = widget.property_value(property);

        if let Ok(s) = value.get::<String>() {
            return Some(s);
        }

        if let Ok(b) = value.get::<bool>() {
            return Some(b.to_string());
        }

        if let Ok(i) = value.get::<i32>() {
            return Some(i.to_string());
        }

        if let Ok(f) = value.get::<f64>() {
            return Some(f.to_string());
        }

        None
    }

    pub fn update_property_by_name(
        &mut self,
        widget_name: &str,
        property_and_value: (String, String),
    ) -> bool {
        if let Some((&id, _)) = self
            .widgets
            .iter()
            .find(|(_, widget)| widget.widget_name().as_str() == widget_name)
        {
            if let Some(widget) = self.widgets.get(&id) {
                set_property_from_string_anywhere(
                    widget,
                    &property_and_value.0,
                    &property_and_value.1,
                );
            }
        }

        false
    }

    pub fn update_class_of_widget_by_name(
        &mut self,
        widget_name: &str,
        class: &str,
        remove: bool,
    ) -> bool {
        if let Some((&id, _)) = self
            .widgets
            .iter()
            .find(|(_, widget)| widget.widget_name().as_str() == widget_name)
        {
            if let Some(widget) = self.widgets.get(&id) {
                if !remove {
                    widget.add_css_class(class);
                } else {
                    widget.remove_css_class(class);
                }
            }
        }

        false
    }
}

pub(super) fn build_gtk_box(
    props: &Map,
    children: &Vec<WidgetNode>,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Box> {
    let gtk_widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    bind_property!(&props, "orientation", get_string_prop, Some("h"), [gtk_widget], |v: String| {
        if let Ok(o) = parse_orientation(&v) {
            gtk_widget.set_orientation(o)
        }
    });
    bind_property!(&props, "spacing", get_i64_prop, Some(0), [gtk_widget], |v: i64| {
        gtk_widget.set_spacing(v as i32)
    });
    bind_property!(&props, "space_evenly", get_bool_prop, Some(true), [gtk_widget], |v: bool| {
        gtk_widget.set_homogeneous(v)
    });

    for child in children {
        let child_widget = build_gtk_widget(&WidgetInput::BorrowedNode(child), widget_registry)?;
        gtk_widget.append(&child_widget);
    }

    let id = hash_props_and_type(&props, "Box");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
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

    let id = hash_props_and_type(&props, "Overlay");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
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

    let id = hash_props_and_type(&props, "Tooltip");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
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
    let gtk_widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    // controllers
    let hover_controller = EventControllerMotion::new();
    let gesture_controller = GestureClick::new();
    gesture_controller.set_button(0);
    let scroll_controller = EventControllerScroll::new(gtk4::EventControllerScrollFlags::BOTH_AXES);
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

    // Support :active selector and onclick variant commands
    gesture_controller.connect_pressed(glib::clone!(
        #[weak]
        gtk_widget,
        #[strong]
        controller_data,
        move |gesture, _, _, _| {
            gtk_widget.set_state_flags(gtk4::StateFlags::ACTIVE, false);

            let controller = controller_data.borrow();
            let button = gesture.current_button();

            match button {
                1 => run_command(controller.cmd_timeout, &controller.onclick_cmd, &[] as &[&str]),
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
    ));

    gesture_controller.connect_released(glib::clone!(
        #[weak]
        gtk_widget,
        move |_, _, _, _| {
            gtk_widget.unset_state_flags(gtk4::StateFlags::ACTIVE);
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
    gtk_widget.add_controller(drop_text_target);
    gtk_widget.add_controller(drop_uri_target);
    gtk_widget.add_controller(drag_source);
    gtk_widget.add_controller(key_controller);

    // timeout - timeout of the command. Default: "200ms"
    controller_data.borrow_mut().cmd_timeout =
        get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    // onscroll - event to execute when the user scrolls with the mouse over the widget. The placeholder `{}` used in the command will be replaced with either `up` or `down`.
    bind_property!(&props, "onscroll", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().onscroll_cmd = v;
    });

    // onhover - event to execute when the user hovers over the widget
    bind_property!(&props, "onhover", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().onhover_cmd = v;
    });

    // onhoverlost - event to execute when the user loses hover over the widget
    bind_property!(&props, "onhoverlost", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().onhoverlost_cmd = v;
    });

    // cursor - Cursor to show while hovering (see [gtk3-cursors](https://docs.gtk.org/gdk3/ctor.Cursor.new_from_name.html) for possible names)
    bind_property!(&props, "cursor", get_string_prop, Some("default"), [controller_data], |v: String| {
        controller_data.borrow_mut().hover_cursor = v;
    });

    // ondropped - Command to execute when something is dropped on top of this element. The placeholder `{}` used in the command will be replaced with the uri to the dropped thing.
    bind_property!(&props, "ondropped", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().ondropped_cmd = v;
    });

    // dragtype - Type of value that should be dragged from this widget. Possible values: $dragtype
    bind_property!(&props, "drag_type", get_string_prop, Some("file"), [controller_data], |v: String| {
        if let Ok(dt) = parse_dragtype(&v) {
            controller_data.borrow_mut().dragtype = dt;
        }
    });

    // dragvalue - URI that will be provided when dragging from this widget
    bind_property!(&props, "dragvalue", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().dragvalue = v;
    });

    // onclick - command to run when the widget is clicked
    bind_property!(&props, "onclick", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().onclick_cmd = v;
    });

    // onmiddleclick - command to run when the widget is middleclicked
    bind_property!(&props, "onmiddleclick", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().onmiddleclick_cmd = v;
    });

    // onrightclick - command to run when the widget is rightclicked
    bind_property!(&props, "onrightclick", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().onrightclick_cmd = v;
    });

    // onkeypress - command to run when a key is pressed
    bind_property!(&props, "onkeypress", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().onkeypress_cmd = Some(v);
    });

    // onkeyrelease - command to run when a key is released
    bind_property!(&props, "onkeyrelease", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().onkeyrelease_cmd = Some(v);
    });

    bind_property!(&props, "orientation", get_string_prop, Some("h"), [gtk_widget], |v: String| {
        if let Ok(o) = parse_orientation(&v) {
            gtk_widget.set_orientation(o);
        }
    });

    bind_property!(&props, "spacing", get_i64_prop, Some(0), [gtk_widget], |v: i64| {
        gtk_widget.set_spacing(v as i32);
    });

    bind_property!(&props, "space_evenly", get_bool_prop, Some(true), [gtk_widget], |v: bool| {
        gtk_widget.set_homogeneous(v);
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

    let id = hash_props_and_type(&props, "EventBox");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;
    Ok(gtk_widget)
}

struct FlowBoxCtrlData {
    onaccept_cmd: String,
    cmd_timeout: Duration,
}

pub(crate) fn build_gtk_flowbox(
    props: &Map,
    children: &Vec<WidgetNode>,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::FlowBox> {
    let gtk_widget = gtk4::FlowBox::new();

    let controller_data = Rc::new(RefCell::new(FlowBoxCtrlData {
        onaccept_cmd: String::new(),
        cmd_timeout: Duration::from_millis(200),
    }));

    gtk_widget.connect_child_activated(glib::clone!(
        #[strong]
        controller_data,
        move |_, flow_child: &gtk4::FlowBoxChild| {
            let controller = controller_data.borrow();

            if let Some(child) = flow_child.child() {
                let widget_name = child.widget_name();
                run_command(controller.cmd_timeout, &controller.onaccept_cmd, &[widget_name]);
            } else {
                log::error!("Failed to get the child of FlowBoxChild.");
            }
        }
    ));

    let mut index = 0;

    for child in children {
        let child_widget = build_gtk_widget(&WidgetInput::BorrowedNode(child), widget_registry)?;
        gtk_widget.insert(&child_widget, index);
        index += 1;
    }

    bind_property!(&props, "default_select", get_i32_prop, None, [gtk_widget], |dsv: i32| {
        if let Some(child) = gtk_widget.child_at_index(dsv) {
            gtk_widget.select_child(&child);
            child.grab_focus();
        } else {
            log::error!("Failed to get child at index {} from FlowBox", dsv);
        }
    });

    bind_property!(&props, "orientation", get_string_prop, Some("h"), [gtk_widget], |v: String| {
        if let Ok(o) = parse_orientation(&v) {
            gtk_widget.set_orientation(o);
        }
    });

    bind_property!(&props, "space_evenly", get_bool_prop, Some(true), [gtk_widget], |v: bool| {
        gtk_widget.set_homogeneous(v);
    });

    bind_property!(&props, "selection_model", get_string_prop, None, [gtk_widget], |v: String| {
        if let Ok(selection_model) = parse_selection_model(&v) {
            gtk_widget.set_selection_mode(selection_model);
        }
    });

    bind_property!(&props, "onaccept", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().onaccept_cmd = v;
    });

    let id = hash_props_and_type(&props, "FlowBox");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
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
    }

    // parsing the properties
    bind_property!(&props, "selected", get_i32_prop, None, [gtk_widget], |v: i32| {
        gtk_widget.set_visible_child_name(&v.to_string());
    });

    bind_property!(&props, "transition", get_string_prop, Some("crossfade"), [gtk_widget], |v: String| {
        if let Ok(t) = parse_stack_transition(&v) {
            gtk_widget.set_transition_type(t);
        }
    });

    bind_property!(&props, "transition_duration", get_i32_prop, None, [gtk_widget], |v: i32| {
        gtk_widget.set_transition_duration(v as u32);
    });

    // let same_size = get_bool_prop(&props, "same_size", Some(false))?;
    // gtk_widget.set_homogeneous(same_size);

    let id = hash_props_and_type(&props, "Stack");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;
    Ok(gtk_widget)
}

pub(super) fn build_circular_progress_bar(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<CircProg> {
    let widget = CircProg::new();

    bind_property!(&props, "value", get_f64_prop, None, [widget], |v: f64| {
        widget.set_property("value", v.clamp(0.0, 100.0));
    });

    bind_property!(&props, "start_at", get_f64_prop, None, [widget], |v: f64| {
        widget.set_property("start-at", v.clamp(0.0, 100.0));
    });

    bind_property!(&props, "thickness", get_f64_prop, None, [widget], |v: f64| {
        widget.set_property("thickness", v);
    });

    bind_property!(&props, "clockwise", get_bool_prop, None, [widget], |v: bool| {
        widget.set_property("clockwise", v);
    });

    bind_property!(&props, "fg_color", get_string_prop, None, [widget], |v: String| {
        if let Ok(rgba) = gdk::RGBA::parse(v) {
            widget.set_property("fg-color", rgba);
        }
    });

    bind_property!(&props, "bg_color", get_string_prop, None, [widget], |v: String| {
        if let Ok(rgba) = gdk::RGBA::parse(v) {
            widget.set_property("bg-color", rgba);
        }
    });

    let id = hash_props_and_type(&props, "CircularProgress");
    widget_registry.widgets.insert(id, widget.clone().upcast());
    resolve_rhai_widget_attrs(&widget.clone().upcast::<gtk4::Widget>(), &props)?;
    Ok(widget)
}

pub(super) fn build_graph(props: &Map, widget_registry: &mut WidgetRegistry) -> Result<Graph> {
    let widget = Graph::new();

    bind_property!(&props, "value", get_f64_prop, None, [widget], |value: f64| {
        if value.is_nan() || value.is_infinite() {
            log::error!("Graph's value should never be NaN or infinite");
            return;
        }
        widget.set_property("value", value);
    });

    if let Ok(time_range) = get_duration_prop(&props, "time_range", None) {
        let millis = time_range.as_millis();
        let millis_u32 = u32::try_from(millis).map_err(|_| {
            anyhow!(
                "Graph's time_range ({}ms) exceeds maximum representable ({}ms)",
                millis,
                u32::MAX
            )
        })?;

        widget.set_property("time-range", millis_u32);
    }

    let min_val: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.0));
    let max_val: Rc<RefCell<f64>> = Rc::new(RefCell::new(100.0));

    let apply_min_max = {
        let widget = widget.clone();
        let min_val = min_val.clone();
        let max_val = max_val.clone();
        Rc::new(move || {
            let min = *min_val.borrow();
            let max = *max_val.borrow();
            if min > max {
                log::error!("Graph's min ({min}) should never be higher than max ({max})");
                return;
            }
            widget.set_property("min", min);
            widget.set_property("max", max);
        })
    };

    bind_property!(&props, "min", get_f64_prop, Some(0.0), [min_val, apply_min_max], |v: f64| {
        *min_val.borrow_mut() = v;
        apply_min_max();
    });

    bind_property!(&props, "max", get_f64_prop, Some(100.0), [max_val, apply_min_max], |v: f64| {
        *max_val.borrow_mut() = v;
        apply_min_max();
    });

    bind_property!(&props, "dynamic", get_bool_prop, None, [widget], |dynamic: bool| {
        widget.set_property("dynamic", dynamic);
    });

    bind_property!(&props, "type", get_string_prop, None, [widget], |render_type: String| {
        match parse_graph_render_type(render_type.as_str()) {
            Ok(t) => widget.set_property("type", t),
            Err(e) => {
                log::error!("Failed to parse graph type property: {}", e);
                return;
            },
        };
    });

    bind_property!(&props, "thickness", get_f64_prop, None, [widget], |thickness: f64| {
        if !matches!(widget.property("type"), RenderType::Line | RenderType::StepLine) {
            log::error!("Property thickness can only be used with line graphs");
            return;
        }

        widget.set_property("thickness", thickness);
    });

    bind_property!(&props, "line_style", get_string_prop, None, [widget], |line_style: String| {
        if !matches!(widget.property("type"), RenderType::Line | RenderType::StepLine) {
            log::error!("Property line-style can only be used with line graphs");
            return;
        }

        match parse_graph_line_style(line_style.as_str()) {
            Ok(ls) => widget.set_property("line-style", ls),
            Err(e) => {
                log::error!("Failed to parse graph line-style property: {}", e);
                return;
            },
        };
    });

    // flip-x - whether the x axis should go from high to low
    bind_property!(&props, "flip_x", get_bool_prop, None, [widget], |flip_x: bool| {
        widget.set_property("flip-x", flip_x);
    });

    // flip-y - whether the y axis should go from high to low
    bind_property!(&props, "flip_y", get_bool_prop, None, [widget], |flip_y: bool| {
        widget.set_property("flip-y", flip_y);
    });

    // vertical - if set to true, the x and y axes will be exchanged
    bind_property!(&props, "vertical", get_bool_prop, None, [widget], |vertical: bool| {
        widget.set_property("vertical", vertical);
    });

    bind_property!(&props, "animate", get_bool_prop, None, [widget], |animate: bool| {
        widget.set_property("animate", animate);
    });

    let id = hash_props_and_type(&props, "Graph");
    widget_registry.widgets.insert(id, widget.clone().upcast());
    resolve_rhai_widget_attrs(&widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(widget)
}

pub(super) fn build_gtk_progress(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ProgressBar> {
    let gtk_widget = gtk4::ProgressBar::new();

    bind_property!(&props, "orientation", get_string_prop, Some("h"), [gtk_widget], |v: String| {
        if let Ok(o) = parse_orientation(&v) {
            gtk_widget.set_orientation(o)
        }
    });

    bind_property!(&props, "flipped", get_bool_prop, Some(false), [gtk_widget], |flipped: bool| {
        gtk_widget.set_inverted(flipped)
    });

    bind_property!(&props, "value", get_f64_prop, None, [gtk_widget], |bar_value: f64| {
        gtk_widget.set_fraction(bar_value / 100f64);
    });

    bind_property!(&props, "text", get_string_prop, None, [gtk_widget], |bar_text: String| {
        gtk_widget.set_text(Some(&bar_text));
    });

    bind_property!(&props, "show_text", get_bool_prop, None, [gtk_widget], |show_text: bool| {
        gtk_widget.set_show_text(show_text);
    });

    let id = hash_props_and_type(&props, "Progress");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;
    Ok(gtk_widget)
}

pub(super) fn build_image(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Picture> {
    let gtk_widget = gtk4::Picture::new();

    let path_prop = get_string_prop(&props, "path", None)?;
    let image_width_prop = get_i32_prop(&props, "image_width", Some(-1))?;
    let image_height_prop = get_i32_prop(&props, "image_height", Some(-1))?;
    let preserve_aspect_ratio_prop = get_bool_prop(&props, "preserve_aspect_ratio", Some(true))?;
    let fill_svg_prop = get_string_prop(&props, "fill_svg", Some(""))?;

    let current_path = Rc::new(RefCell::new(path_prop.initial_value()));
    let current_image_width = Rc::new(RefCell::new(image_width_prop.initial_value()));
    let current_image_height = Rc::new(RefCell::new(image_height_prop.initial_value()));
    let current_preserve_aspect_ratio = Rc::new(RefCell::new(preserve_aspect_ratio_prop.initial_value()));
    let current_fill_svg = Rc::new(RefCell::new(fill_svg_prop.initial_value()));

    let re_render = {
        let gtk_widget = gtk_widget.clone();
        let current_path = current_path.clone();
        let current_image_width = current_image_width.clone();
        let current_image_height = current_image_height.clone();
        let current_preserve_aspect_ratio = current_preserve_aspect_ratio.clone();
        let current_fill_svg = current_fill_svg.clone();

        Rc::new(move || {
            let path = current_path.borrow().clone();
            let image_width = *current_image_width.borrow();
            let image_height = *current_image_height.borrow();
            let preserve_aspect_ratio = *current_preserve_aspect_ratio.borrow();
            let fill_svg = current_fill_svg.borrow().clone();

            // is multiplied by 2 because it appears that it looks 
            // half in size in my screen.
            gtk_widget.set_height_request(image_height * 2);
            gtk_widget.set_width_request(image_width * 2);

            if !path.ends_with(".svg") && !fill_svg.is_empty() {
                log::warn!("Fill attribute ignored, file is not an svg image");
            }

            if path.ends_with(".gif") {
                let pixbuf_animation = match gtk4::gdk_pixbuf::PixbufAnimation::from_file(
                    std::path::PathBuf::from(&path),
                ) {
                    Ok(a) => a,
                    Err(e) => {
                        log::error!("Failed to load GIF `{path}`: {e}");
                        return;
                    }
                };
                let iter = pixbuf_animation.iter(None);
                let frame_pixbuf = iter.pixbuf();
                gtk_widget.set_pixbuf(Some(&frame_pixbuf));
                let widget_clone = gtk_widget.clone();
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
                let scale = gtk_widget.scale_factor();
                let pixbuf = if path.ends_with(".svg") && !fill_svg.is_empty() {
                    let svg_data = match std::fs::read_to_string(std::path::PathBuf::from(&path)) {
                        Ok(d) => d,
                        Err(e) => {
                            log::error!("Failed to read SVG `{path}`: {e}");
                            return;
                        }
                    };
                    let svg_data = if svg_data.contains("fill=") {
                        let reg = match regex::Regex::new(r#"fill="[^"]*""#) {
                            Ok(r) => r,
                            Err(e) => {
                                log::error!("Regex error: {e}");
                                return;
                            }
                        };
                        reg.replace(&svg_data, &format!("fill=\"{}\"", fill_svg))
                    } else {
                        let reg = match regex::Regex::new(r"<svg") {
                            Ok(r) => r,
                            Err(e) => {
                                log::error!("Regex error: {e}");
                                return;
                            }
                        };
                        reg.replace(&svg_data, &format!("<svg fill=\"{}\"", fill_svg))
                    };
                    let stream = gtk4::gio::MemoryInputStream::from_bytes(
                        &gtk4::glib::Bytes::from(svg_data.as_bytes()),
                    );
                    let result = gtk4::gdk_pixbuf::Pixbuf::from_stream_at_scale(
                        &stream,
                        image_width * scale,
                        image_height * scale,
                        preserve_aspect_ratio,
                        None::<&gtk4::gio::Cancellable>,
                    );
                    if let Err(e) = stream.close(None::<&gtk4::gio::Cancellable>) {
                        log::error!("Failed to close SVG stream: {e}");
                    }
                    match result {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("Failed to render SVG `{path}`: {e}");
                            return;
                        }
                    }
                } else {
                    let width = if image_width > 0 { image_width * scale } else { -1 };
                    let height = if image_height > 0 { image_height * scale } else { -1 };
                    match gtk4::gdk_pixbuf::Pixbuf::from_file_at_scale(
                        std::path::PathBuf::from(&path),
                        width,
                        height,
                        preserve_aspect_ratio,
                    ) {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("Failed to load image `{path}`: {e}");
                            return;
                        }
                    }
                };

                gtk_widget.set_pixbuf(Some(&pixbuf));
            }
        })
    };

    // Single initial render
    re_render();

    // Watch for future changes on bound props only
    apply_property_watch!(path_prop, [current_path, re_render], |v: String| {
        *current_path.borrow_mut() = v;
        re_render();
    });
    apply_property_watch!(image_width_prop, [current_image_width, re_render], |v: i32| {
        *current_image_width.borrow_mut() = v;
        re_render();
    });
    apply_property_watch!(image_height_prop, [current_image_height, re_render], |v: i32| {
        *current_image_height.borrow_mut() = v;
        re_render();
    });
    apply_property_watch!(preserve_aspect_ratio_prop, [current_preserve_aspect_ratio, re_render], |v: bool| {
        *current_preserve_aspect_ratio.borrow_mut() = v;
        re_render();
    });
    apply_property_watch!(fill_svg_prop, [current_fill_svg, re_render], |v: String| {
        *current_fill_svg.borrow_mut() = v;
        re_render();
    });

    bind_property!(&props, "content_fit", get_string_prop, Some("contain"), [gtk_widget], |v: String| {
        if let Ok(content_fit) = parse_content_fit(&v) {
            gtk_widget.set_content_fit(content_fit);
        };
    });

    bind_property!(&props, "can_shrink", get_bool_prop, Some(true), [gtk_widget], |v: bool| {
        gtk_widget.set_can_shrink(v);
    });

    let id = hash_props_and_type(&props, "Image");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
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
    let gesture_controller = GestureClick::new();
    gesture_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    gesture_controller.set_button(0);

    gtk_widget.connect_clicked(glib::clone!(
        #[weak]
        gtk_widget,
        move |_| {
            gtk_widget.emit_activate();
        }
    ));

    gesture_controller.connect_pressed(glib::clone!(
        #[strong]
        controller_data,
        move |gesture, _, _, _| {
            let button = gesture.current_button();
            let controller = controller_data.borrow();
            match button {
                1 => run_command(controller.cmd_timeout, &controller.onclick_cmd, &[] as &[&str]),
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
    gtk_widget.add_controller(gesture_controller);

    controller_data.borrow_mut().cmd_timeout =
        get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    bind_property!(&props, "onclick", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().onclick_cmd = v;
    });

    bind_property!(&props, "onmiddleclick", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().onmiddleclick_cmd = v;
    });

    bind_property!(&props, "onrightclick", get_string_prop, None, [controller_data], |v: String| {
        controller_data.borrow_mut().onrightclick_cmd = v;
    });

    bind_property!(&props, "label", get_string_prop, None, [gtk_widget], |lbl: String| {
        gtk_widget.set_label(&lbl);
    });

    let id = hash_props_and_type(&props, "Button");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;
    Ok(gtk_widget)
}

struct LabelTextParams {
    truncate: bool,
    limit_width: i32,
    truncate_left: bool,
    show_truncated: bool,
    unindent: bool,
}

pub(super) fn build_gtk_label(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Label> {
    let gtk_widget = gtk4::Label::new(None);

    let current_text: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let current_markup: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    let has_text = props.get("text").is_some();
    let has_markup = props.get("markup").is_some();

    if has_text && has_markup {
        bail!("Cannot set both 'text' and 'markup' for a label");
    }

    // Quick Info: 
    // limit_width wouldn't work if show_truncated is true.
    let truncate_prop = get_bool_prop(&props, "truncate", Some(false))?;
    let limit_width_prop = get_i32_prop(&props, "limit_width", Some(i32::MAX))?;
    let truncate_left_prop = get_bool_prop(&props, "truncate_left", Some(false))?;
    let show_truncated_prop = get_bool_prop(&props, "show_truncated", Some(false))?;
    let unindent_prop = get_bool_prop(&props, "unindent", Some(true))?;

    let text_params = Rc::new(RefCell::new(LabelTextParams {
        truncate: truncate_prop.initial_value(),
        limit_width: limit_width_prop.initial_value(),
        truncate_left: truncate_left_prop.initial_value(),
        show_truncated: show_truncated_prop.initial_value(),
        unindent: unindent_prop.initial_value(),
    }));

    if has_text {
        let text_prop = get_string_prop(&props, "text", None)?;
        *current_text.borrow_mut() = Some(text_prop.initial_value());
    } else if has_markup {
        let markup_prop = get_string_prop(&props, "markup", None)?;
        *current_markup.borrow_mut() = Some(markup_prop.initial_value());
    } else {
        bail!("Either 'text' or 'markup' must be set");
    }

    let re_render = {
        let gtk_widget = gtk_widget.clone();
        let current_text = current_text.clone();
        let current_markup = current_markup.clone();
        let text_params = text_params.clone();
        Rc::new(move || {
            let p = text_params.borrow();
            if let Some(text) = &*current_text.borrow() {
                let t = if p.show_truncated {
                    if p.limit_width == i32::MAX {
                        gtk_widget.set_max_width_chars(-1);
                    } else {
                        gtk_widget.set_max_width_chars(p.limit_width);
                    }
                    apply_ellipsize_settings(&gtk_widget, p.truncate, p.limit_width, p.truncate_left, p.show_truncated);
                    text.to_string()
                } else {
                    gtk_widget.set_ellipsize(pango::EllipsizeMode::None);
                    let limit_width = p.limit_width as usize;
                    let char_count = text.chars().count();
                    if char_count > limit_width {
                        if p.truncate_left {
                            text.chars().skip(char_count - limit_width).collect()
                        } else {
                            text.chars().take(limit_width).collect()
                        }
                    } else {
                        text.to_string()
                    }
                };
                match unescape::unescape(&t) {
                    Some(unescaped) => {
                        let final_text = if p.unindent { util::unindent(&unescaped) } else { unescaped };
                        gtk_widget.set_text(&final_text);
                    },
                    None => {
                        log::error!("Failed to unescape...");
                    }
                }
            } else if let Some(markup) = &*current_markup.borrow() {
                apply_ellipsize_settings(&gtk_widget, p.truncate, p.limit_width, p.truncate_left, p.show_truncated);
                gtk_widget.set_markup(markup);
            }
        })
    };

    // Single initial render
    re_render();

    apply_property_watch!(truncate_prop, [text_params, re_render], |v: bool| {
        text_params.borrow_mut().truncate = v;
        re_render();
    });
    apply_property_watch!(limit_width_prop, [text_params, re_render], |v: i32| {
        text_params.borrow_mut().limit_width = v;
        re_render();
    });
    apply_property_watch!(truncate_left_prop, [text_params, re_render], |v: bool| {
        text_params.borrow_mut().truncate_left = v;
        re_render();
    });
    apply_property_watch!(show_truncated_prop, [text_params, re_render], |v: bool| {
        text_params.borrow_mut().show_truncated = v;
        re_render();
    });
    apply_property_watch!(unindent_prop, [text_params, re_render], |v: bool| {
        text_params.borrow_mut().unindent = v;
        re_render();
    });

    if has_text {
        let text_prop = get_string_prop(&props, "text", None)?;
        apply_property_watch!(text_prop, [current_text, re_render], |text: String| {
            *current_text.borrow_mut() = Some(text);
            re_render();
        });
    } else if has_markup {
        let markup_prop = get_string_prop(&props, "markup", None)?;
        apply_property_watch!(markup_prop, [current_markup, re_render], |markup: String| {
            *current_markup.borrow_mut() = Some(markup);
            re_render();
        });
    }

    // independent properties
    bind_property!(&props, "wrap", get_bool_prop, Some(false), [gtk_widget], |wrap: bool| {
        gtk_widget.set_wrap(wrap);
    });

    // if let Ok(angle) = get_f64_prop(&props, "angle", Some(0.0)) {
    //     widget.set_angle(angle);
    // }

    bind_property!(&props, "gravity", get_string_prop, Some("south"), [gtk_widget], |grav: String| {
        if let Ok(v) = parse_gravity(&grav) {
            gtk_widget.pango_context().set_base_gravity(v);
        }
    });

    bind_property!(&props, "xalign", get_f64_prop, Some(0.5), [gtk_widget], |v: f64| {
        gtk_widget.set_xalign(v as f32);
    });

    bind_property!(&props, "yalign", get_f64_prop, Some(0.5), [gtk_widget], |v: f64| {
        gtk_widget.set_yalign(v as f32);
    });

    bind_property!(&props, "justify", get_string_prop, Some("left"), [gtk_widget], |justify: String| {
        if let Ok(v) = parse_justification(&justify) {
            gtk_widget.set_justify(v);
        }
    });

    bind_property!(&props, "wrap_mode", get_string_prop, Some("word"), [gtk_widget], |wrap: String| {
        if let Ok(v) = parse_wrap_mode(&wrap) {
            gtk_widget.set_wrap_mode(v);
        }
    });

    bind_property!(&props, "lines", get_i32_prop, Some(-1), [gtk_widget], |lines: i32| {
        gtk_widget.set_lines(lines);
    });

    let id = hash_props_and_type(&props, "Label");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_input(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Entry> {
    let gtk_widget = gtk4::Entry::new();

    bind_property!(&props, "value", get_string_prop, None, [gtk_widget], |value: String| {
        gtk_widget.set_text(&value);
    });

    bind_property!(&props, "placeholder", get_string_prop, None, [gtk_widget], |value: String| {
        gtk_widget.set_placeholder_text(Some(&value));
    });

    bind_property!(&props, "password", get_bool_prop, Some(false), [gtk_widget], |password: bool| {
        gtk_widget.set_visibility(!password);
    });

    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    let onchange_cmd = Rc::new(RefCell::new(None::<String>));
    bind_property!(&props, "onchange", get_string_prop, None, [onchange_cmd], |v: String| {
        *onchange_cmd.borrow_mut() = Some(v);
    });

    gtk_widget.connect_changed(glib::clone!(
        #[strong]
        onchange_cmd,
        move |widget| {
            if let Some(cmd) = &*onchange_cmd.borrow() {
                run_command(timeout, cmd, &[widget.text().to_string()]);
            }
        }
    ));

    let onaccept_cmd = Rc::new(RefCell::new(None::<String>));
    bind_property!(&props, "onaccept", get_string_prop, None, [onaccept_cmd], |v: String| {
        *onaccept_cmd.borrow_mut() = Some(v);
    });

    gtk_widget.connect_activate(glib::clone!(
        #[strong]
        onaccept_cmd,
        move |widget| {
            if let Some(cmd) = &*onaccept_cmd.borrow() {
                run_command(timeout, cmd, &[widget.text().to_string()]);
            }
        }
    ));

    let id = hash_props_and_type(&props, "Input");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_calendar(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Calendar> {
    let gtk_widget = gtk4::Calendar::new();

    // day - the selected day
    bind_property!(&props, "day", get_f64_prop, None, [gtk_widget], |day: f64| {
        if !(1f64..=31f64).contains(&day) {
            log::warn!("Calendar day is not a number between 1 and 31");
        } else {
            gtk_widget.set_day(day as i32);
        }
    });

    // month - the selected month
    bind_property!(&props, "month", get_f64_prop, None, [gtk_widget], |month: f64| {
        if !(1f64..=12f64).contains(&month) {
            log::warn!("Calendar month is not a number between 1 and 12");
        } else {
            gtk_widget.set_month(month as i32 - 1);
        }
    });

    // year - the selected year
    bind_property!(&props, "year", get_f64_prop, None, [gtk_widget], |year: f64| {
        gtk_widget.set_year(year as i32);
    });

    // // show-details - show details
    // bind_property!(&props, "show_details", get_bool_prop, None, |show_details| {
    //     gtk_widget.set_show_details(show_details);
    // });

    // show-heading - show heading line
    bind_property!(&props, "show_heading", get_bool_prop, None, [gtk_widget], |show_heading: bool| {
        gtk_widget.set_show_heading(show_heading);
    });

    // show-day-names - show names of days
    bind_property!(&props, "show_day_names", get_bool_prop, None, [gtk_widget], |show_day_names: bool| {
        gtk_widget.set_show_day_names(show_day_names);
    });

    // show-week-numbers - show week numbers
    bind_property!(&props, "show_week_numbers", get_bool_prop, None, [gtk_widget], |show_week_numbers: bool| {
        gtk_widget.set_show_week_numbers(show_week_numbers);
    });

    // timeout - timeout of the command. Default: "200ms"
    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    // onclick - command to run when the user selects a date. The `{0}` placeholder will be replaced by the selected day, `{1}` will be replaced by the month, and `{2}` by the year.
    let onclick_cmd = Rc::new(RefCell::new(None::<String>));

    bind_property!(&props, "onclick", get_string_prop, None, [onclick_cmd], |v: String| {
        *onclick_cmd.borrow_mut() = Some(v);
    });

    gtk_widget.connect_day_selected(glib::clone!(
        #[strong]
        onclick_cmd,
        move |w| {
            if let Some(cmd) = &*onclick_cmd.borrow() {
                run_command(timeout, cmd, &[w.day(), w.month(), w.year()]);
            }
        }
    ));

    let id = hash_props_and_type(&props, "Calendar");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

#[allow(deprecated)]
pub(super) fn build_gtk_combo_box_text(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ComboBoxText> {
    let gtk_widget = gtk4::ComboBoxText::new();

    if let Ok(items) = get_vec_string_prop(&props, "items", None) {
        let current_items: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(
            items.iter().map(|p| p.initial_value()).collect()
        ));

        let apply_items = {
            let gtk_widget = gtk_widget.clone();
            let current_items = current_items.clone();
            Rc::new(move || {
                gtk_widget.remove_all();
                for item in current_items.borrow().iter() {
                    gtk_widget.append_text(item);
                }
            })
        };

        apply_items();

        for (i, item) in items.into_iter().enumerate() {
            apply_property_watch!(item, [current_items, apply_items], |v: String| {
                current_items.borrow_mut()[i] = v;
                apply_items();
            });
        }
    }

    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    let onchange_cmd = Rc::new(RefCell::new(String::new()));

    bind_property!(&props, "onchange", get_string_prop, Some(""), [onchange_cmd], |v: String| {
        *onchange_cmd.borrow_mut() = v;
    });

    gtk_widget.connect_changed(glib::clone!(
        #[strong]
        onchange_cmd,
        move |widget| {
            run_command(
                timeout,
                &onchange_cmd.borrow(),
                &[widget.active_text().unwrap_or_else(|| "".into())],
            );
        }
    ));

    let id = hash_props_and_type(&props, "ComboBoxText");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_ui_file(props: &Map) -> Result<gtk4::Widget> {
    let path = unwrap_static("file", get_string_prop(&props, "file", None)?);
    let main_id = unwrap_static("id", get_string_prop(&props, "id", None)?);

    if !std::path::Path::new(&path).exists() {
        return Err(anyhow::anyhow!("UI file not found: {}", path));
    }

    let builder = gtk4::Builder::from_file(&path);

    let gtk_widget = builder
        .object(&main_id)
        .ok_or_else(|| anyhow::anyhow!("No widget with id '{}' in {}", main_id, path))?;

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


    bind_property!(&props, "name", get_string_prop, None, [gtk_widget], |name: String| {
        gtk_widget.set_label(Some(&name));
    });

    bind_property!(&props, "expanded", get_bool_prop, None, [gtk_widget], |expanded: bool| {
        gtk_widget.set_expanded(expanded);
    });

    let id = hash_props_and_type(&props, "Expander");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;
    Ok(gtk_widget)
}

pub(super) fn build_gtk_revealer(
    props: &Map,
    children: &Vec<WidgetNode>,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Revealer> {
    let gtk_widget = gtk4::Revealer::new();

    bind_property!(&props, "transition", get_string_prop, Some("crossfade"), [gtk_widget], |transition: String| {
        if let Ok(t) = parse_revealer_transition(&transition) {
            gtk_widget.set_transition_type(t);
        }
    });

    bind_property!(&props, "reveal", get_bool_prop, None, [gtk_widget], |reveal: bool| {
        gtk_widget.set_reveal_child(reveal);
    });

    let duration = get_duration_prop(&props, "timeout", Some(Duration::from_millis(500)))?;
    gtk_widget.set_transition_duration(duration.as_millis() as u32);

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
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_gtk_checkbox(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::CheckButton> {
    let gtk_widget = gtk4::CheckButton::new();

    bind_property!(&props, "checked", get_bool_prop, Some(false), [gtk_widget], |checked: bool| {
        gtk_widget.set_active(checked);
    });

    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;
    let onchecked_cmd = Rc::new(RefCell::new(String::new()));
    let onunchecked_cmd = Rc::new(RefCell::new(String::new()));

    bind_property!(&props, "onchecked", get_string_prop, Some(""), [onchecked_cmd], |v: String| {
        *onchecked_cmd.borrow_mut() = v;
    });

    bind_property!(&props, "onunchecked", get_string_prop, Some(""), [onunchecked_cmd], |v: String| {
        *onunchecked_cmd.borrow_mut() = v;
    });

    gtk_widget.connect_toggled(glib::clone!(
        #[strong]
        onchecked_cmd,
        #[strong]
        onunchecked_cmd,
        move |widget| {
            let oncheck = &onchecked_cmd.borrow();
            let onuncheck = &onunchecked_cmd.borrow();

            run_command(
                timeout,
                if widget.is_active() { &oncheck } else { &onuncheck },
                &[] as &[&str],
            );
    }));

    let id = hash_props_and_type(&props, "Checkbox");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

#[allow(deprecated)]
pub(super) fn build_gtk_color_button(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ColorButton> {
    let gtk_widget = gtk4::ColorButton::builder().build();

    // use-alpha - bool to wether or not use alpha
    bind_property!(&props, "use_alpha", get_bool_prop, None, [gtk_widget], |use_alpha: bool| {
        gtk_widget.set_use_alpha(use_alpha);
    });

    // timeout - timeout of the command. Default: "200ms"
    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    // onchange - runs the code when the color was selected
    let onchange_cmd = Rc::new(RefCell::new(String::new()));

    bind_property!(&props, "onchange", get_string_prop, None, [onchange_cmd], |v: String| {
        *onchange_cmd.borrow_mut() = v;
    });

    gtk_widget.connect_color_set(glib::clone!(
        #[strong]
        onchange_cmd,
        move |widget| {
            run_command(timeout, &onchange_cmd.borrow(), &[widget.rgba()]);
        }
    ));

    let id = hash_props_and_type(&props, "ColorButton");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

#[allow(deprecated)]
pub(super) fn build_gtk_color_chooser(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ColorChooserWidget> {
    let gtk_widget = gtk4::ColorChooserWidget::new();

    // use-alpha - bool to wether or not use alpha
    bind_property!(&props, "use_alpha", get_bool_prop, None, [gtk_widget], |use_alpha: bool| {
        gtk_widget.set_use_alpha(use_alpha);
    });

    // timeout - timeout of the command. Default: "200ms"
    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    // onchange - runs the code when the color was selected
    let onchange_cmd = Rc::new(RefCell::new(String::new()));

    bind_property!(&props, "onchange", get_string_prop, None, [onchange_cmd], |v: String| {
        *onchange_cmd.borrow_mut() = v;
    });

    gtk_widget.connect_color_activated(glib::clone!(
        #[strong]
        onchange_cmd,
        move |_a, color| {
            run_command(timeout, &onchange_cmd.borrow(), &[*color]);
        }
    ));

    let id = hash_props_and_type(&props, "ColorChooser");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
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
    let gtk_widget =
        gtk4::Scale::new(gtk4::Orientation::Horizontal, Some(&gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 1.0)));

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
                gtk4::gdk::EventType::ButtonPress | gtk4::gdk::EventType::TouchBegin => {
                    scale_dat.borrow_mut().is_being_dragged = true;
                }
                gtk4::gdk::EventType::ButtonRelease | gtk4::gdk::EventType::TouchEnd => {
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

    bind_property!(&props, "orientation", get_string_prop, Some("h"), [gtk_widget], |v: String| {
        if let Ok(o) = parse_orientation(&v) {
            gtk_widget.set_orientation(o)
        }
    });

    bind_property!(&props, "flipped", get_bool_prop, Some(false), [gtk_widget], |v: bool| {
        gtk_widget.set_inverted(v);
    });

    bind_property!(&props, "marks", get_string_prop, None, [gtk_widget], |marks: String| {
        gtk_widget.clear_marks();
        for mark in marks.split(',') {
            if let Ok(val) = mark.trim().parse() {
                gtk_widget.add_mark(val, gtk4::PositionType::Bottom, None);
            }
        }
    });

    bind_property!(&props, "draw_value", get_bool_prop, Some(false), [gtk_widget], |v: bool| {
        gtk_widget.set_draw_value(v);
    });

    bind_property!(&props, "value_pos", get_string_prop, None, [gtk_widget], |value_pos: String| {
        if let Ok(pos) = parse_position_type(&value_pos) {
            gtk_widget.set_value_pos(pos);
        }
    });

    bind_property!(&props, "round_digits", get_i32_prop, Some(0), [gtk_widget], |v: i32| {
        gtk_widget.set_round_digits(v);
    });

    resolve_range_attrs(&props, gtk_widget.upcast_ref::<gtk4::Range>(), scale_dat)?;

    let id = hash_props_and_type(&props, "Scale");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
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

    bind_property!(&props, "hscroll", get_bool_prop, Some(true), [gtk_widget], |v: bool| {
        gtk_widget.set_hscrollbar_policy(
            if v { gtk4::PolicyType::Automatic } else { gtk4::PolicyType::Never }
        );
    });
    bind_property!(&props, "vscroll", get_bool_prop, Some(true), [gtk_widget], |v: bool| {
        gtk_widget.set_vscrollbar_policy(
            if v { gtk4::PolicyType::Automatic } else { gtk4::PolicyType::Never }
        );
    });

    bind_property!(&props, "propagate_natural_height", get_bool_prop, None, [gtk_widget], |natural_height: bool| {
        gtk_widget.set_propagate_natural_height(natural_height);
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

    let id = hash_props_and_type(&props, "ScrolledWindow");
    widget_registry.widgets.insert(id, gtk_widget.clone().upcast());
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
    bind_property!(&props, "visible", get_bool_prop, Some(true), [gtk_widget], |v: bool| {
        gtk_widget.set_visible(v);
    });

    // Handle classes
    bind_property!(&props, "class", get_string_prop, None, [gtk_widget], |class_str: String| {
        // remove all classes
        for class in gtk_widget.css_classes() {
            gtk_widget.remove_css_class(&class);
        }

        // then apply the classes
        for class in class_str.split_whitespace() {
            gtk_widget.add_css_class(class);
        }
    });

    // style and css need SCSS compilation
    bind_property!(&props, "style", get_string_prop, None, [gtk_widget], |style_str: String| {
        let css_provider = gtk4::CssProvider::new();
        let scss = format!("* {{ {} }}", style_str);
        if let Ok(compiled) = grass::from_string(scss, &grass::Options::default()) {
            css_provider.load_from_string(&compiled);
            gtk_widget.style_context().add_provider(&css_provider, 950);
        }
    });

    bind_property!(&props, "css", get_string_prop, None, [gtk_widget], |css_str: String| {
        let css_provider = gtk4::CssProvider::new();
        if let Ok(compiled) = grass::from_string(css_str, &grass::Options::default()) {
            css_provider.load_from_string(&compiled);
            gtk_widget.style_context().add_provider(&css_provider, 950);
        }
    });

    bind_property!(&props, "valign", get_string_prop, None, [gtk_widget], |valign: String| {
        if let Ok(a) = parse_align(&valign) {
            gtk_widget.set_valign(a);
        }
    });

    bind_property!(&props, "halign", get_string_prop, None, [gtk_widget], |halign: String| {
        if let Ok(a) = parse_align(&halign) {
            gtk_widget.set_halign(a);
        }
    });

    bind_property!(&props, "vexpand", get_bool_prop, Some(false), [gtk_widget], |v: bool| {
        gtk_widget.set_vexpand(v);
    });

    bind_property!(&props, "hexpand", get_bool_prop, Some(false), [gtk_widget], |v: bool| {
        gtk_widget.set_hexpand(v);
    });

    bind_property!(&props, "width", get_i32_prop, None, [gtk_widget], |w: i32| {
        gtk_widget.set_width_request(w);
    });

    bind_property!(&props, "height", get_i32_prop, None, [gtk_widget], |h: i32| {
        gtk_widget.set_height_request(h);
    });

    bind_property!(&props, "active", get_bool_prop, Some(true), [gtk_widget], |v: bool| {
        gtk_widget.set_sensitive(v);
    });

    bind_property!(&props, "tooltip", get_string_prop, None, [gtk_widget], |tooltip: String| {
        gtk_widget.set_tooltip_text(Some(&tooltip));
    });

    bind_property!(&props, "can_target", get_bool_prop, None, [gtk_widget], |v: bool| {
        gtk_widget.set_can_target(v);
    });

    bind_property!(&props, "focusable", get_bool_prop, Some(true), [gtk_widget], |v: bool| {
        gtk_widget.set_focusable(v);
    });

    bind_property!(&props, "widget_name", get_string_prop, None, [gtk_widget], |name: String| {
        gtk_widget.set_widget_name(&name);
    });

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
    bind_property!(&props, "value", get_f64_prop, None, [gtk_widget, range_dat], |v: f64| {
        if !range_dat.borrow().is_being_dragged {
            range_dat.borrow_mut().last_set_value = Some(v);
            gtk_widget.set_value(v);
        }
    });

    bind_property!(&props, "min", get_f64_prop, None, [gtk_widget], |min: f64| {
        gtk_widget.adjustment().set_lower(min);
    });

    bind_property!(&props, "max", get_f64_prop, None, [gtk_widget], |max: f64| {
        gtk_widget.adjustment().set_upper(max);
    });

    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    bind_property!(&props, "onchange", get_string_prop, None, [range_dat], |onchange: String| {
        range_dat.borrow_mut().onchange_cmd = onchange;
        range_dat.borrow_mut().cmd_timeout = timeout;
    });

    Ok(())
}