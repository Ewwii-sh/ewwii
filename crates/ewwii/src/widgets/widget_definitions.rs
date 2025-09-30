#![allow(clippy::option_map_unit_fn)]

use crate::util;
use crate::widgets::build_widget::{build_gtk_widget, WidgetInput};
use anyhow::{anyhow, bail, Result};
use gdk::{ModifierType, NotifyType};
use gtk4::glib::translate::FromGlib;
use gtk4::{self, prelude::*};
use gtk4::{gdk, glib, pango};
use gtk4::{GestureClick, EventControllerScroll, EventControllerMotion};
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
    Remove(u64, u64),                 // widget_id, parent_id
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
                PatchGtkWidget::Remove(widget_id, parent_id) => {
                    self.remove_widget(widget_id, parent_id)
                }
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
        for (id, new_info) in &old_map {
            if !new_map.contains_key(id) {
                patch.push(PatchGtkWidget::Remove(
                    *id,
                    new_info.parent_id.expect(&format!(
                        "Parent ID must exist. Widget type: {}",
                        &new_info.widget_type
                    )),
                ));
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
            let parent_widget = parent.widget.clone();

            if let Some(container) = parent_widget.dynamic_cast::<gtk4::Container>().ok() {
                // check if the widget already exists
                let position = self.widgets.get(&widget_id).and_then(|old_entry| {
                    container.children().iter().position(|w| w == &old_entry.widget)
                });

                // obliterate that widget....
                // how dare it try to create duplication...
                if let Some(old_entry) = self.widgets.get(&widget_id) {
                    container.remove(&old_entry.widget);
                }

                // `build_gtk_widget` also inserts info into widgetentry
                // self is passed for that reason.
                let gtk_widget = build_gtk_widget(&WidgetInput::BorrowedNode(widget_node), self)?;

                // insert into container
                if let Some(box_container) = container.clone().dynamic_cast::<gtk4::Box>().ok() {
                    box_container.add(&gtk_widget);

                    if let Some(pos) = position {
                        log::trace!("Reordering contents of gtk4::Box. Position: '{}'", pos);
                        box_container.reorder_child(&gtk_widget, pos as i32);
                    }
                } else {
                    container.add(&gtk_widget);
                }
            }
        }

        Ok(())
    }

    pub fn update_props(&self, widget_id: u64, new_props: Map) {
        if let Some(entry) = self.widgets.get(&widget_id) {
            (entry.update_fn)(&new_props);
        }
    }

    // pub fn remove_widget(&mut self, widget_id: u64) {
    //     if let Some(entry) = self.widgets.remove(&widget_id) {
    //         if let Some(parent) = entry.widget.parent() {
    //             if let Ok(container) = parent.downcast::<gtk4::Container>() {
    //                 container.remove(&entry.widget);
    //             }
    //         }
    //     }
    // }

    pub fn remove_widget(&mut self, widget_id: u64, parent_id: u64) {
        log::trace!("Removing '{}' from '{}'", widget_id, parent_id);
        if let Some(entry) = self.widgets.remove(&widget_id) {
            if let Some(parent) = self.widgets.get(&parent_id) {
                let parent_widget = parent.widget.clone();

                if let Some(container) = parent_widget.dynamic_cast::<gtk4::Container>().ok() {
                    container.remove(&entry.widget);
                }
            }
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
        gtk_widget.add(&child_widget);
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
    gtk_widget.add(&first);
    first.show();
    for child in children {
        let child = child?;
        gtk_widget.add_overlay(&child);
        gtk_widget.set_overlay_pass_through(&child, true);
        child.show();
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
    gtk_widget.add(&content_widget);

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

pub(super) fn build_center_box(
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

    let count = children.len();

    if count < 3 {
        bail!("centerbox must contain exactly 3 children");
    } else if count > 3 {
        bail!("centerbox must contain exactly 3 children, but got more");
    }

    let first = build_gtk_widget(
        &WidgetInput::Node(children.get(0).cloned().ok_or_else(|| anyhow!("missing child 0"))?),
        widget_registry,
    )?;
    let center = build_gtk_widget(
        &WidgetInput::Node(children.get(1).cloned().ok_or_else(|| anyhow!("missing child 1"))?),
        widget_registry,
    )?;
    let end = build_gtk_widget(
        &WidgetInput::Node(children.get(2).cloned().ok_or_else(|| anyhow!("missing child 2"))?),
        widget_registry,
    )?;

    let gtk_widget = gtk4::Box::new(orientation, 0);
    gtk_widget.pack_start(&first, true, true, 0);
    gtk_widget.set_center_widget(Some(&center));
    gtk_widget.pack_end(&end, true, true, 0);

    first.show();
    center.show();
    end.show();

    let gtk_widget_clone = gtk_widget.clone();

    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let orientation = match props
            .get("orientation")
            .and_then(|v| v.clone().try_cast::<String>())
            .map(|s| parse_orientation(&s))
            .transpose()
        {
            Ok(opt) => opt.unwrap_or(gtk4::Orientation::Horizontal),
            Err(e) => {
                eprintln!("Error parsing orientation: {:?}", e);
                gtk4::Orientation::Horizontal
            }
        };

        gtk_widget_clone.set_orientation(orientation);

        // now re-apply generic widget attrs
        if let Err(err) =
            resolve_rhai_widget_attrs(&gtk_widget_clone.clone().upcast::<gtk4::Widget>(), &props)
        {
            eprintln!("Failed to update widget attrs: {:?}", err);
        }
    });

    let id = hash_props_and_type(&props, "CenterBox");

    widget_registry
        .widgets
        .insert(id, WidgetEntry { update_fn, widget: gtk_widget.clone().upcast() });

    resolve_rhai_widget_attrs(&gtk_widget.clone().upcast::<gtk4::Widget>(), &props)?;

    Ok(gtk_widget)
}

pub(super) fn build_event_box(
    props: &Map,
    children: &Vec<WidgetNode>,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Box> {
    let gtk_widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    let hover_controller = EventControllerMotion::new();
    let gesture_controller = GestureClick::new();
    let scroll_controller = EventControllerScroll::new(gtk4::Orientation::Both, Some(20.0));

    // Support :hover selector
    hover_controller.connect_enter(|gtk_widget, evt| {
        if evt.detail() != NotifyType::Inferior {
            gtk_widget.set_state_flags(gtk4::StateFlags::PRELIGHT, false);
        }
    });

    hover_controller.connect_leave(|gtk_widget, evt| {
        if evt.detail() != NotifyType::Inferior {
            gtk_widget.unset_state_flags(gtk4::StateFlags::PRELIGHT);
        }
    });

    // Support :active selector
    gesture_controller.connect_pressed(|gtk_widget, _, _, _| {
        gtk_widget.set_state_flags(gtk4::StateFlags::ACTIVE, false);
    });

    gesture_controller.connect_released(|gtk_widget, _, _, _| {
        gtk_widget.unset_state_flags(gtk4::StateFlags::ACTIVE);
    });

    // onscroll - event to execute when the user scrolls with the mouse over the widget. The placeholder `{}` used in the command will be replaced with either `up` or `down`.
    let apply_props = |props: &Map, widget: &gtk4::Box| -> Result<()> {
        // timeout - timeout of the command. Default: "200ms"
        let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

        if let Ok(onscroll) = get_string_prop(&props, "onscroll", None) {
            connect_signal_handler!(
                widget,
                scroll_controller.connect_scroll(move |_, evt| {
                    let delta = evt.delta().1;
                    if delta != 0f64 {
                        // Ignore the first event https://bugzilla.gnome.org/show_bug.cgi?id=675959
                        run_command(
                            timeout,
                            &onscroll,
                            &[if delta < 0f64 { "up" } else { "down" }],
                        );
                    }
                })
            );
        }


        // onhover - event to execute when the user hovers over the widget
        if let Ok(onhover) = get_string_prop(&props, "onhover", None) {
            connect_signal_handler!(
                widget,
                hover_controller.connect_enter(move |_, x, y| {
                    run_command(timeout, &onhover, &[x, y]);
                })
            );
        }

        // onhoverlost - event to execute when the user losts hovers over the widget
        if let Ok(onhoverlost) = get_string_prop(&props, "onhoverlost", None) {
            connect_signal_handler!(
                widget,
                hover_controller.connect_leave(move |_, evt| {
                    if evt.detail() != NotifyType::Inferior {
                        run_command(timeout, &onhoverlost, &[evt.position().0, evt.position().1]);
                    }
                })
            );
        }

        // cursor - Cursor to show while hovering (see [gtk3-cursors](https://docs.gtk.org/gdk3/ctor.Cursor.new_from_name.html) for possible names)
        if let Ok(cursor) = get_string_prop(&props, "cursor", None) {
            connect_signal_handler!(
                widget,
                hover_controller.connect_enter(move |widget, _, _| {
                    let display = gdk::Display::default();
                    let gdk_window = widget.window();
                    if let (Some(display), Some(gdk_window)) = (display, gdk_window) {
                        gdk_window
                            .set_cursor(gdk::Cursor::from_name(&display, &cursor).as_ref());
                    }
                })
            );
            connect_signal_handler!(
                widget,
                hover_controller.connect_leave(move |widget, _evt| {
                    if _evt.detail() != NotifyType::Inferior {
                        let gdk_window = widget.window();
                        if let Some(gdk_window) = gdk_window {
                            gdk_window.set_cursor(None);
                        }
                    }
                })
            );
        }

        // ondropped - Command to execute when something is dropped on top of this element. The placeholder `{}` used in the command will be replaced with the uri to the dropped thing.
        if let Ok(ondropped) = get_string_prop(&props, "ondropped", None) {
            widget.drag_dest_set(
                DestDefaults::ALL,
                &[
                    TargetEntry::new(
                        "text/uri-list",
                        gtk4::TargetFlags::OTHER_APP | gtk4::TargetFlags::OTHER_WIDGET,
                        0,
                    ),
                    TargetEntry::new(
                        "text/plain",
                        gtk4::TargetFlags::OTHER_APP | gtk4::TargetFlags::OTHER_WIDGET,
                        0,
                    ),
                ],
                gdk::DragAction::COPY,
            );
            connect_signal_handler!(
                widget,
                widget.connect_drag_data_received(
                    move |_, _, _x, _y, selection_data, _target_type, _timestamp| {
                        if let Some(data) = selection_data.uris().first() {
                            run_command(
                                timeout,
                                &ondropped,
                                &[data.to_string(), "file".to_string()],
                            );
                        } else if let Some(data) = selection_data.text() {
                            run_command(
                                timeout,
                                &ondropped,
                                &[data.to_string(), "text".to_string()],
                            );
                        }
                    }
                )
            );
        }

        // dragtype - Type of value that should be dragged from this widget. Possible values: $dragtype
        let dragtype = get_string_prop(&props, "drag_type", Some("file"))?;

        // dragvalue - URI that will be provided when dragging from this widget
        if let Ok(dragvalue) = get_string_prop(&props, "dragvalue", None) {
            let dragtype = parse_dragtype(&dragtype)?;
            if dragvalue.is_empty() {
                widget.drag_source_unset();
            } else {
                let target_entry = match dragtype {
                    DragEntryType::File => TargetEntry::new(
                        "text/uri-list",
                        gtk4::TargetFlags::OTHER_APP | gtk4::TargetFlags::OTHER_WIDGET,
                        0,
                    ),
                    DragEntryType::Text => TargetEntry::new(
                        "text/plain",
                        gtk4::TargetFlags::OTHER_APP | gtk4::TargetFlags::OTHER_WIDGET,
                        0,
                    ),
                };
                widget.drag_source_set(
                    ModifierType::BUTTON1_MASK,
                    &[target_entry.clone()],
                    gdk::DragAction::COPY | gdk::DragAction::MOVE,
                );
                widget.drag_source_set_target_list(Some(&TargetList::new(&[target_entry])));
            }

            connect_signal_handler!(widget, if !dragvalue.is_empty(), widget.connect_drag_data_get(move |_, _, data, _, _| {
                match dragtype {
                    DragEntryType::File => data.set_uris(&[&dragvalue]),
                    DragEntryType::Text => data.set_text(&dragvalue),
                };
            }));
        }

        // onclick - command to run when the widget is clicked
        let onclick = get_string_prop(&props, "onclick", Some(""))?;
        // onmiddleclick - command to run when the widget is middleclicked
        let onmiddleclick = get_string_prop(&props, "onmiddleclick", Some(""))?;
        // onrightclick - command to run when the widget is rightclicked
        let onrightclick = get_string_prop(&props, "onrightclick", Some(""))?;

        connect_signal_handler!(
            widget,
            gesture_controller.connect_released(move |_, evt, _, _| {
                match evt.button() {
                    1 => run_command(timeout, &onclick, &[] as &[&str]),
                    2 => run_command(timeout, &onmiddleclick, &[] as &[&str]),
                    3 => run_command(timeout, &onrightclick, &[] as &[&str]),
                    _ => {}
                }
                glib::Propagation::Proceed
            })
        );

        Ok(())
    };

    gtk_widget.add_controller(gesture_controller);
    gtk_widget.add_controller(hover_controller);
    gtk_widget.add_controller(scroll_controller);

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
        bail!("expander must contain exactly one element");
    } else if count > 1 {
        bail!("expander must contain exactly one element, but got more");
    }

    let child = children.get(0).cloned().ok_or_else(|| anyhow!("missing child 0"))?;
    let child_widget = build_gtk_widget(&WidgetInput::Node(child), widget_registry)?;
    gtk_widget.add(&child_widget);
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

        let same_size = get_bool_prop(&props, "same_size", Some(false))?;
        widget.set_homogeneous(same_size);

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

pub(super) fn build_gtk_image(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Image> {
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
            let paintable = pixbuf_animation.to_printable();
            image.set_paintable(Some(&paintable));
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
            let icon_size = get_string_prop(&props, "icon-size", Some("button"))?;
            widget.set_from_icon_name(Some(&icon_name), parse_icon_size(&icon_size)?);
            // return Ok(widget);
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

pub(super) fn build_gtk_button(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Button> {
    let gtk_widget = gtk4::Button::new();

    let key_controller = EventControllerKey::new();
    let mouse_controller = GestureClick::new();

    let apply_props = |props: &Map, widget: &gtk4::Button| -> Result<()> {
        let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

        let onclick = get_string_prop(&props, "onclick", Some(""))?;
        let onmiddleclick = get_string_prop(&props, "onmiddleclick", Some(""))?;
        let onrightclick = get_string_prop(&props, "onrightclick", Some(""))?;

        // animate button upon right-/middleclick (if gtk theme supports it)
        // since we do this, we can't use `connect_clicked` as that would always run `onclick` as well
        connect_signal_handler!(
            widget,
            mouse_controller.connect_pressed(move |button, _, _, _| {
                button.emit_activate();
            })
        );
        let onclick_ = onclick.clone();
        // mouse click events
        connect_signal_handler!(
            widget,
            mouse_controller.connect_released(move |_, evt, _, _| {
                match evt.button() {
                    1 => run_command(timeout, &onclick, &[] as &[&str]),
                    2 => run_command(timeout, &onmiddleclick, &[] as &[&str]),
                    3 => run_command(timeout, &onrightclick, &[] as &[&str]),
                    _ => {}
                }
            })
        );
        // keyboard events
        connect_signal_handler!(
            widget,
            key_controller.connect_key_released(move |_, evt| {
                match evt.scancode() {
                    // return
                    36 => run_command(timeout, &onclick_, &[] as &[&str]),
                    // space
                    65 => run_command(timeout, &onclick_, &[] as &[&str]),
                    _ => {}
                }
            })
        );

        if let Ok(button_label) = get_string_prop(&props, "label", None) {
            widget.set_label(&button_label);
        }

        Ok(())
    };

    apply_props(&props, &gtk_widget)?;

    gtk_widget.add_controller(key_controller);

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
    expander.set_child(Some(&child_widget));
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

pub(super) fn build_gtk_scale(
    props: &Map,
    widget_registry: &mut WidgetRegistry,
) -> Result<gtk4::Scale> {
    let gtk_widget = gtk4::Scale::new(
        gtk4::Orientation::Horizontal,
        Some(&gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 1.0)),
    );

    // only allow changing the value via the value property if the user isn't currently dragging
    let is_being_dragged = Rc::new(RefCell::new(false));

    // Reusable closure for applying props
    let apply_props =
        |props: &Map, widget: &gtk4::Scale, is_being_dragged: Rc<RefCell<bool>>| -> Result<()> {
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

            resolve_range_attrs(props, widget.upcast_ref::<gtk4::Range>(), is_being_dragged)?;
            Ok(())
        };

    apply_props(&props, &gtk_widget, is_being_dragged.clone())?;

    let gtk_widget_clone = gtk_widget.clone();
    let is_being_dragged_clone = is_being_dragged.clone();
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        let _ = apply_props(props, &gtk_widget_clone, is_being_dragged_clone.clone());

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
        css_provider
            .load_from_data(&grass::from_string(scss, &grass::Options::default())?);
        gtk_widget
            .style_context()
            .add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    if let Ok(css_str) = get_string_prop(&props, "css", None) {
        let css_provider = gtk4::CssProvider::new();
        css_provider
            .load_from_data(&grass::from_string(css_str, &grass::Options::default())?);
        gtk_widget
            .style_context()
            .add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
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

    Ok(())
}

/// Shared rage atribute
pub(super) fn resolve_range_attrs(
    props: &Map,
    gtk_widget: &gtk4::Range,
    is_being_dragged: Rc<RefCell<bool>>,
) -> Result<()> {
    let gesture = GestureClick::new();
    
    gesture.connect_pressed(glib::clone!(#[strong] is_being_dragged, move |_, _, _, _| {
        *is_being_dragged.borrow_mut() = true;
    }));

    gesture.connect_released(glib::clone!(#[strong] is_being_dragged, move |_, _, _, _| {
        *is_being_dragged.borrow_mut() = false;
    }));

    gtk_widget.add_controller(gesture);

    // We keep track of the last value that has been set via gtk_widget.set_value (by a change in the value property).
    // We do this so we can detect if the new value came from a scripted change or from a user input from within the value_changed handler
    // and only run on_change when it's caused by manual user input
    let last_set_value = Rc::new(RefCell::new(None));
    let last_set_value_clone = last_set_value.clone();

    if let Ok(value) = get_f64_prop(&props, "value", None) {
        if !*is_being_dragged.borrow() {
            *last_set_value.borrow_mut() = Some(value);
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
        let last_set_value = last_set_value_clone.clone();
        connect_signal_handler!(
            gtk_widget,
            gtk_widget.connect_value_changed(move |gtk_widget| {
                let value = gtk_widget.value();
                if last_set_value.borrow_mut().take() != Some(value) {
                    run_command(timeout, &onchange, &[value]);
                }
            })
        );
    }

    Ok(())
}
