#![allow(clippy::option_map_unit_fn)]

use crate::gtk::glib::translate::FromGlib;
use crate::gtk::prelude::LabelExt;
use crate::util;
use crate::widgets::build_widget::{build_gtk_widget, WidgetInput};
use anyhow::{anyhow, bail, Result};
use gdk::{ModifierType, NotifyType};
use gtk::{self, prelude::*, DestDefaults, TargetEntry, TargetList};
use gtk::{gdk, glib, pango};
use iirhai::widgetnode::{WidgetNode, hash_props_and_type};
use once_cell::sync::Lazy;
use rhai::Map;

use super::widget_definitions_helper::*;
use ewwii_shared_util::general_helper::*;
use std::{
    cell::RefCell,
    // cmp::Ordering,
    collections::HashSet,
    rc::Rc,
    time::Duration,
    collections::HashMap,
};

// custom widgets
use crate::widgets::{circular_progressbar::CircProg, transform::Transform};

/// Connect a gtk signal handler inside of this macro to ensure that when the same code gets run multiple times,
/// the previously connected singal handler first gets disconnected.
/// Can take an optional condition.
/// If the condition is false, we disconnect the handler without running the connect_expr,
/// thus not connecting a new handler unless the condition is met.
macro_rules! connect_signal_handler {
    ($widget:ident, if $cond:expr, $connect_expr:expr) => {{
        const KEY:&str = std::concat!("signal-handler:", std::line!());
        unsafe {
            let old = $widget.data::<gtk::glib::SignalHandlerId>(KEY);

            if let Some(old) = old {
                 let a = old.as_ref().as_raw();
                 $widget.disconnect(gtk::glib::SignalHandlerId::from_glib(a));
            }

            $widget.set_data::<gtk::glib::SignalHandlerId>(KEY, $connect_expr);
        }
    }};
    ($widget:ident, $connect_expr:expr) => {{
        connect_signal_handler!($widget, if true, $connect_expr)
    }};
}

pub type UpdateFn = Box<dyn Fn(&Map)>;

pub struct WidgetEntry {
    // pub widget: gtk::Widget, // not needed now
    pub update_fn: UpdateFn,
}

pub struct WidgetRegistry {
    widgets: HashMap<u64, WidgetEntry>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
        }
    }

    pub fn update_prop_changes(&self, id_to_props: HashMap<u64, Map>) {
        for (id, props) in id_to_props {
            if let Some(entry) = self.widgets.get(&id) {
                (entry.update_fn)(&props);
            } // else {
                // println!("Warning: id {} not found in widget_registry", id);
            // }
        }
    }
}

pub(super) fn build_gtk_box(props: Map, children: Vec<WidgetNode>, widget_registry: &mut WidgetRegistry) -> Result<gtk::Box> {
    // Parse initial props to create the widget:
    let orientation = props
        .get("orientation")
        .and_then(|v| v.clone().try_cast::<String>())
        .map(|s| parse_orientation(&s))
        .transpose()?
        .unwrap_or(gtk::Orientation::Horizontal);

    let spacing = props.get("spacing").and_then(|v| v.clone().try_cast::<i64>()).unwrap_or(0) as i32;

    let space_evenly = get_bool_prop(&props, "space_evenly", Some(true))?;

    let gtk_widget = gtk::Box::new(orientation, spacing);
    gtk_widget.set_homogeneous(space_evenly);

    for child in children {
        let child_widget = build_gtk_widget(WidgetInput::Node(child), widget_registry)?;
        gtk_widget.add(&child_widget);
    }

    let gtk_widget_clone = gtk_widget.clone();
    
    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        if let Some(orientation_str) = props.get("orientation").and_then(|v| v.clone().try_cast::<String>()) {
            if let Ok(orientation) = parse_orientation(&orientation_str) {
                gtk_widget_clone.set_orientation(orientation);
            }
        }

        if let Some(spacing_val) = props.get("spacing").and_then(|v| v.clone().try_cast::<i64>()) {
            gtk_widget_clone.set_spacing(spacing_val as i32);
        }

        if let Ok(space_evenly) = get_bool_prop(props, "space_evenly", Some(true)) {
            gtk_widget_clone.set_homogeneous(space_evenly);
        }
    });

    let id = hash_props_and_type(&props, "Box");

    widget_registry.widgets.insert(id, WidgetEntry {
        // widget: gtk_widget.upcast(),
        update_fn,
    });

    Ok(gtk_widget)
}

pub(super) fn build_gtk_overlay(children: Vec<WidgetNode>, widget_registry: &mut WidgetRegistry) -> Result<gtk::Overlay> {
    let gtk_widget = gtk::Overlay::new();

    let count = children.len();

    if count < 1 {
        bail!("overlay must contain at least one element");
    }

    let mut children = children.into_iter().map(|child| build_gtk_widget(WidgetInput::Node(child), widget_registry));

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
    Ok(gtk_widget)
}

pub(super) fn build_tooltip(children: Vec<WidgetNode>, widget_registry: &mut WidgetRegistry) -> Result<gtk::Box> {
    let gtk_widget = gtk::Box::new(gtk::Orientation::Horizontal, 0);
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
    let content_widget = build_gtk_widget(WidgetInput::Node(content_node), widget_registry)?;
    gtk_widget.add(&content_widget);

    let tooltip_node = Rc::new(tooltip_node);
    let tooltip_widget = build_gtk_widget(WidgetInput::Node(Rc::clone(&tooltip_node).as_ref().clone()), widget_registry)
        .expect("Failed to build tooltip widget");

    gtk_widget.connect_query_tooltip(move |_widget, _x, _y, _keyboard_mode, tooltip| {
        tooltip.set_custom(Some(&tooltip_widget));
        true
    });

    Ok(gtk_widget)
}

