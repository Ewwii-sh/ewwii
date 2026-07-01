#![allow(clippy::option_map_unit_fn)]

use crate::widgets::build_widget::{build_gtk_widget, WidgetInput};
use crate::{apply_property, apply_property_watch, bind_property};
use anyhow::{anyhow, bail, Result};
use ewwii_shared_utils::ast::{hash_props_and_type, WidgetNode};
use ewwii_shared_utils::prop::{Property, PropertyMap};
use gtk4::gdk::DragAction;
use gtk4::{self, prelude::*};
use gtk4::{gdk, glib};
use gtk4::{
    DragSource, DropTarget, EventControllerKey, EventControllerLegacy, EventControllerMotion,
    EventControllerScroll, GestureClick,
};
use smart_default::SmartDefault;

use super::widget_definitions_helper::*;
use ewwii_shared_utils::prop_utils::*;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    // cmp::Ordering,
    rc::Rc,
    time::Duration,
};

// custom widgets
// use crate::widgets::{circular_progressbar::CircProg, transform::Transform};
use crate::widgets::circular_progressbar::CircProg;
use crate::widgets::ewwii_image::EwwiiImage;
use crate::widgets::ewwii_label::EwwiiLabel;
use crate::widgets::graph::{Graph, RenderType};

pub trait EwwiiWidget {
    fn widget(&self) -> &gtk4::Widget;
    fn build(
        &mut self,
        props: &PropertyMap,
        children: &[WidgetNode],
        widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget>;
    fn update_prop(&mut self, key: &str, value: &Property);
}

pub struct WidgetRegistry {
    pub widgets: HashMap<u64, Box<dyn EwwiiWidget>>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self { widgets: HashMap::new() }
    }

    pub fn create_widget(
        &mut self,
        widget_node: &WidgetNode,
        widget_id: u64,
        parent_id: u64,
    ) -> Result<()> {
        log::trace!("Creating '{}'", widget_id);
        if let Some(parent) = self.widgets.get(&parent_id) {
            let parent_widget = parent.widget().clone();

            // find old siblings if the widget already exists
            let (prev_sibling, next_sibling) =
                if let Some(old_widget) = self.widgets.get(&widget_id) {
                    (old_widget.widget().prev_sibling(), old_widget.widget().next_sibling())
                } else {
                    (None, None)
                };

            // check if widget already exists
            if let Some(old_widget) = self.widgets.remove(&widget_id) {
                // obliterate that widget....
                // how dare it try to create duplication...
                old_widget.widget().unparent();
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

    // pub fn remove_widget(&mut self, widget_id: u64) {
    //     log::trace!("Removing '{}'", widget_id);
    //     if let Some(widget) = self.widgets.remove(&widget_id) {
    //         widget.unparent();
    //     }
    // }

    pub fn remove_widget_by_name(&mut self, name: &str) -> bool {
        if let Some((&id, _)) =
            self.widgets.iter().find(|(_, widget)| widget.widget().widget_name().as_str() == name)
        {
            if let Some(widget) = self.widgets.remove(&id) {
                widget.widget().unparent();
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
            .find(|(_, widget)| widget.widget().widget_name().as_str() == name)
            .map(|(&id, _)| id)
    }

    pub fn get_property_by_name(&self, widget_name: &str, property: &str) -> Option<String> {
        let widget =
            self.widgets.values().find(|widget| widget.widget().widget_name() == widget_name)?;

        let widget_obj = widget.widget();
        let value: glib::Value = if widget_obj.find_property(property).is_some() {
            widget_obj.property_value(property)
        } else {
            return None;
        };

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
            .find(|(_, widget)| widget.widget().widget_name().as_str() == widget_name)
        {
            if let Some(widget) = self.widgets.get_mut(&id) {
                widget.update_prop(&property_and_value.0, &Property::String(property_and_value.1))
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
            .find(|(_, widget)| widget.widget().widget_name().as_str() == widget_name)
        {
            if let Some(widget) = self.widgets.get(&id) {
                if !remove {
                    widget.widget().add_css_class(class);
                } else {
                    widget.widget().remove_css_class(class);
                }
            }
        }

        false
    }
}

// === Widget Definition === //

#[derive(Default)]
struct BoxWidget {
    gtk_widget: gtk4::Box,
}

impl EwwiiWidget for BoxWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        children: &[WidgetNode],
        widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        self.gtk_widget.set_homogeneous(true);

        for (key, value) in props {
            self.update_prop(key, value);
        }

        for child in children {
            let child_widget =
                build_gtk_widget(&WidgetInput::BorrowedNode(child), widget_registry)?;
            self.gtk_widget.append(&child_widget);
        }

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "orientation" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |v: String| {
                    if let Ok(o) = parse_orientation(&v) {
                        gtk_widget.set_orientation(o)
                    }
                });
            }
            "spacing" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_i64_prop, [gtk_widget], |v: i64| {
                    gtk_widget.set_spacing(v as i32)
                });
            }
            "space_evenly" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |v: bool| {
                    gtk_widget.set_homogeneous(v)
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct OverlayWidget {
    gtk_widget: gtk4::Overlay,
}

impl EwwiiWidget for OverlayWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        children: &[WidgetNode],
        widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::Overlay::new();

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let count = children.len();
        if count < 1 {
            bail!("overlay must contain at least one element");
        }

        let mut children = children
            .iter()
            .map(|child| build_gtk_widget(&WidgetInput::BorrowedNode(child), widget_registry));

        // we have more than one child, we can unwrap
        let first = children.next().unwrap()?;
        self.gtk_widget.set_child(Some(&first));
        for child in children {
            let child = child?;
            self.gtk_widget.add_overlay(&child);
        }

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
    }
}

#[derive(Default)]
struct TooltipWidget {
    gtk_widget: gtk4::Box,
}

impl EwwiiWidget for TooltipWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        children: &[WidgetNode],
        widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        self.gtk_widget.set_has_tooltip(true);

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let count = children.len();
        if count < 2 {
            bail!("tooltip must contain exactly 2 children");
        } else if count > 2 {
            bail!("tooltip must contain exactly 2 children, but got more");
        }

        let tooltip_node = children.first().cloned().ok_or_else(|| anyhow!("missing tooltip"))?;
        let content_node = children.get(1).cloned().ok_or_else(|| anyhow!("missing content"))?;

        // The visible child immediately
        let content_widget = build_gtk_widget(&WidgetInput::Node(content_node), widget_registry)?;
        self.gtk_widget.append(&content_widget);

        let tooltip_node = Rc::new(tooltip_node);
        let tooltip_widget = build_gtk_widget(
            &WidgetInput::BorrowedNode(Rc::clone(&tooltip_node).as_ref()),
            widget_registry,
        )
        .expect("Failed to build tooltip widget");

        self.gtk_widget.connect_query_tooltip(move |_widget, _x, _y, _keyboard_mode, tooltip| {
            tooltip.set_custom(Some(&tooltip_widget));
            true
        });

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
    }
}

#[derive(SmartDefault)]
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
    dragvalue: Option<String>,
    #[default(DragEntryType::File)]
    dragtype: DragEntryType,

    // key controller data
    onkeypress_cmd: Option<String>,
    onkeyrelease_cmd: Option<String>,

    // other
    cmd_timeout: Duration,
}

#[derive(Default)]
struct EventBoxWidget {
    gtk_widget: gtk4::Box,
    controller: Rc<RefCell<EventBoxCtrlData>>,
}

impl EwwiiWidget for EventBoxWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        children: &[WidgetNode],
        widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        self.gtk_widget.set_homogeneous(true);
        self.controller.borrow_mut().cmd_timeout = Duration::from_millis(200);

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let controller_data = self.controller.clone();
        let gtk_widget = self.gtk_widget.clone();
        let hover_controller = EventControllerMotion::new();
        let gesture_controller = GestureClick::new();
        gesture_controller.set_button(0);
        let scroll_controller =
            EventControllerScroll::new(gtk4::EventControllerScrollFlags::BOTH_AXES);
        let drop_text_target = DropTarget::new(String::static_type(), gdk::DragAction::COPY);
        let drop_uri_target = DropTarget::new(String::static_type(), gdk::DragAction::COPY);
        let key_controller = EventControllerKey::new();

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

        let press_coords = Rc::new(Cell::new((0.0f64, 0.0f64)));

        // Support :active selector and onclick variant commands
        gesture_controller.connect_pressed(glib::clone!(
            #[weak]
            gtk_widget,
            #[strong]
            press_coords,
            move |_, _, x, y| {
                press_coords.set((x, y));
                gtk_widget.set_state_flags(gtk4::StateFlags::ACTIVE, false);
            }
        ));

        gesture_controller.connect_released(glib::clone!(
            #[weak]
            gtk_widget,
            #[strong]
            controller_data,
            #[strong]
            press_coords,
            move |gesture, _, x, y| {
                gtk_widget.unset_state_flags(gtk4::StateFlags::ACTIVE);

                // return if press is long
                let (px, py) = press_coords.get();
                let dist = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
                if dist > 8.0 {
                    return;
                }

                let controller = controller_data.borrow();
                let button = gesture.current_button();

                match button {
                    1 => {
                        run_command(controller.cmd_timeout, &controller.onclick_cmd, &[] as &[&str])
                    }
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

                if let Some(drag_value) = &controller.dragvalue {
                    match controller.dragtype {
                        DragEntryType::File => Some(gdk::ContentProvider::for_value(
                            &glib::Value::from(&[drag_value.as_str()][..]),
                        )),
                        DragEntryType::Text => {
                            Some(gdk::ContentProvider::for_value(&glib::Value::from(&drag_value)))
                        }
                    }
                } else {
                    None
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

        self.gtk_widget.add_controller(gesture_controller);
        self.gtk_widget.add_controller(hover_controller);
        self.gtk_widget.add_controller(scroll_controller);
        self.gtk_widget.add_controller(drop_text_target);
        self.gtk_widget.add_controller(drop_uri_target);
        self.gtk_widget.add_controller(drag_source);
        self.gtk_widget.add_controller(key_controller);

        let count = children.len();

        if count < 1 {
            bail!("event box must contain exactly one element");
        } else if count > 1 {
            bail!("event box must contain exactly one element, but got more");
        }

        let child = children.first().cloned().ok_or_else(|| anyhow!("missing child 0"))?;
        let child_widget = build_gtk_widget(&WidgetInput::Node(child), widget_registry)?;
        gtk_widget.append(&child_widget);

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "timeout" => {
                let new_timeout =
                    get_duration_prop(value, key).unwrap_or(Duration::from_millis(200));
                self.controller.borrow_mut().cmd_timeout = new_timeout;
            }
            "onscroll" => {
                // onscroll - event to execute when the user scrolls with the mouse over the widget. The placeholder `{}` used in the command will be replaced with either `up` or `down`.
                let controller_data = self.controller.clone();
                bind_property!(&value, &key, get_string_prop, [controller_data], |v: String| {
                    controller_data.borrow_mut().onscroll_cmd = v;
                });
            }
            "onhover" => {
                // onhover - event to execute when the user hovers over the widget
                let controller_data = self.controller.clone();
                bind_property!(&value, &key, get_string_prop, [controller_data], |v: String| {
                    controller_data.borrow_mut().onhover_cmd = v;
                });
            }
            "onhoverlost" => {
                // onhoverlost - event to execute when the user loses hover over the widget
                let controller_data = self.controller.clone();
                bind_property!(&value, &key, get_string_prop, [controller_data], |v: String| {
                    controller_data.borrow_mut().onhoverlost_cmd = v;
                });
            }
            "cursor" => {
                // cursor - Cursor to show while hovering
                let controller_data = self.controller.clone();
                bind_property!(&value, &key, get_string_prop, [controller_data], |v: String| {
                    controller_data.borrow_mut().hover_cursor = v;
                });
            }
            "ondropped" => {
                let controller_data = self.controller.clone();
                // ondropped - Command to execute when something is dropped on top of this element. The placeholder `{}` used in the command will be replaced with the uri to the dropped thing.
                bind_property!(&value, &key, get_string_prop, [controller_data], |v: String| {
                    controller_data.borrow_mut().ondropped_cmd = v;
                });
            }
            "drag_type" => {
                // dragtype - Type of value that should be dragged from this widget. Possible values: $dragtype
                let controller_data = self.controller.clone();
                bind_property!(&value, &key, get_string_prop, [controller_data], |v: String| {
                    if let Ok(dt) = parse_dragtype(&v) {
                        controller_data.borrow_mut().dragtype = dt;
                    }
                });
            }
            "dragvalue" => {
                let controller_data = self.controller.clone();
                // dragvalue - URI that will be provided when dragging from this widget
                bind_property!(&value, &key, get_string_prop, [controller_data], |v: String| {
                    controller_data.borrow_mut().dragvalue = Some(v);
                });
            }
            "onclick" => {
                let controller_data = self.controller.clone();
                // onclick - command to run when the widget is clicked
                bind_property!(&value, &key, get_string_prop, [controller_data], |v: String| {
                    controller_data.borrow_mut().onclick_cmd = v;
                });
            }
            "onmiddleclick" => {
                let controller_data = self.controller.clone();
                // onmiddleclick - command to run when the widget is middleclicked
                bind_property!(&value, &key, get_string_prop, [controller_data], |v: String| {
                    controller_data.borrow_mut().onmiddleclick_cmd = v;
                });
            }
            "onrightclick" => {
                let controller_data = self.controller.clone();
                // onrightclick - command to run when the widget is rightclicked
                bind_property!(&value, &key, get_string_prop, [controller_data], |v: String| {
                    controller_data.borrow_mut().onrightclick_cmd = v;
                });
            }
            "onkeypress" => {
                let controller_data = self.controller.clone();
                // onkeypress - command to run when a key is pressed
                bind_property!(&value, &key, get_string_prop, [controller_data], |v: String| {
                    controller_data.borrow_mut().onkeypress_cmd = Some(v);
                });
            }
            "onkeyrelease" => {
                // onkeyrelease - command to run when a key is released
                let controller_data = self.controller.clone();
                bind_property!(&value, &key, get_string_prop, [controller_data], |v: String| {
                    controller_data.borrow_mut().onkeyrelease_cmd = Some(v);
                });
            }
            "orientation" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |v: String| {
                    if let Ok(o) = parse_orientation(&v) {
                        gtk_widget.set_orientation(o);
                    }
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct FlowBoxWidget {
    gtk_widget: gtk4::FlowBox,
    onaccept_cmd: Rc<RefCell<String>>,
    cmd_timeout: Rc<RefCell<Duration>>,
}

impl EwwiiWidget for FlowBoxWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        children: &[WidgetNode],
        widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::FlowBox::new();
        *self.cmd_timeout.borrow_mut() = Duration::from_millis(200);
        self.gtk_widget.set_homogeneous(true);

        let onaccept_cmd = self.onaccept_cmd.clone();
        let cmd_timeout = self.cmd_timeout.clone();

        self.gtk_widget.connect_child_activated(glib::clone!(
            #[strong]
            onaccept_cmd,
            #[strong]
            cmd_timeout,
            move |_, flow_child: &gtk4::FlowBoxChild| {
                if let Some(child) = flow_child.child() {
                    let widget_name = child.widget_name();
                    run_command(*cmd_timeout.borrow(), &onaccept_cmd.borrow(), &[widget_name]);
                } else {
                    log::error!("Failed to get the child of FlowBoxChild.");
                }
            }
        ));

        for (key, value) in props {
            self.update_prop(key, value);
        }

        for (index, child) in children.iter().enumerate() {
            let child_widget =
                build_gtk_widget(&WidgetInput::BorrowedNode(child), widget_registry)?;
            self.gtk_widget.insert(&child_widget, index as i32);
        }

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "default_select" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_i32_prop, [gtk_widget], |dsv: i32| {
                    if let Some(child) = gtk_widget.child_at_index(dsv) {
                        gtk_widget.select_child(&child);
                        child.grab_focus();
                    } else {
                        log::error!("Failed to get child at index {} from FlowBox", dsv);
                    }
                });
            }
            "orientation" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |v: String| {
                    if let Ok(o) = parse_orientation(&v) {
                        gtk_widget.set_orientation(o);
                    }
                });
            }
            "space_evenly" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |v: bool| {
                    gtk_widget.set_homogeneous(v);
                });
            }
            "selection_model" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |v: String| {
                    if let Ok(selection_model) = parse_selection_model(&v) {
                        gtk_widget.set_selection_mode(selection_model);
                    }
                });
            }
            "timeout" => {
                let new_timeout =
                    get_duration_prop(value, key).unwrap_or(Duration::from_millis(200));
                *self.cmd_timeout.borrow_mut() = new_timeout;
            }
            "onaccept" => {
                let onaccept_cmd = self.onaccept_cmd.clone();
                bind_property!(&value, &key, get_string_prop, [onaccept_cmd], |v: String| {
                    *onaccept_cmd.borrow_mut() = v;
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct StackWidget {
    gtk_widget: gtk4::Stack,
}

impl EwwiiWidget for StackWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        children: &[WidgetNode],
        widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::Stack::new();

        for (key, value) in props {
            self.update_prop(key, value);
        }

        if children.is_empty() {
            return Err(anyhow!("stack must contain at least one element"));
        }

        let children = children
            .iter()
            .map(|child| build_gtk_widget(&WidgetInput::BorrowedNode(child), widget_registry));

        for (i, child) in children.enumerate() {
            let child = child?;
            self.gtk_widget.add_named(&child, Some(&i.to_string()));
        }

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "selected" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_i32_prop, [gtk_widget], |v: i32| {
                    gtk_widget.set_visible_child_name(&v.to_string());
                });
            }
            "transition" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |v: String| {
                    if let Ok(t) = parse_stack_transition(&v) {
                        gtk_widget.set_transition_type(t);
                    }
                });
            }
            "transition_duration" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_i32_prop, [gtk_widget], |v: i32| {
                    gtk_widget.set_transition_duration(v as u32);
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct CircularProgressWidget {
    gtk_widget: CircProg,
}

impl EwwiiWidget for CircularProgressWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = CircProg::new();

        for (key, value) in props {
            self.update_prop(key, value);
        }

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "value" => {
                let widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_f64_prop, [widget], |v: f64| {
                    widget.set_property("value", v.clamp(0.0, 100.0));
                });
            }
            "start_at" => {
                let widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_f64_prop, [widget], |v: f64| {
                    widget.set_property("start-at", v.clamp(0.0, 100.0));
                });
            }
            "thickness" => {
                let widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_f64_prop, [widget], |v: f64| {
                    widget.set_property("thickness", v);
                });
            }
            "clockwise" => {
                let widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [widget], |v: bool| {
                    widget.set_property("clockwise", v);
                });
            }
            "fg_color" => {
                let widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [widget], |v: String| {
                    if let Ok(rgba) = gdk::RGBA::parse(v) {
                        widget.set_property("fg-color", rgba);
                    }
                });
            }
            "bg_color" => {
                let widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [widget], |v: String| {
                    if let Ok(rgba) = gdk::RGBA::parse(v) {
                        widget.set_property("bg-color", rgba);
                    }
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct GraphWidget {
    gtk_widget: Graph,
    min_val: Rc<RefCell<f64>>,
    max_val: Rc<RefCell<f64>>,
}

impl EwwiiWidget for GraphWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = Graph::new();

        for (key, value) in props {
            self.update_prop(key, value);
        }

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        let apply_min_max = {
            let widget = self.gtk_widget.clone();
            let min_val = self.min_val.clone();
            let max_val = self.max_val.clone();
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

        match key {
            "value" => {
                let widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_f64_prop, [widget], |value: f64| {
                    if value.is_nan() || value.is_infinite() {
                        log::error!("Graph's value should never be NaN or infinite");
                        return;
                    }
                    widget.set_property("value", value);
                });
            }
            "time_range" => {
                let widget = self.gtk_widget.clone();
                if let Ok(time_range) = get_duration_prop(value, key) {
                    let millis = time_range.as_millis();
                    let millis_u32 = match u32::try_from(millis) {
                        Ok(m) => m,
                        Err(_) => {
                            log::error!(
                                "Graph's time_range ({}ms) exceeds maximum representable ({}ms)",
                                millis,
                                u32::MAX
                            );
                            200
                        }
                    };

                    widget.set_property("time-range", millis_u32);
                }
            }
            "min" => {
                let min_val = self.min_val.clone();
                bind_property!(&value, &key, get_f64_prop, [min_val, apply_min_max], |v: f64| {
                    *min_val.borrow_mut() = v;
                    apply_min_max();
                });
            }
            "max" => {
                let max_val = self.max_val.clone();
                bind_property!(&value, &key, get_f64_prop, [max_val, apply_min_max], |v: f64| {
                    *max_val.borrow_mut() = v;
                    apply_min_max();
                });
            }
            "dynamic" => {
                let widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [widget], |dynamic: bool| {
                    widget.set_property("dynamic", dynamic);
                });
            }
            "type" => {
                let widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [widget], |render_type: String| {
                    match parse_graph_render_type(render_type.as_str()) {
                        Ok(t) => widget.set_property("type", t),
                        Err(e) => {
                            log::error!("Failed to parse graph type property: {}", e);
                            return;
                        }
                    };
                });
            }
            "thickness" => {
                let widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_f64_prop, [widget], |thickness: f64| {
                    if !matches!(widget.property("type"), RenderType::Line | RenderType::StepLine) {
                        log::error!("Property thickness can only be used with line graphs");
                        return;
                    }

                    widget.set_property("thickness", thickness);
                });
            }
            "line_style" => {
                let widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [widget], |line_style: String| {
                    if !matches!(widget.property("type"), RenderType::Line | RenderType::StepLine) {
                        log::error!("Property line-style can only be used with line graphs");
                        return;
                    }

                    match parse_graph_line_style(line_style.as_str()) {
                        Ok(ls) => widget.set_property("line-style", ls),
                        Err(e) => {
                            log::error!("Failed to parse graph line-style property: {}", e);
                            return;
                        }
                    };
                });
            }
            "flip_x" => {
                let widget = self.gtk_widget.clone();
                // flip-x - whether the x axis should go from high to low
                bind_property!(&value, &key, get_bool_prop, [widget], |flip_x: bool| {
                    widget.set_property("flip-x", flip_x);
                });
            }
            "flip_y" => {
                let widget = self.gtk_widget.clone();
                // flip-y - whether the y axis should go from high to low
                bind_property!(&value, &key, get_bool_prop, [widget], |flip_y: bool| {
                    widget.set_property("flip-y", flip_y);
                });
            }
            "vertical" => {
                let widget = self.gtk_widget.clone();
                // vertical - if set to true, the x and y axes will be exchanged
                bind_property!(&value, &key, get_bool_prop, [widget], |vertical: bool| {
                    widget.set_property("vertical", vertical);
                });
            }
            "animate" => {
                let widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [widget], |animate: bool| {
                    widget.set_property("animate", animate);
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct ProgressWidget {
    gtk_widget: gtk4::ProgressBar,
}

impl EwwiiWidget for ProgressWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::ProgressBar::new();

        for (key, value) in props {
            self.update_prop(key, value);
        }

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "orientation" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |v: String| {
                    if let Ok(o) = parse_orientation(&v) {
                        gtk_widget.set_orientation(o)
                    }
                });
            }
            "flipped" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |flipped: bool| {
                    gtk_widget.set_inverted(flipped)
                });
            }
            "value" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_f64_prop, [gtk_widget], |bar_value: f64| {
                    gtk_widget.set_fraction(bar_value / 100f64);
                });
            }
            "text" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |bar_text: String| {
                    gtk_widget.set_text(Some(&bar_text));
                });
            }
            "show_text" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |show_text: bool| {
                    gtk_widget.set_show_text(show_text);
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct ImageWidget {
    gtk_widget: EwwiiImage,
}

impl EwwiiWidget for ImageWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = EwwiiImage::default();

        for (key, value) in props {
            self.update_prop(key, value);
        }

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "path" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |v: String| {
                    gtk_widget.set_path(v);
                });
            }
            "image_width" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_i32_prop, [gtk_widget], |v: i32| {
                    gtk_widget.set_image_width(v);
                });
            }
            "image_height" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_i32_prop, [gtk_widget], |v: i32| {
                    gtk_widget.set_image_height(v);
                });
            }
            "preserve_aspect_ratio" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |v: bool| {
                    gtk_widget.set_preserve_aspect_ratio(v);
                });
            }
            "fill_svg" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |v: String| {
                    gtk_widget.set_fill_svg(v);
                });
            }
            "content_fit" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |v: String| {
                    if let Ok(content_fit) = parse_content_fit(&v) {
                        gtk_widget.set_content_fit(content_fit);
                    };
                });
            }
            "can_shrink" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |v: bool| {
                    gtk_widget.set_can_shrink(v);
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct ButtonWidget {
    gtk_widget: gtk4::Button,
    onclick_cmd: Rc<RefCell<String>>,
    onmiddleclick_cmd: Rc<RefCell<String>>,
    onrightclick_cmd: Rc<RefCell<String>>,
    cmd_timeout: Rc<RefCell<Duration>>,
}

impl EwwiiWidget for ButtonWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::Button::new();
        *self.cmd_timeout.borrow_mut() = Duration::from_millis(200);

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let key_controller = EventControllerKey::new();
        let gesture_controller = GestureClick::new();

        gesture_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
        gesture_controller.set_button(0);

        self.gtk_widget.connect_clicked(move |button| {
            button.emit_activate();
        });

        let press_coords = Rc::new(Cell::new((0.0f64, 0.0f64)));

        gesture_controller.connect_pressed(glib::clone!(
            #[strong]
            press_coords,
            move |_, _, x, y| {
                press_coords.set((x, y));
            }
        ));

        let cmd_timeout = self.cmd_timeout.clone();
        let onclick_cmd = self.onclick_cmd.clone();
        let onmiddleclick_cmd = self.onmiddleclick_cmd.clone();
        let onrightclick_cmd = self.onrightclick_cmd.clone();

        let gtk_widget = self.gtk_widget.clone();
        gesture_controller.connect_released(glib::clone!(
            #[weak]
            gtk_widget,
            #[strong]
            cmd_timeout,
            #[strong]
            onclick_cmd,
            #[strong]
            onmiddleclick_cmd,
            #[strong]
            onrightclick_cmd,
            #[strong]
            press_coords,
            move |gesture, _, x, y| {
                gtk_widget.unset_state_flags(gtk4::StateFlags::ACTIVE);

                // return if press is long
                let (px, py) = press_coords.get();
                let dist = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
                if dist > 8.0 {
                    return;
                }

                let button = gesture.current_button();

                match button {
                    1 => run_command(*cmd_timeout.borrow(), &onclick_cmd.borrow(), &[] as &[&str]),
                    2 => run_command(
                        *cmd_timeout.borrow(),
                        &onmiddleclick_cmd.borrow(),
                        &[] as &[&str],
                    ),
                    3 => run_command(
                        *cmd_timeout.borrow(),
                        &onrightclick_cmd.borrow(),
                        &[] as &[&str],
                    ),
                    _ => {}
                }
            }
        ));

        key_controller.connect_key_released(glib::clone!(
            #[strong]
            cmd_timeout,
            #[strong]
            onclick_cmd,
            move |_, _, code, _| {
                match code {
                    // return
                    36 => run_command(*cmd_timeout.borrow(), &onclick_cmd.borrow(), &[] as &[&str]),
                    // space
                    65 => run_command(*cmd_timeout.borrow(), &onclick_cmd.borrow(), &[] as &[&str]),
                    _ => {}
                }
            }
        ));

        self.gtk_widget.add_controller(key_controller);
        self.gtk_widget.add_controller(gesture_controller);

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "timeout" => {
                let new_timeout =
                    get_duration_prop(value, key).unwrap_or(Duration::from_millis(200));
                *self.cmd_timeout.borrow_mut() = new_timeout;
            }
            "onclick" => {
                let onclick_cmd = self.onclick_cmd.clone();
                bind_property!(&value, &key, get_string_prop, [onclick_cmd], |v: String| {
                    *onclick_cmd.borrow_mut() = v;
                });
            }
            "onmiddleclick" => {
                let onmiddleclick_cmd = self.onmiddleclick_cmd.clone();
                bind_property!(&value, &key, get_string_prop, [onmiddleclick_cmd], |v: String| {
                    *onmiddleclick_cmd.borrow_mut() = v;
                });
            }
            "onrightclick" => {
                let onrightclick_cmd = self.onrightclick_cmd.clone();
                bind_property!(&value, &key, get_string_prop, [onrightclick_cmd], |v: String| {
                    *onrightclick_cmd.borrow_mut() = v;
                });
            }
            "label" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |lbl: String| {
                    gtk_widget.set_label(&lbl);
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct LabelWidget {
    gtk_widget: EwwiiLabel,
}

impl EwwiiWidget for LabelWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = EwwiiLabel::new();

        for (key, value) in props {
            self.update_prop(key, value);
        }

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "text" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |value: String| {
                    gtk_widget.set_text(Some(value))
                });
            }
            "markup" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |value: String| {
                    gtk_widget.set_markup(Some(value));
                });
            }
            "truncate" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |value: bool| {
                    gtk_widget.set_ellipsize(value);
                });
            }
            "limit_width" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_i32_prop, [gtk_widget], |value: i32| {
                    gtk_widget.set_max_chars(value);
                });
            }
            "truncate_left" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |value: bool| {
                    gtk_widget.set_ellipsize_start(value);
                });
            }
            "unescape" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |value: bool| {
                    gtk_widget.set_unescape(value);
                });
            }
            "unindent" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |value: bool| {
                    gtk_widget.set_unindent(value);
                });
            }

            // independant props
            "wrap" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |wrap: bool| {
                    gtk_widget.set_wrap(wrap);
                });
            }
            "gravity" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |grav: String| {
                    if let Ok(v) = parse_gravity(&grav) {
                        gtk_widget.pango_context().set_base_gravity(v);
                    }
                });
            }
            "xalign" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_f64_prop, [gtk_widget], |v: f64| {
                    gtk_widget.set_xalign(v as f32);
                });
            }
            "yalign" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_f64_prop, [gtk_widget], |v: f64| {
                    gtk_widget.set_yalign(v as f32);
                });
            }
            "justify" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |justify: String| {
                    if let Ok(v) = parse_justification(&justify) {
                        gtk_widget.set_justify(v);
                    }
                });
            }
            "wrap_mode" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |wrap: String| {
                    if let Ok(v) = parse_wrap_mode(&wrap) {
                        gtk_widget.set_wrap_mode(v);
                    }
                });
            }
            "lines" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_i32_prop, [gtk_widget], |lines: i32| {
                    gtk_widget.set_lines(lines);
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct InputWidget {
    gtk_widget: gtk4::Entry,
    timeout: Rc<RefCell<Duration>>,
    onchange_cmd: Rc<RefCell<String>>,
    onaccept_cmd: Rc<RefCell<String>>,
}

impl EwwiiWidget for InputWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::Entry::new();
        *self.timeout.borrow_mut() = Duration::from_millis(200);

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let timeout = self.timeout.clone();
        let onchange_cmd = self.onchange_cmd.clone();
        let onaccept_cmd = self.onaccept_cmd.clone();

        self.gtk_widget.connect_changed(glib::clone!(
            #[strong]
            timeout,
            #[strong]
            onchange_cmd,
            move |widget| {
                run_command(
                    *timeout.borrow(),
                    &onchange_cmd.borrow(),
                    &[widget.text().to_string()],
                );
            }
        ));

        self.gtk_widget.connect_activate(glib::clone!(
            #[strong]
            timeout,
            #[strong]
            onaccept_cmd,
            move |widget| {
                run_command(
                    *timeout.borrow(),
                    &onaccept_cmd.borrow(),
                    &[widget.text().to_string()],
                );
            }
        ));

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "value" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |value: String| {
                    gtk_widget.set_text(&value);
                });
            }
            "placeholder" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |value: String| {
                    gtk_widget.set_placeholder_text(Some(&value));
                });
            }
            "password" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |password: bool| {
                    gtk_widget.set_visibility(!password);
                });
            }
            "timeout" => {
                let new_timeout =
                    get_duration_prop(value, key).unwrap_or(Duration::from_millis(200));
                *self.timeout.borrow_mut() = new_timeout;
            }
            "onchange" => {
                let onchange_cmd = self.onchange_cmd.clone();
                bind_property!(&value, &key, get_string_prop, [onchange_cmd], |v: String| {
                    *onchange_cmd.borrow_mut() = v;
                });
            }
            "onaccept" => {
                let onaccept_cmd = self.onaccept_cmd.clone();
                bind_property!(&value, &key, get_string_prop, [onaccept_cmd], |v: String| {
                    *onaccept_cmd.borrow_mut() = v;
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct CalendarWidget {
    gtk_widget: gtk4::Calendar,
    timeout: Rc<RefCell<Duration>>,
    onclick_cmd: Rc<RefCell<String>>,
}

impl EwwiiWidget for CalendarWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::Calendar::new();
        *self.timeout.borrow_mut() = Duration::from_millis(200);

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let timeout = self.timeout.clone();
        let onclick_cmd = self.onclick_cmd.clone();

        self.gtk_widget.connect_day_selected(glib::clone!(
            #[strong]
            onclick_cmd,
            #[strong]
            timeout,
            move |w| {
                run_command(
                    *timeout.borrow(),
                    &onclick_cmd.borrow(),
                    &[w.day(), w.month(), w.year()],
                );
            }
        ));

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "day" => {
                let gtk_widget = self.gtk_widget.clone();
                // day - the selected day
                bind_property!(&value, &key, get_f64_prop, [gtk_widget], |day: f64| {
                    if !(1f64..=31f64).contains(&day) {
                        log::warn!("Calendar day is not a number between 1 and 31");
                    } else {
                        gtk_widget.set_day(day as i32);
                    }
                });
            }
            "month" => {
                let gtk_widget = self.gtk_widget.clone();
                // month - the selected month
                bind_property!(&value, &key, get_f64_prop, [gtk_widget], |month: f64| {
                    if !(1f64..=12f64).contains(&month) {
                        log::warn!("Calendar month is not a number between 1 and 12");
                    } else {
                        gtk_widget.set_month(month as i32 - 1);
                    }
                });
            }
            "year" => {
                let gtk_widget = self.gtk_widget.clone();
                // year - the selected year
                bind_property!(&value, &key, get_f64_prop, [gtk_widget], |year: f64| {
                    gtk_widget.set_year(year as i32);
                });
            }
            "show_heading" => {
                let gtk_widget = self.gtk_widget.clone();
                // show-heading - show heading line
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |show_heading: bool| {
                    gtk_widget.set_show_heading(show_heading);
                });
            }
            "show_day_names" => {
                let gtk_widget = self.gtk_widget.clone();
                // show-day-names - show names of days
                bind_property!(
                    &value,
                    &key,
                    get_bool_prop,
                    [gtk_widget],
                    |show_day_names: bool| {
                        gtk_widget.set_show_day_names(show_day_names);
                    }
                );
            }
            "show_week_numbers" => {
                let gtk_widget = self.gtk_widget.clone();
                // show-week-numbers - show week numbers
                bind_property!(
                    &value,
                    &key,
                    get_bool_prop,
                    [gtk_widget],
                    |show_week_numbers: bool| {
                        gtk_widget.set_show_week_numbers(show_week_numbers);
                    }
                );
            }
            "timeout" => {
                let new_timeout =
                    get_duration_prop(value, key).unwrap_or(Duration::from_millis(200));
                *self.timeout.borrow_mut() = new_timeout;
            }
            "onclick" => {
                let onclick_cmd = self.onclick_cmd.clone();
                bind_property!(&value, &key, get_string_prop, [onclick_cmd], |v: String| {
                    *onclick_cmd.borrow_mut() = v;
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
#[allow(deprecated)]
struct ComboBoxTextWidget {
    gtk_widget: gtk4::ComboBoxText,
    timeout: Rc<RefCell<Duration>>,
    onchange_cmd: Rc<RefCell<String>>,
}

#[allow(deprecated)]
impl EwwiiWidget for ComboBoxTextWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::ComboBoxText::new();
        *self.timeout.borrow_mut() = Duration::from_millis(200);

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let onchange_cmd = self.onchange_cmd.clone();
        let timeout = self.timeout.clone();
        self.gtk_widget.connect_changed(glib::clone!(
            #[strong]
            onchange_cmd,
            #[strong]
            timeout,
            move |combo_box| {
                run_command(
                    *timeout.borrow(),
                    &onchange_cmd.borrow(),
                    &[combo_box.active_text().unwrap_or_else(|| "".into())],
                );
            }
        ));

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "items" => {
                let gtk_widget = self.gtk_widget.clone();
                if let Ok(items) = get_vec_string_prop(value, key) {
                    let current_items: Rc<RefCell<Vec<String>>> =
                        Rc::new(RefCell::new(items.iter().map(|p| p.initial_value()).collect()));

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
            }
            "timeout" => {
                let new_timeout =
                    get_duration_prop(value, key).unwrap_or(Duration::from_millis(200));
                *self.timeout.borrow_mut() = new_timeout;
            }
            "onchange" => {
                let onchange_cmd = self.onchange_cmd.clone();
                bind_property!(&value, &key, get_string_prop, [onchange_cmd], |v: String| {
                    *onchange_cmd.borrow_mut() = v;
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct ExpanderWidget {
    gtk_widget: gtk4::Expander,
}

impl EwwiiWidget for ExpanderWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        children: &[WidgetNode],
        widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::Expander::new(None);

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let count = children.len();
        if count < 1 {
            bail!("expander must contain exactly one element");
        } else if count > 1 {
            bail!("expander must contain exactly one element, but got more");
        }

        let child = children.first().cloned().ok_or_else(|| anyhow!("missing child 0"))?;
        let child_widget = build_gtk_widget(&WidgetInput::Node(child), widget_registry)?;
        self.gtk_widget.set_child(Some(&child_widget));

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "name" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |name: String| {
                    gtk_widget.set_label(Some(&name));
                });
            }
            "expanded" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |expanded: bool| {
                    gtk_widget.set_expanded(expanded);
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct RevealerWidget {
    gtk_widget: gtk4::Revealer,
}

impl EwwiiWidget for RevealerWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        children: &[WidgetNode],
        widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::Revealer::new();

        for (key, value) in props {
            self.update_prop(key, value);
        }

        match children.len() {
            0 => { /* maybe warn? */ }
            1 => {
                let child_widget =
                    build_gtk_widget(&WidgetInput::Node(children[0].clone()), widget_registry)?;
                self.gtk_widget.set_child(Some(&child_widget));
            }
            n => {
                return Err(anyhow!(
                    "A revealer must only have a maximum of 1 child but got: {}",
                    n
                ));
            }
        }

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "transition" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(
                    &value,
                    &key,
                    get_string_prop,
                    [gtk_widget],
                    |transition: String| {
                        if let Ok(t) = parse_revealer_transition(&transition) {
                            gtk_widget.set_transition_type(t);
                        }
                    }
                );
            }
            "reveal" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |reveal: bool| {
                    gtk_widget.set_reveal_child(reveal);
                });
            }
            "duration" => {
                let duration = get_duration_prop(value, key).unwrap_or(Duration::from_millis(500));
                self.gtk_widget.set_transition_duration(duration.as_millis() as u32);
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct CheckboxWidget {
    gtk_widget: gtk4::CheckButton,
    timeout: Rc<RefCell<Duration>>,
    onchecked_cmd: Rc<RefCell<String>>,
    onunchecked_cmd: Rc<RefCell<String>>,
}

impl EwwiiWidget for CheckboxWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::CheckButton::new();
        *self.timeout.borrow_mut() = Duration::from_millis(200);

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let timeout = self.timeout.clone();
        let onchecked_cmd = self.onchecked_cmd.clone();
        let onunchecked_cmd = self.onunchecked_cmd.clone();

        self.gtk_widget.connect_toggled(glib::clone!(
            #[strong]
            onchecked_cmd,
            #[strong]
            onunchecked_cmd,
            #[strong]
            timeout,
            move |widget| {
                let oncheck = &onchecked_cmd.borrow();
                let onuncheck = &onunchecked_cmd.borrow();

                run_command(
                    *timeout.borrow(),
                    if widget.is_active() { oncheck } else { onuncheck },
                    &[] as &[&str],
                );
            }
        ));

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "checked" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |checked: bool| {
                    gtk_widget.set_active(checked);
                });
            }
            "timeout" => {
                let new_timeout =
                    get_duration_prop(value, key).unwrap_or(Duration::from_millis(200));
                *self.timeout.borrow_mut() = new_timeout;
            }
            "onchecked" => {
                let onchecked_cmd = self.onchecked_cmd.clone();
                bind_property!(&value, &key, get_string_prop, [onchecked_cmd], |v: String| {
                    *onchecked_cmd.borrow_mut() = v;
                });
            }
            "onunchecked" => {
                let onunchecked_cmd = self.onunchecked_cmd.clone();
                bind_property!(&value, &key, get_string_prop, [onunchecked_cmd], |v: String| {
                    *onunchecked_cmd.borrow_mut() = v;
                });
            }

            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
#[allow(deprecated)]
struct ColorButtonWidget {
    gtk_widget: gtk4::ColorButton,
    timeout: Rc<RefCell<Duration>>,
    onchange_cmd: Rc<RefCell<String>>,
}

#[allow(deprecated)]
impl EwwiiWidget for ColorButtonWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::ColorButton::builder().build();
        *self.timeout.borrow_mut() = Duration::from_millis(200);

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let timeout = self.timeout.clone();
        let onchange_cmd = self.onchange_cmd.clone();
        self.gtk_widget.connect_color_set(glib::clone!(
            #[strong]
            onchange_cmd,
            #[strong]
            timeout,
            move |widget| {
                run_command(*timeout.borrow(), &onchange_cmd.borrow(), &[widget.rgba()]);
            }
        ));

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "use_alpha" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |use_alpha: bool| {
                    gtk_widget.set_use_alpha(use_alpha);
                });
            }
            "timeout" => {
                let new_timeout =
                    get_duration_prop(value, key).unwrap_or(Duration::from_millis(200));
                *self.timeout.borrow_mut() = new_timeout;
            }
            "onchange" => {
                let onchange_cmd = self.onchange_cmd.clone();
                bind_property!(&value, &key, get_string_prop, [onchange_cmd], |v: String| {
                    *onchange_cmd.borrow_mut() = v;
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
#[allow(deprecated)]
struct ColorChooserEwwiiWidget {
    gtk_widget: gtk4::ColorChooserWidget,
    timeout: Rc<RefCell<Duration>>,
    onchange_cmd: Rc<RefCell<String>>,
}

#[allow(deprecated)]
impl EwwiiWidget for ColorChooserEwwiiWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::ColorChooserWidget::new();
        *self.timeout.borrow_mut() = Duration::from_millis(200);

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let timeout = self.timeout.clone();
        let onchange_cmd = self.onchange_cmd.clone();
        self.gtk_widget.connect_color_activated(glib::clone!(
            #[strong]
            onchange_cmd,
            move |_a, color| {
                run_command(*timeout.borrow(), &onchange_cmd.borrow(), &[*color]);
            }
        ));

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "use_alpha" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |use_alpha: bool| {
                    gtk_widget.set_use_alpha(use_alpha);
                });
            }
            "timeout" => {
                let new_timeout =
                    get_duration_prop(value, key).unwrap_or(Duration::from_millis(200));
                *self.timeout.borrow_mut() = new_timeout;
            }
            "onchange" => {
                let onchange_cmd = self.onchange_cmd.clone();
                bind_property!(&value, &key, get_string_prop, [onchange_cmd], |v: String| {
                    *onchange_cmd.borrow_mut() = v;
                });
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct RangeCtrlData {
    onchange_cmd: String,
    cmd_timeout: Duration,
    is_being_dragged: bool,
    last_set_value: Option<f64>,
}

#[derive(Default)]
struct ScaleWidget {
    gtk_widget: gtk4::Scale,
    range_dat: Rc<RefCell<RangeCtrlData>>,
}

impl EwwiiWidget for ScaleWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        _children: &[WidgetNode],
        _widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::Scale::new(
            gtk4::Orientation::Horizontal,
            Some(&gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 10.0, 0.0)),
        );
        self.range_dat.borrow_mut().cmd_timeout = Duration::from_millis(200);

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let scale_dat = self.range_dat.clone();

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

        self.gtk_widget.add_controller(legacy_controller);
        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "orientation" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |v: String| {
                    if let Ok(o) = parse_orientation(&v) {
                        gtk_widget.set_orientation(o)
                    }
                });
            }
            "flipped" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |v: bool| {
                    gtk_widget.set_inverted(v);
                });
            }
            "marks" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |marks: String| {
                    gtk_widget.clear_marks();
                    for mark in marks.split(',') {
                        if let Ok(val) = mark.trim().parse() {
                            gtk_widget.add_mark(val, gtk4::PositionType::Bottom, None);
                        }
                    }
                });
            }
            "draw_value" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |v: bool| {
                    gtk_widget.set_draw_value(v);
                });
            }
            "value_pos" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_string_prop, [gtk_widget], |value_pos: String| {
                    if let Ok(pos) = parse_position_type(&value_pos) {
                        gtk_widget.set_value_pos(pos);
                    }
                });
            }
            "round_digits" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_i32_prop, [gtk_widget], |v: i32| {
                    gtk_widget.set_round_digits(v);
                });
            }
            _ => {
                match resolve_range_attrs(
                    self.gtk_widget.upcast_ref::<gtk4::Range>(),
                    key,
                    value,
                    self.range_dat.clone(),
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Failed to apply range attributes: {}", e);
                        return;
                    }
                }
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