pub(super) fn build_center_box(props: Map, children: Vec<WidgetNode>, widget_registry: &mut WidgetRegistry) -> Result<gtk::Box> {
    let orientation = props
        .get("orientation")
        .and_then(|v| v.clone().try_cast::<String>())
        .map(|s| parse_orientation(&s))
        .transpose()?
        .unwrap_or(gtk::Orientation::Horizontal);

    let count = children.len();

    if count < 3 {
        bail!("centerbox must contain exactly 3 children");
    } else if count > 3 {
        bail!("centerbox must contain exactly 3 children, but got more");
    }

    let first = build_gtk_widget(WidgetInput::Node(children.get(0).cloned().ok_or_else(|| anyhow!("missing child 0"))?), widget_registry)?;
    let center = build_gtk_widget(WidgetInput::Node(children.get(1).cloned().ok_or_else(|| anyhow!("missing child 1"))?), widget_registry)?;
    let end = build_gtk_widget(WidgetInput::Node(children.get(2).cloned().ok_or_else(|| anyhow!("missing child 2"))?), widget_registry)?;

    let gtk_widget = gtk::Box::new(orientation, 0);
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
            .transpose() {
                Ok(opt) => opt.unwrap_or(gtk::Orientation::Horizontal),
                Err(e) => {
                    eprintln!("Error parsing orientation: {:?}", e);
                    gtk::Orientation::Horizontal
                }
            };

        gtk_widget_clone.set_orientation(orientation);
    });


    let id = hash_props_and_type(&props, "CenterBox");

    widget_registry.widgets.insert(id, WidgetEntry {
        update_fn
    });

    Ok(gtk_widget)
}