#[derive(Default)]
struct ScrolledWindowWidget {
    gtk_widget: gtk4::ScrolledWindow,
}

impl EwwiiWidget for ScrolledWindowWidget {
    fn widget(&self) -> &gtk4::Widget {
        self.gtk_widget.upcast_ref()
    }

    fn build(
        &mut self,
        props: &PropertyMap,
        children: &[WidgetNode],
        widget_registry: &mut WidgetRegistry,
    ) -> Result<gtk4::Widget> {
        self.gtk_widget = gtk4::ScrolledWindow::new();

        for (key, value) in props {
            self.update_prop(key, value);
        }

        let count = children.len();

        if count < 1 {
            bail!("scrolled window must contain exactly one element");
        } else if count > 1 {
            bail!("scrolled window contain exactly one element, but got more");
        }

        let child = children.first().cloned().ok_or_else(|| anyhow!("missing child 0"))?;
        let child_widget = build_gtk_widget(&WidgetInput::Node(child), widget_registry)?;
        self.gtk_widget.set_child(Some(&child_widget));

        Ok(self.gtk_widget.clone().upcast())
    }

    fn update_prop(&mut self, key: &str, value: &Property) {
        match key {
            "hscroll" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |v: bool| {
                    gtk_widget.set_hscrollbar_policy(if v {
                        gtk4::PolicyType::Automatic
                    } else {
                        gtk4::PolicyType::Never
                    });
                });
            }
            "vscroll" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(&value, &key, get_bool_prop, [gtk_widget], |v: bool| {
                    gtk_widget.set_vscrollbar_policy(if v {
                        gtk4::PolicyType::Automatic
                    } else {
                        gtk4::PolicyType::Never
                    });
                });
            }
            "propagate_natural_height" => {
                let gtk_widget = self.gtk_widget.clone();
                bind_property!(
                    &value,
                    &key,
                    get_bool_prop,
                    [gtk_widget],
                    |natural_height: bool| {
                        gtk_widget.set_propagate_natural_height(natural_height);
                    }
                );
            }
            _ => {
                resolve_widget_attrs(&self.gtk_widget.clone().upcast::<gtk4::Widget>(), key, value)
            }
        }
    }
}

// === Widget Registration === //

pub(super) fn build_gtk_box(
    props: &PropertyMap,
    children: &[WidgetNode],
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Box> {
    let mut widget = BoxWidget::default();
    let gtk_widget = widget.build(props, children, widget_registry)?;

    let id = hash_props_and_type(props, "Box");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<gtk4::Box>().expect("Box was expected to be a box."))
}

pub(super) fn build_gtk_overlay(
    props: &PropertyMap,
    children: &[WidgetNode],
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Overlay> {
    let mut widget = OverlayWidget::default();
    let gtk_widget = widget.build(props, children, widget_registry)?;

    let id = hash_props_and_type(props, "Overlay");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<gtk4::Overlay>().expect("Overlay was expected to be an overlay."))
}

pub(super) fn build_tooltip(
    props: &PropertyMap,
    children: &[WidgetNode],
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Box> {
    let mut widget = TooltipWidget::default();
    let gtk_widget = widget.build(props, children, widget_registry)?;

    let id = hash_props_and_type(props, "Tooltip");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<gtk4::Box>().expect("Tooltip was expected to be a Box."))
}

pub(super) fn build_event_box(
    props: &PropertyMap,
    children: &[WidgetNode],
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Box> {
    let mut widget = EventBoxWidget::default();
    let gtk_widget = widget.build(props, children, widget_registry)?;

    let id = hash_props_and_type(props, "EventBox");
    widget_registry.widgets.insert(id, Box::new(widget));
    Ok(gtk_widget.downcast::<gtk4::Box>().expect("Eventbox was expected to be a Box."))
}

pub(crate) fn build_gtk_flowbox(
    props: &PropertyMap,
    children: &[WidgetNode],
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::FlowBox> {
    let mut widget = FlowBoxWidget::default();
    let gtk_widget = widget.build(props, children, widget_registry)?;

    let id = hash_props_and_type(props, "FlowBox");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<gtk4::FlowBox>().expect("FlowBox was expected to be a FlowBox."))
}

pub(super) fn build_gtk_stack(
    props: &PropertyMap,
    children: &[WidgetNode],
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Stack> {
    let mut widget = StackWidget::default();
    let gtk_widget = widget.build(props, children, widget_registry)?;

    let id = hash_props_and_type(props, "Stack");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<gtk4::Stack>().expect("Stack was expected to be a stack."))
}

pub(super) fn build_circular_progress_bar(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<CircProg> {
    let mut widget = CircularProgressWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "CircularProgress");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget
        .downcast::<CircProg>()
        .expect("CircularProgress was expected to be a CircularProgress."))
}