pub(super) fn build_gtk_event_box(props: Map, children: Vec<WidgetNode>, widget_registry: &mut WidgetRegistry) -> Result<gtk::EventBox> {
    let gtk_widget = gtk::EventBox::new();

    // Support :hover selector
    gtk_widget.connect_enter_notify_event(|gtk_widget, evt| {
        if evt.detail() != NotifyType::Inferior {
            gtk_widget.set_state_flags(gtk::StateFlags::PRELIGHT, false);
        }
        glib::Propagation::Proceed
    });

    gtk_widget.connect_leave_notify_event(|gtk_widget, evt| {
        if evt.detail() != NotifyType::Inferior {
            gtk_widget.unset_state_flags(gtk::StateFlags::PRELIGHT);
        }
        glib::Propagation::Proceed
    });

    // Support :active selector
    gtk_widget.connect_button_press_event(|gtk_widget, _| {
        gtk_widget.set_state_flags(gtk::StateFlags::ACTIVE, false);
        glib::Propagation::Proceed
    });

    gtk_widget.connect_button_release_event(|gtk_widget, _| {
        gtk_widget.unset_state_flags(gtk::StateFlags::ACTIVE);
        glib::Propagation::Proceed
    });

    // timeout - timeout of the command. Default: "200ms"
    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    // onscroll - event to execute when the user scrolls with the mouse over the widget. The placeholder `{}` used in the command will be replaced with either `up` or `down`.
    if let Ok(onscroll) = get_string_prop(&props, "onscroll", None) {
        gtk_widget.add_events(gdk::EventMask::SCROLL_MASK);
        gtk_widget.add_events(gdk::EventMask::SMOOTH_SCROLL_MASK);
        connect_signal_handler!(
            gtk_widget,
            gtk_widget.connect_scroll_event(move |_, evt| {
                let delta = evt.delta().1;
                if delta != 0f64 {
                    // Ignore the first event https://bugzilla.gnome.org/show_bug.cgi?id=675959
                    run_command(timeout, &onscroll, &[if delta < 0f64 { "up" } else { "down" }]);
                }
                glib::Propagation::Proceed
            })
        );
    }

    // onhover - event to execute when the user hovers over the widget
    if let Ok(onhover) = get_string_prop(&props, "onhover", None) {
        gtk_widget.add_events(gdk::EventMask::ENTER_NOTIFY_MASK);
        connect_signal_handler!(
            gtk_widget,
            gtk_widget.connect_enter_notify_event(move |_, evt| {
                if evt.detail() != NotifyType::Inferior {
                    run_command(timeout, &onhover, &[evt.position().0, evt.position().1]);
                }
                glib::Propagation::Proceed
            })
        );
    }

    // onhoverlost - event to execute when the user losts hovers over the widget
    if let Ok(onhoverlost) = get_string_prop(&props, "onhoverlost", None) {
        gtk_widget.add_events(gdk::EventMask::LEAVE_NOTIFY_MASK);
        connect_signal_handler!(
            gtk_widget,
            gtk_widget.connect_leave_notify_event(move |_, evt| {
                if evt.detail() != NotifyType::Inferior {
                    run_command(timeout, &onhoverlost, &[evt.position().0, evt.position().1]);
                }
                glib::Propagation::Proceed
            })
        );
    }

    // cursor - Cursor to show while hovering (see [gtk3-cursors](https://docs.gtk.org/gdk3/ctor.Cursor.new_from_name.html) for possible names)
    if let Ok(cursor) = get_string_prop(&props, "cursor", None) {
        gtk_widget.add_events(gdk::EventMask::ENTER_NOTIFY_MASK);
        gtk_widget.add_events(gdk::EventMask::LEAVE_NOTIFY_MASK);

        connect_signal_handler!(
            gtk_widget,
            gtk_widget.connect_enter_notify_event(move |widget, _evt| {
                if _evt.detail() != NotifyType::Inferior {
                    let display = gdk::Display::default();
                    let gdk_window = widget.window();
                    if let (Some(display), Some(gdk_window)) = (display, gdk_window) {
                        gdk_window.set_cursor(gdk::Cursor::from_name(&display, &cursor).as_ref());
                    }
                }
                glib::Propagation::Proceed
            })
        );
        connect_signal_handler!(
            gtk_widget,
            gtk_widget.connect_leave_notify_event(move |widget, _evt| {
                if _evt.detail() != NotifyType::Inferior {
                    let gdk_window = widget.window();
                    if let Some(gdk_window) = gdk_window {
                        gdk_window.set_cursor(None);
                    }
                }
                glib::Propagation::Proceed
            })
        );
    }

    // ondropped - Command to execute when something is dropped on top of this element. The placeholder `{}` used in the command will be replaced with the uri to the dropped thing.
    if let Ok(ondropped) = get_string_prop(&props, "ondropped", None) {
        gtk_widget.drag_dest_set(
            DestDefaults::ALL,
            &[
                TargetEntry::new("text/uri-list", gtk::TargetFlags::OTHER_APP | gtk::TargetFlags::OTHER_WIDGET, 0),
                TargetEntry::new("text/plain", gtk::TargetFlags::OTHER_APP | gtk::TargetFlags::OTHER_WIDGET, 0),
            ],
            gdk::DragAction::COPY,
        );
        connect_signal_handler!(
            gtk_widget,
            gtk_widget.connect_drag_data_received(move |_, _, _x, _y, selection_data, _target_type, _timestamp| {
                if let Some(data) = selection_data.uris().first() {
                    run_command(timeout, &ondropped, &[data.to_string(), "file".to_string()]);
                } else if let Some(data) = selection_data.text() {
                    run_command(timeout, &ondropped, &[data.to_string(), "text".to_string()]);
                }
            })
        );
    }

    // dragtype - Type of value that should be dragged from this widget. Possible values: $dragtype
    let dragtype = get_string_prop(&props, "drag_type", Some("file"))?;

    // dragvalue - URI that will be provided when dragging from this widget
    if let Ok(dragvalue) = get_string_prop(&props, "dragvalue", None) {
        let dragtype = parse_dragtype(&dragtype)?;
        if dragvalue.is_empty() {
            gtk_widget.drag_source_unset();
        } else {
            let target_entry = match dragtype {
                DragEntryType::File => {
                    TargetEntry::new("text/uri-list", gtk::TargetFlags::OTHER_APP | gtk::TargetFlags::OTHER_WIDGET, 0)
                }
                DragEntryType::Text => {
                    TargetEntry::new("text/plain", gtk::TargetFlags::OTHER_APP | gtk::TargetFlags::OTHER_WIDGET, 0)
                }
            };
            gtk_widget.drag_source_set(
                ModifierType::BUTTON1_MASK,
                &[target_entry.clone()],
                gdk::DragAction::COPY | gdk::DragAction::MOVE,
            );
            gtk_widget.drag_source_set_target_list(Some(&TargetList::new(&[target_entry])));
        }

        connect_signal_handler!(gtk_widget, if !dragvalue.is_empty(), gtk_widget.connect_drag_data_get(move |_, _, data, _, _| {
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

    gtk_widget.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
    connect_signal_handler!(
        gtk_widget,
        gtk_widget.connect_button_release_event(move |_, evt| {
            match evt.button() {
                1 => run_command(timeout, &onclick, &[] as &[&str]),
                2 => run_command(timeout, &onmiddleclick, &[] as &[&str]),
                3 => run_command(timeout, &onrightclick, &[] as &[&str]),
                _ => {}
            }
            glib::Propagation::Proceed
        })
    );

    let count = children.len();

    if count < 1 {
        bail!("expander must contain exactly one element");
    } else if count > 1 {
        bail!("expander must contain exactly one element, but got more");
    }

    let child = children.get(0).cloned().ok_or_else(|| anyhow!("missing child 0"))?;
    let child_widget = build_gtk_widget(WidgetInput::Node(child), widget_registry)?;
    gtk_widget.add(&child_widget);
    child_widget.show();

    let id = hash_props_and_type(&props, "EventBox");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_stack(props: Map, children: Vec<WidgetNode>, widget_registry: &mut WidgetRegistry) -> Result<gtk::Stack> {
    let gtk_widget = gtk::Stack::new();

    if children.is_empty() {
        return Err(anyhow!("stack must contain at least one element"));
    }

    let children = children.into_iter().map(|child| build_gtk_widget(WidgetInput::Node(child), widget_registry));

    for (i, child) in children.enumerate() {
        let child = child?;
        gtk_widget.add_named(&child, &i.to_string());
        child.show();
    }

    // parsing the properties
    if let Ok(selected) = get_i32_prop(&props, "selected", None) {
        gtk_widget.set_visible_child_name(&selected.to_string());
    }

    let transition = get_string_prop(&props, "transition", Some("crossfade"))?;
    gtk_widget.set_transition_type(parse_stack_transition(&transition)?);

    let same_size = get_bool_prop(&props, "same_size", Some(false))?;
    gtk_widget.set_homogeneous(same_size);

    let id = hash_props_and_type(&props, "Stack");

    Ok(gtk_widget)
}

pub(super) fn build_transform(props: Map, widget_registry: &mut WidgetRegistry) -> Result<Transform> {
    let widget = Transform::new();

    // rotate - the percentage to rotate
    if let Ok(rotate) = get_f64_prop(&props, "rotate", None) {
        widget.set_property("rotate", rotate);
    }

    // transform-origin-x - x coordinate of origin of transformation (px or %)
    if let Ok(transform_origin_x) = get_string_prop(&props, "transform_origin_x", None) {
        widget.set_property("transform-origin-x", transform_origin_x);
    }

    // transform-origin-y - y coordinate of origin of transformation (px or %)
    if let Ok(transform_origin_y) = get_string_prop(&props, "transform_origin_y", None) {
        widget.set_property("transform-origin-y", transform_origin_y);
    }

    // translate-x - the amount to translate in the x direction (px or %)
    if let Ok(translate_x) = get_string_prop(&props, "translate_x", None) {
        widget.set_property("translate-x", translate_x);
    }

    // translate-y - the amount to translate in the y direction (px or %)
    if let Ok(translate_y) = get_string_prop(&props, "translate_y", None) {
        widget.set_property("translate-y", translate_y);
    }

    // scale-x - the amount to scale in the x direction (px or %)
    if let Ok(scale_x) = get_string_prop(&props, "scale_x", None) {
        widget.set_property("scale-x", scale_x);
    }

    // scale-y - the amount to scale in the y direction (px or %)
    if let Ok(scale_y) = get_string_prop(&props, "scale_y", None) {
        widget.set_property("scale-y", scale_y);
    }

    let id = hash_props_and_type(&props, "Transform");

    Ok(widget)
}

pub(super) fn build_circular_progress_bar(props: Map, widget_registry: &mut WidgetRegistry) -> Result<CircProg> {
    let widget = CircProg::new();

    if let Ok(value) = get_f64_prop(&props, "value", None) {
        widget.set_property("value", value.clamp(0.0, 100.0));
    }

    if let Ok(start_at) = get_f64_prop(&props, "start_at", None) {
        widget.set_property("start-at", start_at.clamp(0.0, 100.0));
    }

    if let Ok(thickness) = get_f64_prop(&props, "thickness", None) {
        widget.set_property("thickness", thickness);
    }

    if let Ok(clockwise) = get_f64_prop(&props, "clockwise", None) {
        widget.set_property("clockwise", clockwise);
    }

    let id = hash_props_and_type(&props, "CircularProgressBar");

    Ok(widget)
}

pub(super) fn build_graph(props: Map, widget_registry: &mut WidgetRegistry) -> Result<super::graph::Graph> {
    let widget = super::graph::Graph::new();

    if let Ok(value) = get_f64_prop(&props, "value", None) {
        if value.is_nan() || value.is_infinite() {
            return Err(anyhow!("Graph's value should never be NaN or infinite"));
        }
        widget.set_property("value", value);
    }

    if let Ok(thickness) = get_f64_prop(&props, "thickness", None) {
        widget.set_property("thickness", thickness);
    }

    if let Ok(time_range) = get_duration_prop(&props, "time_range", None) {
        widget.set_property("time-range", time_range.as_millis() as u64);
    }

    let min = get_f64_prop(&props, "min", Some(0.0)).ok();
    let max = get_f64_prop(&props, "max", Some(100.0)).ok();

    if let (Some(mi), Some(ma)) = (min, max) {
        if mi > ma {
            return Err(anyhow!("Graph's min ({mi}) should never be higher than max ({ma})"));
        }
    }

    if let Some(mi) = min {
        widget.set_property("min", mi);
    }

    if let Some(ma) = max {
        widget.set_property("max", ma);
    }

    if let Ok(dynamic) = get_bool_prop(&props, "dynamic", None) {
        widget.set_property("dynamic", dynamic);
    }

    if let Ok(line_style) = get_string_prop(&props, "line_style", None) {
        widget.set_property("line-style", line_style);
    }

    // flip-x - whether the x axis should go from high to low
    if let Ok(flip_x) = get_bool_prop(&props, "flip_x", None) {
        widget.set_property("flip-x", flip_x);
    }

    // flip-y - whether the y axis should go from high to low
    if let Ok(flip_y) = get_bool_prop(&props, "flip_y", None) {
        widget.set_property("flip-y", flip_y);
    }

    // vertical - if set to true, the x and y axes will be exchanged
    if let Ok(vertical) = get_bool_prop(&props, "vertical", None) {
        widget.set_property("vertical", vertical);
    }

    let id = hash_props_and_type(&props, "Graph");

    Ok(widget)
}

pub(super) fn build_gtk_progress(props: Map, widget_registry: &mut WidgetRegistry) -> Result<gtk::ProgressBar> {
    let gtk_widget = gtk::ProgressBar::new();

    let orientation = props
        .get("orientation")
        .and_then(|v| v.clone().try_cast::<String>())
        .map(|s| parse_orientation(&s))
        .transpose()?
        .unwrap_or(gtk::Orientation::Horizontal);

    gtk_widget.set_orientation(orientation);

    if let Ok(flipped) = get_bool_prop(&props, "flipped", Some(false)) {
        gtk_widget.set_inverted(flipped)
    }

    if let Ok(bar_value) = get_f64_prop(&props, "value", None) {
        gtk_widget.set_fraction(bar_value / 100f64)
    }

    let id = hash_props_and_type(&props, "Progress");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_image(props: Map, widget_registry: &mut WidgetRegistry) -> Result<gtk::Image> {
    let gtk_widget = gtk::Image::new();

    let path = get_string_prop(&props, "path", None)?;
    let image_width = get_i32_prop(&props, "image_width", Some(-1))?;
    let image_height = get_i32_prop(&props, "image_height", Some(-1))?;
    let preserve_aspect_ratio = get_bool_prop(&props, "preserve_aspect_ratio", Some(true))?;
    let fill_svg = get_string_prop(&props, "fill_svg", Some(""))?;

    if !path.ends_with(".svg") && !fill_svg.is_empty() {
        log::warn!("Fill attribute ignored, file is not an svg image");
    }

    if path.ends_with(".gif") {
        let pixbuf_animation = gtk::gdk_pixbuf::PixbufAnimation::from_file(std::path::PathBuf::from(path))?;
        gtk_widget.set_from_animation(&pixbuf_animation);
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
            let stream = gtk::gio::MemoryInputStream::from_bytes(&gtk::glib::Bytes::from(svg_data.as_bytes()));
            pixbuf = gtk::gdk_pixbuf::Pixbuf::from_stream_at_scale(
                &stream,
                image_width,
                image_height,
                preserve_aspect_ratio,
                None::<&gtk::gio::Cancellable>,
            )?;
            stream.close(None::<&gtk::gio::Cancellable>)?;
        } else {
            pixbuf = gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(
                std::path::PathBuf::from(path),
                image_width,
                image_height,
                preserve_aspect_ratio,
            )?;
        }
        gtk_widget.set_from_pixbuf(Some(&pixbuf));
    }

    if let Ok(icon_name) = get_string_prop(&props, "icon", None) {
        let icon_size = get_string_prop(&props, "icon-size", Some("button"))?;
        gtk_widget.set_from_icon_name(Some(&icon_name), parse_icon_size(&icon_size)?);
        return Ok(gtk_widget);
    }

    let id = hash_props_and_type(&props, "Image");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_button(props: Map, widget_registry: &mut WidgetRegistry) -> Result<gtk::Button> {
    let gtk_widget = gtk::Button::new();

    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    let onclick = get_string_prop(&props, "onclick", Some(""))?;
    let onmiddleclick = get_string_prop(&props, "onmiddleclick", Some(""))?;
    let onrightclick = get_string_prop(&props, "onrightclick", Some(""))?;

    // animate button upon right-/middleclick (if gtk theme supports it)
    // since we do this, we can't use `connect_clicked` as that would always run `onclick` as well
    connect_signal_handler!(
        gtk_widget,
        gtk_widget.connect_button_press_event(move |button, _| {
            button.emit_activate();
            glib::Propagation::Proceed
        })
    );
    let onclick_ = onclick.clone();
    // mouse click events
    connect_signal_handler!(
        gtk_widget,
        gtk_widget.connect_button_release_event(move |_, evt| {
            match evt.button() {
                1 => run_command(timeout, &onclick, &[] as &[&str]),
                2 => run_command(timeout, &onmiddleclick, &[] as &[&str]),
                3 => run_command(timeout, &onrightclick, &[] as &[&str]),
                _ => {}
            }
            glib::Propagation::Proceed
        })
    );
    // keyboard events
    connect_signal_handler!(
        gtk_widget,
        gtk_widget.connect_key_release_event(move |_, evt| {
            match evt.scancode() {
                // return
                36 => run_command(timeout, &onclick_, &[] as &[&str]),
                // space
                65 => run_command(timeout, &onclick_, &[] as &[&str]),
                _ => {}
            }
            glib::Propagation::Proceed
        })
    );

    if let Ok(button_label) = get_string_prop(&props, "label", None) {
        gtk_widget.set_label(&button_label);
    }

    let id = hash_props_and_type(&props, "Button");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_label(props: Map, widget_registry: &mut WidgetRegistry) -> Result<gtk::Label> {
    let gtk_widget = gtk::Label::new(None);

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
                gtk_widget.set_max_width_chars(-1);
            } else {
                gtk_widget.set_max_width_chars(limit_width);
            }
            apply_ellipsize_settings(&gtk_widget, truncate, limit_width, truncate_left, show_truncated);
            text
        } else {
            gtk_widget.set_ellipsize(pango::EllipsizeMode::None);

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

        let unescaped = unescape::unescape(&t).ok_or_else(|| anyhow!("Failed to unescape..."))?;
        let final_text = if unindent { util::unindent(&unescaped) } else { unescaped };
        gtk_widget.set_text(&final_text);
    } else if has_markup {
        let markup = get_string_prop(&props, "markup", None)?;
        apply_ellipsize_settings(&gtk_widget, truncate, limit_width, truncate_left, show_truncated);
        gtk_widget.set_markup(&markup);
    } else {
        bail!("Either 'text' or 'markup' must be set");
    }

    if let Ok(wrap) = get_bool_prop(&props, "wrap", Some(false)) {
        gtk_widget.set_line_wrap(wrap);
    }

    if let Ok(angle) = get_f64_prop(&props, "angle", Some(0.0)) {
        gtk_widget.set_angle(angle);
    }

    let gravity = get_string_prop(&props, "gravity", Some("south"))?;
    gtk_widget.pango_context().set_base_gravity(parse_gravity(&gravity)?);

    if let Ok(xalign) = get_f64_prop(&props, "xalign", Some(0.5)) {
        gtk_widget.set_xalign(xalign as f32);
    }

    if let Ok(yalign) = get_f64_prop(&props, "yalign", Some(0.5)) {
        gtk_widget.set_yalign(yalign as f32);
    }

    let justify = get_string_prop(&props, "justify", Some("left"))?;
    gtk_widget.set_justify(parse_justification(&justify)?);

    let wrap_mode = get_string_prop(&props, "wrap_mode", Some("word"))?;
    gtk_widget.set_wrap_mode(parse_wrap_mode(&wrap_mode)?);

    if let Ok(lines) = get_i32_prop(&props, "lines", Some(-1)) {
        gtk_widget.set_lines(lines);
    }

    let gtk_widget_clone = gtk_widget.clone();

    let update_fn: UpdateFn = Box::new(move |props: &Map| {
        // Handle text or markup update
        if let Some(text_val) = props.get("text").and_then(|v| v.clone().try_cast::<String>()) {
            let truncate = get_bool_prop(props, "truncate", Some(false)).unwrap_or(false);
            let limit_width = get_i32_prop(props, "limit_width", Some(i32::MAX)).unwrap_or(i32::MAX);
            let truncate_left = get_bool_prop(props, "truncate_left", Some(false)).unwrap_or(false);
            let show_truncated = get_bool_prop(props, "show_truncated", Some(true)).unwrap_or(true);
            let unindent = get_bool_prop(props, "unindent", Some(true)).unwrap_or(true);

            if show_truncated {
                if limit_width == i32::MAX {
                    gtk_widget_clone.set_max_width_chars(-1);
                } else {
                    gtk_widget_clone.set_max_width_chars(limit_width);
                }
                apply_ellipsize_settings(&gtk_widget_clone, truncate, limit_width, truncate_left, show_truncated);
            } else {
                gtk_widget_clone.set_ellipsize(pango::EllipsizeMode::None);
            }

            let t = if show_truncated {
                text_val.clone()
            } else {
                let limit_width = limit_width as usize;
                let char_count = text_val.chars().count();
                if char_count > limit_width {
                    if truncate_left {
                        text_val.chars().skip(char_count - limit_width).collect()
                    } else {
                        text_val.chars().take(limit_width).collect()
                    }
                } else {
                    text_val.clone()
                }
            };

            if let Some(unescaped) = unescape::unescape(&t) {
                let final_text = if unindent { util::unindent(&unescaped) } else { unescaped };
                gtk_widget_clone.set_text(&final_text);
            }
        } else if let Some(markup_val) = props.get("markup").and_then(|v| v.clone().try_cast::<String>()) {
            let truncate = get_bool_prop(props, "truncate", Some(false)).unwrap_or(false);
            let limit_width = get_i32_prop(props, "limit_width", Some(i32::MAX)).unwrap_or(i32::MAX);
            let truncate_left = get_bool_prop(props, "truncate_left", Some(false)).unwrap_or(false);
            let show_truncated = get_bool_prop(props, "show_truncated", Some(true)).unwrap_or(true);

            apply_ellipsize_settings(&gtk_widget_clone, truncate, limit_width, truncate_left, show_truncated);
            gtk_widget_clone.set_markup(&markup_val);
        }

        if let Ok(wrap) = get_bool_prop(props, "wrap", Some(false)) {
            gtk_widget_clone.set_line_wrap(wrap);
        }

        if let Ok(angle) = get_f64_prop(props, "angle", Some(0.0)) {
            gtk_widget_clone.set_angle(angle);
        }

        if let Ok(xalign) = get_f64_prop(props, "xalign", Some(0.5)) {
            gtk_widget_clone.set_xalign(xalign as f32);
        }

        if let Ok(yalign) = get_f64_prop(props, "yalign", Some(0.5)) {
            gtk_widget_clone.set_yalign(yalign as f32);
        }

        if let Ok(justify) = get_string_prop(props, "justify", Some("left")) {
            if let Ok(j) = parse_justification(&justify) {
                gtk_widget_clone.set_justify(j);
            }
        }

        if let Ok(wrap_mode) = get_string_prop(props, "wrap_mode", Some("word")) {
            if let Ok(wm) = parse_wrap_mode(&wrap_mode) {
                gtk_widget_clone.set_wrap_mode(wm);
            }
        }

        if let Ok(lines) = get_i32_prop(props, "lines", Some(-1)) {
            gtk_widget_clone.set_lines(lines);
        }

        if let Ok(gravity) = get_string_prop(props, "gravity", Some("south")) {
            if let Ok(g) = parse_gravity(&gravity) {
                gtk_widget_clone.pango_context().set_base_gravity(g);
            }
        }
    });

    let id = hash_props_and_type(&props, "Label");

    widget_registry.widgets.insert(id, WidgetEntry {
        update_fn,
    });

    Ok(gtk_widget)
}


pub(super) fn build_gtk_input(props: Map, widget_registry: &mut WidgetRegistry) -> Result<gtk::Entry> {
    let gtk_widget = gtk::Entry::new();

    if let Ok(value) = get_string_prop(&props, "value", None) {
        gtk_widget.set_text(&value);
    }

    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    if let Ok(onchange) = get_string_prop(&props, "onchange", None) {
        connect_signal_handler!(
            gtk_widget,
            gtk_widget.connect_changed(move |gtk_widget| {
                run_command(timeout, &onchange, &[gtk_widget.text().to_string()]);
            })
        );
    }

    if let Ok(onaccept) = get_string_prop(&props, "onaccept", None) {
        connect_signal_handler!(
            gtk_widget,
            gtk_widget.connect_activate(move |gtk_widget| {
                run_command(timeout, &onaccept, &[gtk_widget.text().to_string()]);
            })
        );
    }

    let password: bool = get_bool_prop(&props, "password", Some(false))?;
    gtk_widget.set_visibility(!password);

    let id = hash_props_and_type(&props, "Input");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_calendar(props: Map, widget_registry: &mut WidgetRegistry) -> Result<gtk::Calendar> {
    let gtk_widget = gtk::Calendar::new();

    // day - the selected day
    if let Ok(day) = get_f64_prop(&props, "day", None) {
        if !(1f64..=31f64).contains(&day) {
            log::warn!("Calendar day is not a number between 1 and 31");
        } else {
            gtk_widget.set_day(day as i32)
        }
    }

    // month - the selected month
    if let Ok(month) = get_f64_prop(&props, "month", None) {
        if !(1f64..=12f64).contains(&month) {
            log::warn!("Calendar month is not a number between 1 and 12");
        } else {
            gtk_widget.set_month(month as i32 - 1)
        }
    }

    // year - the selected year
    if let Ok(year) = get_f64_prop(&props, "year", None) {
        gtk_widget.set_year(year as i32)
    }

    // show-details - show details
    if let Ok(show_details) = get_bool_prop(&props, "show_details", None) {
        gtk_widget.set_show_details(show_details)
    }

    // show-heading - show heading line
    if let Ok(show_heading) = get_bool_prop(&props, "show_heading", None) {
        gtk_widget.set_show_heading(show_heading)
    }

    // show-day-names - show names of days
    if let Ok(show_day_names) = get_bool_prop(&props, "show_day_names", None) {
        gtk_widget.set_show_day_names(show_day_names)
    }

    // show-week-numbers - show week numbers
    if let Ok(show_week_numbers) = get_bool_prop(&props, "show_week_numbers", None) {
        gtk_widget.set_show_week_numbers(show_week_numbers)
    }

    // timeout - timeout of the command. Default: "200ms"
    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    // onclick - command to run when the user selects a date. The `{0}` placeholder will be replaced by the selected day, `{1}` will be replaced by the month, and `{2}` by the year.
    if let Ok(onclick) = get_string_prop(&props, "onclick", None) {
        connect_signal_handler!(
            gtk_widget,
            gtk_widget.connect_day_selected(move |w| { run_command(timeout, &onclick, &[w.day(), w.month(), w.year()]) })
        );
    }

    let id = hash_props_and_type(&props, "Calendar");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_combo_box_text(props: Map, widget_registry: &mut WidgetRegistry) -> Result<gtk::ComboBoxText> {
    let gtk_widget = gtk::ComboBoxText::new();

    if let Ok(items) = get_vec_string_prop(&props, "items", None) {
        gtk_widget.remove_all();
        for i in items {
            gtk_widget.append_text(&i);
        }
    }

    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;
    let onchange = get_string_prop(&props, "onchange", Some(""))?;

    connect_signal_handler!(
        gtk_widget,
        gtk_widget.connect_changed(move |gtk_widget| {
            run_command(timeout, &onchange, &[gtk_widget.active_text().unwrap_or_else(|| "".into())]);
        })
    );

    let id = hash_props_and_type(&props, "ComboBoxText");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_expander(props: Map, children: Vec<WidgetNode>, widget_registry: &mut WidgetRegistry) -> Result<gtk::Expander> {
    let gtk_widget = gtk::Expander::new(None);

    let count = children.len();

    if count < 1 {
        bail!("expander must contain exactly one element");
    } else if count > 1 {
        bail!("expander must contain exactly one element, but got more");
    }

    let child = children.get(0).cloned().ok_or_else(|| anyhow!("missing child 0"))?;
    let child_widget = build_gtk_widget(WidgetInput::Node(child), widget_registry)?;
    gtk_widget.add(&child_widget);
    child_widget.show();

    if let Ok(name) = get_string_prop(&props, "name", None) {
        gtk_widget.set_label(Some(&name));
    }

    if let Ok(expanded) = get_bool_prop(&props, "expanded", None) {
        gtk_widget.set_expanded(expanded);
    }

    let id = hash_props_and_type(&props, "Expander");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_revealer(props: Map, children: Vec<WidgetNode>, widget_registry: &mut WidgetRegistry) -> Result<gtk::Revealer> {
    let gtk_widget = gtk::Revealer::new();

    let transition = get_string_prop(&props, "transition", Some("crossfade"))?;
    gtk_widget.set_transition_type(parse_revealer_transition(&transition)?);

    if let Ok(reveal) = get_bool_prop(&props, "reveal", None) {
        gtk_widget.set_reveal_child(reveal);
    }

    let duration = get_duration_prop(&props, "duration", Some(Duration::from_millis(500)))?;

    gtk_widget.set_transition_duration(duration.as_millis() as u32);

    match children.len() {
        0 => { /* maybe warn? */ }
        1 => {
            let child_widget = build_gtk_widget(WidgetInput::Node(children[0].clone()), widget_registry)?;
            gtk_widget.set_child(Some(&child_widget));
        }
        n => {
            return Err(anyhow!("A revealer must only have a maximum of 1 child but got: {}", n));
        }
    }

    let id = hash_props_and_type(&props, "Revealer");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_checkbox(props: Map, widget_registry: &mut WidgetRegistry) -> Result<gtk::CheckButton> {
    let gtk_widget = gtk::CheckButton::new();

    let checked = get_bool_prop(&props, "checked", Some(false))?;
    gtk_widget.set_active(checked);

    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;
    let onchecked = get_string_prop(&props, "onchecked", Some(""))?;
    let onunchecked = get_string_prop(&props, "onchecked", Some(""))?;

    connect_signal_handler!(
        gtk_widget,
        gtk_widget.connect_toggled(move |gtk_widget| {
            run_command(timeout, if gtk_widget.is_active() { &onchecked } else { &onunchecked }, &[] as &[&str]);
        })
    );

    let id = hash_props_and_type(&props, "Checkbox");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_color_button(props: Map, widget_registry: &mut WidgetRegistry) -> Result<gtk::ColorButton> {
    let gtk_widget = gtk::ColorButton::builder().build();

    // use-alpha - bool to wether or not use alpha
    if let Ok(use_alpha) = get_bool_prop(&props, "use_alpha", None) {
        gtk_widget.set_use_alpha(use_alpha);
    }

    // timeout - timeout of the command. Default: "200ms"
    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    // onchange - runs the code when the color was selected
    if let Ok(onchange) = get_string_prop(&props, "onchange", None) {
        connect_signal_handler!(
            gtk_widget,
            gtk_widget.connect_color_set(move |gtk_widget| {
                run_command(timeout, &onchange, &[gtk_widget.rgba()]);
            })
        );
    }

    let id = hash_props_and_type(&props, "ColorButton");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_color_chooser(props: Map, widget_registry: &mut WidgetRegistry) -> Result<gtk::ColorChooserWidget> {
    let gtk_widget = gtk::ColorChooserWidget::new();

    // use-alpha - bool to wether or not use alpha
    if let Ok(use_alpha) = get_bool_prop(&props, "use_alpha", None) {
        gtk_widget.set_use_alpha(use_alpha);
    }

    // timeout - timeout of the command. Default: "200ms"
    let timeout = get_duration_prop(&props, "timeout", Some(Duration::from_millis(200)))?;

    // onchange - runs the code when the color was selected
    if let Ok(onchange) = get_string_prop(&props, "onchange", None) {
        connect_signal_handler!(
            gtk_widget,
            gtk_widget.connect_color_activated(move |_a, color| {
                run_command(timeout, &onchange, &[*color]);
            })
        );
    }

    let id = hash_props_and_type(&props, "ColorChooser");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_scale(props: Map, widget_registry: &mut WidgetRegistry) -> Result<gtk::Scale> {
    let gtk_widget = gtk::Scale::new(gtk::Orientation::Horizontal, Some(&gtk::Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 1.0)));

    let flipped = get_bool_prop(&props, "flipped", Some(false))?;
    gtk_widget.set_inverted(flipped);

    if let Ok(marks) = get_string_prop(&props, "marks", None) {
        gtk_widget.clear_marks();
        for mark in marks.split(',') {
            gtk_widget.add_mark(mark.trim().parse()?, gtk::PositionType::Bottom, None)
        }
    }

    let draw_value = get_bool_prop(&props, "draw_value", Some(false))?;
    gtk_widget.set_draw_value(draw_value);

    if let Ok(value_pos) = get_string_prop(&props, "value_pos", None) {
        gtk_widget.set_value_pos(parse_position_type(&value_pos)?)
    }

    let round_digits = get_i32_prop(&props, "round_digits", Some(0))?;
    gtk_widget.set_round_digits(round_digits);

    resolve_range_attrs(&props, gtk_widget.upcast_ref::<gtk::Range>())?;

    let id = hash_props_and_type(&props, "Scale");

    Ok(gtk_widget)
}

pub(super) fn build_gtk_scrolledwindow(props: Map, children: Vec<WidgetNode>, widget_registry: &mut WidgetRegistry) -> Result<gtk::ScrolledWindow> {
    // I don't have single idea of what those two generics are supposed to be, but this works.
    let gtk_widget = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);

    let hscroll = get_bool_prop(&props, "hscroll", Some(true))?;
    let vscroll = get_bool_prop(&props, "vscroll", Some(true))?;

    gtk_widget.set_policy(
        if hscroll { gtk::PolicyType::Automatic } else { gtk::PolicyType::Never },
        if vscroll { gtk::PolicyType::Automatic } else { gtk::PolicyType::Never },
    );

    let count = children.len();

    if count < 1 {
        bail!("scrolled window must contain exactly one element");
    } else if count > 1 {
        bail!("scrolled window contain exactly one element, but got more");
    }

    let child = children.get(0).cloned().ok_or_else(|| anyhow!("missing child 0"))?;
    let child_widget = build_gtk_widget(WidgetInput::Node(child), widget_registry)?;
    gtk_widget.add(&child_widget);
    child_widget.show();

    let id = hash_props_and_type(&props, "ScrolledWindow");

    Ok(gtk_widget)
}

// * CSS/SCSS
/// Deprecated attributes from top of widget hierarchy
static DEPRECATED_ATTRS: Lazy<HashSet<&str>> =
    Lazy::new(|| ["timeout", "onscroll", "onhover", "cursor"].iter().cloned().collect());

/// Code that applies css/scss to widgets.
pub(super) fn resolve_rhai_widget_attrs(node: WidgetNode, gtk_widget: &gtk::Widget) -> Result<()> {
    // NOTE: The following variable should contain all widgets!
    let props = match node {
        WidgetNode::Box { props, .. }
        | WidgetNode::CenterBox { props, .. }
        | WidgetNode::EventBox { props, .. }
        | WidgetNode::Graph { props }
        | WidgetNode::Slider { props }
        | WidgetNode::Progress { props }
        | WidgetNode::Image { props }
        | WidgetNode::Button { props }
        | WidgetNode::Label { props }
        | WidgetNode::Input { props }
        | WidgetNode::Calendar { props }
        | WidgetNode::Checkbox { props }
        | WidgetNode::Revealer { props, .. }
        | WidgetNode::Scroll { props, .. } => props,
        _ => return Ok(()),
    };

    // checking deprecated keys
    // see eww issue #251 (https://github.com/elkowar/eww/issues/251)
    for deprecated in DEPRECATED_ATTRS.iter() {
        if props.contains_key(*deprecated) {
            eprintln!("Warning: attribute `{}` is deprecated and ignored", deprecated);
        }
    }

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
        for class in class_str.split_whitespace() {
            style_context.add_class(class);
        }
    }

    if let Ok(style_str) = get_string_prop(&props, "style", None) {
        let css_provider = gtk::CssProvider::new();
        let scss = format!("* {{ {} }}", style_str);
        css_provider.load_from_data(grass::from_string(scss, &grass::Options::default())?.as_bytes())?;
        gtk_widget.style_context().add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    if let Ok(css_str) = get_string_prop(&props, "css", None) {
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_data(grass::from_string(css_str, &grass::Options::default())?.as_bytes())?;
        gtk_widget.style_context().add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
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
pub(super) fn resolve_range_attrs(props: &Map, gtk_widget: &gtk::Range) -> Result<()> {
    gtk_widget.set_sensitive(false);

    // only allow changing the value via the value property if the user isn't currently dragging
    let is_being_dragged = Rc::new(RefCell::new(false));
    gtk_widget.connect_button_press_event(glib::clone!(@strong is_being_dragged => move |_, _| {
        *is_being_dragged.borrow_mut() = true;
        glib::Propagation::Proceed
    }));
    gtk_widget.connect_button_release_event(glib::clone!(@strong is_being_dragged => move |_, _| {
        *is_being_dragged.borrow_mut() = false;
        glib::Propagation::Proceed
    }));

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
        gtk_widget.set_sensitive(true);
        gtk_widget.add_events(gdk::EventMask::PROPERTY_CHANGE_MASK);
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