pub(super) fn build_graph(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<Graph> {
    let mut widget = GraphWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "Graph");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<Graph>().expect("Graph was expected to be a Graph"))
}

pub(super) fn build_gtk_progress(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ProgressBar> {
    let mut widget = ProgressWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "Progress");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget
        .downcast::<gtk4::ProgressBar>()
        .expect("ProgressBar was expected to be a ProgressBar."))
}

pub(super) fn build_image(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<EwwiiImage> {
    let mut widget = ImageWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "Image");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<EwwiiImage>().expect("EwwiiImage was expected to be EwwiiImage"))
}

pub(super) fn build_gtk_button(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Button> {
    let mut widget = ButtonWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "Button");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<gtk4::Button>().expect("Button was expected to be a Button"))
}

pub(super) fn build_gtk_label(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<EwwiiLabel> {
    let mut widget = LabelWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "Label");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<EwwiiLabel>().expect("EwwiiLabel was expected to be EwwiiLabel"))
}

pub(super) fn build_gtk_input(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Entry> {
    let mut widget = InputWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "Input");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<gtk4::Entry>().expect("Entry was expected to be an Entry"))
}

pub(super) fn build_gtk_calendar(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Calendar> {
    let mut widget = CalendarWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "Calendar");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<gtk4::Calendar>().expect("Calendar was expected to be a Calendar"))
}

#[allow(deprecated)]
pub(super) fn build_gtk_combo_box_text(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ComboBoxText> {
    let mut widget = ComboBoxTextWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "ComboBoxText");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget
        .downcast::<gtk4::ComboBoxText>()
        .expect("ComboBoxText was expected to be a ComboBoxText"))
}

// Doesn't require `EwwiiWidget` trait
// because its a special non-widget thingy.
pub(super) fn build_gtk_ui_file(props: &PropertyMap) -> Result<gtk4::Widget> {
    const PATH_KEY: &str = "file";
    const ID_KEY: &str = "id";

    let path_prop = retreive_prop(props, PATH_KEY)?;
    let main_id_prop = retreive_prop(props, ID_KEY)?;

    let path = unwrap_static(PATH_KEY, get_string_prop(path_prop, PATH_KEY)?);
    let main_id = unwrap_static(ID_KEY, get_string_prop(main_id_prop, ID_KEY)?);

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
    props: &PropertyMap,
    children: &[WidgetNode],
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Expander> {
    let mut widget = ExpanderWidget::default();
    let gtk_widget = widget.build(props, children, widget_registry)?;

    let id = hash_props_and_type(props, "Expander");
    widget_registry.widgets.insert(id, Box::new(widget));
    Ok(gtk_widget.downcast::<gtk4::Expander>().expect("Expander was expected to be an Expander"))
}

pub(super) fn build_gtk_revealer(
    props: &PropertyMap,
    children: &[WidgetNode],
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Revealer> {
    let mut widget = RevealerWidget::default();
    let gtk_widget = widget.build(props, children, widget_registry)?;

    let id = hash_props_and_type(props, "Revealer");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<gtk4::Revealer>().expect("Revealer was expected to be a Revealer"))
}

pub(super) fn build_gtk_checkbox(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::CheckButton> {
    let mut widget = CheckboxWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "Checkbox");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget
        .downcast::<gtk4::CheckButton>()
        .expect("CheckButton was expected to be a CheckButton"))
}

#[allow(deprecated)]
pub(super) fn build_gtk_color_button(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ColorButton> {
    let mut widget = ColorButtonWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "ColorButton");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget
        .downcast::<gtk4::ColorButton>()
        .expect("ColorButton was expected to be a ColorButton"))
}

#[allow(deprecated)]
pub(super) fn build_gtk_color_chooser(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ColorChooserWidget> {
    let mut widget = ColorChooserEwwiiWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "ColorChooser");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget
        .downcast::<gtk4::ColorChooserWidget>()
        .expect("ColorChooserWidget was expected to be a ColorChooserWidget"))
}

pub(super) fn build_gtk_scale(
    props: &PropertyMap,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Scale> {
    let mut widget = ScaleWidget::default();
    let gtk_widget = widget.build(props, &[], widget_registry)?;

    let id = hash_props_and_type(props, "Scale");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget.downcast::<gtk4::Scale>().expect("Scale was expected to be a Scale"))
}

pub(super) fn build_gtk_scrolledwindow(
    props: &PropertyMap,
    children: &[WidgetNode],
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::ScrolledWindow> {
    let mut widget = ScrolledWindowWidget::default();
    let gtk_widget = widget.build(props, children, widget_registry)?;

    let id = hash_props_and_type(props, "ScrolledWindow");
    widget_registry.widgets.insert(id, Box::new(widget));

    Ok(gtk_widget
        .downcast::<gtk4::ScrolledWindow>()
        .expect("ScrolledWindow was expected to be a ScrolledWindow"))
}

// commented out because i dont think its needed...
// /// Deprecated attributes from top of widget hierarchy
// static DEPRECATED_ATTRS: Lazy<HashSet<&str>> =
//     Lazy::new(|| ["timeout", "onscroll", "onhover", "cursor"].iter().cloned().collect());

/// Code that applies css/scss to widgets.
fn resolve_widget_attrs(gtk_widget: &gtk4::Widget, key: &str, value: &Property) {
    // // checking deprecated keys
    // // see eww issue #251 (https://github.com/elkowar/eww/issues/251)
    // for deprecated in DEPRECATED_ATTRS.iter() {
    //     if props.contains_key(*deprecated) {
    //         eprintln!("Warning: attribute `{}` is deprecated and ignored", deprecated);
    //     }
    // }

    match key {
        "visible" => {
            bind_property!(&value, "visible", get_bool_prop, [gtk_widget], |v: bool| {
                gtk_widget.set_visible(v);
            });
        }
        "class" => {
            bind_property!(&value, "class", get_string_prop, [gtk_widget], |class_str: String| {
                // remove all classes
                for class in gtk_widget.css_classes() {
                    gtk_widget.remove_css_class(&class);
                }

                // then apply the classes
                for class in class_str.split_whitespace() {
                    gtk_widget.add_css_class(class);
                }
            });
        }
        "style" => {
            bind_property!(&value, "style", get_string_prop, [gtk_widget], |style_str: String| {
                let css_provider = gtk4::CssProvider::new();
                let scss = format!("* {{ {} }}", style_str);
                if let Ok(compiled) = grass::from_string(scss, &grass::Options::default()) {
                    css_provider.load_from_string(&compiled);
                    #[allow(deprecated)]
                    gtk_widget.style_context().add_provider(&css_provider, 950);
                }
            });
        }
        "css" => {
            bind_property!(&value, "css", get_string_prop, [gtk_widget], |css_str: String| {
                let css_provider = gtk4::CssProvider::new();
                if let Ok(compiled) = grass::from_string(css_str, &grass::Options::default()) {
                    css_provider.load_from_string(&compiled);
                    #[allow(deprecated)]
                    gtk_widget.style_context().add_provider(&css_provider, 950);
                }
            });
        }
        "valign" => {
            bind_property!(&value, "valign", get_string_prop, [gtk_widget], |valign: String| {
                if let Ok(a) = parse_align(&valign) {
                    gtk_widget.set_valign(a);
                }
            });
        }
        "halign" => {
            bind_property!(&value, "halign", get_string_prop, [gtk_widget], |halign: String| {
                if let Ok(a) = parse_align(&halign) {
                    gtk_widget.set_halign(a);
                }
            });
        }
        "vexpand" => {
            bind_property!(&value, "vexpand", get_bool_prop, [gtk_widget], |v: bool| {
                gtk_widget.set_vexpand(v);
            });
        }
        "hexpand" => {
            bind_property!(&value, "hexpand", get_bool_prop, [gtk_widget], |v: bool| {
                gtk_widget.set_hexpand(v);
            });
        }
        "width" => {
            bind_property!(&value, "width", get_i32_prop, [gtk_widget], |w: i32| {
                gtk_widget.set_width_request(w);
            });
        }
        "height" => {
            bind_property!(&value, "height", get_i32_prop, [gtk_widget], |h: i32| {
                gtk_widget.set_height_request(h);
            });
        }
        "active" => {
            bind_property!(&value, "active", get_bool_prop, [gtk_widget], |v: bool| {
                gtk_widget.set_sensitive(v);
            });
        }
        "tooltip" => {
            bind_property!(&value, "tooltip", get_string_prop, [gtk_widget], |tooltip: String| {
                gtk_widget.set_tooltip_text(Some(&tooltip));
            });
        }
        "can_target" => {
            bind_property!(&value, "can_target", get_bool_prop, [gtk_widget], |v: bool| {
                gtk_widget.set_can_target(v);
            });
        }
        "focusable" => {
            bind_property!(&value, "focusable", get_bool_prop, [gtk_widget], |v: bool| {
                gtk_widget.set_focusable(v);
            });
        }
        "widget_name" => {
            bind_property!(&value, "widget_name", get_string_prop, [gtk_widget], |name: String| {
                gtk_widget.set_widget_name(&name);
            });
        }
        _ => {}
    }
}

/// Shared rage atribute
fn resolve_range_attrs(
    gtk_widget: &gtk4::Range,
    key: &str,
    value: &Property,
    range_dat: Rc<RefCell<RangeCtrlData>>,
) -> Result<()> {
    match key {
        "min" => {
            bind_property!(&value, &key, get_f64_prop, [gtk_widget], |min: f64| {
                gtk_widget.adjustment().set_lower(min);
            });
        }
        "max" => {
            bind_property!(&value, &key, get_f64_prop, [gtk_widget], |max: f64| {
                gtk_widget.adjustment().set_upper(max);
            });
        }
        "value" => {
            // We keep track of the last value that has been set via gtk_widget.set_value (by a change in the value property).
            // We do this so we can detect if the new value came from a scripted change or from a user input from within the value_changed handler
            // and only run on_change when it's caused by manual user input
            bind_property!(&value, &key, get_f64_prop, [gtk_widget, range_dat], |v: f64| {
                if !range_dat.borrow().is_being_dragged {
                    range_dat.borrow_mut().last_set_value = Some(v);
                    gtk_widget.set_value(v);
                }
            });
        }
        "timeout" => {
            let new_timeout = get_duration_prop(value, key).unwrap_or(Duration::from_millis(200));
            range_dat.borrow_mut().cmd_timeout = new_timeout;
        }
        "onchange" => {
            bind_property!(&value, "onchange", get_string_prop, [range_dat], |onchange: String| {
                range_dat.borrow_mut().onchange_cmd = onchange;
            });
        }
        _ => {}
    }

    Ok(())
}
