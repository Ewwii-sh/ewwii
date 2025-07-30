#![allow(clippy::option_map_unit_fn)]

use iirhai::widgetnode::WidgetNode;
use rhai::Map;
// use super::{build_widget::BuilderArgs, circular_progressbar::*, run_command, transform::*};
use crate::{
    enum_parse, error_handling_ctx,
    util::{self, list_difference},
    widgets::{build_widget::build_gtk_widget},
};
use anyhow::{anyhow, Context, Result};
use codespan_reporting::diagnostic::Severity;
use ewwii_shared_util::Spanned;

use gdk::{ModifierType, NotifyType};
use glib::translate::FromGlib;
use gtk::{self, glib, prelude::*, DestDefaults, TargetEntry, TargetList};
use gtk::{gdk, pango};
use itertools::Itertools;
use once_cell::sync::Lazy;

use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::{HashMap, HashSet},
    rc::Rc,
    time::Duration,
};

use super::widget_definitions_helper::*;

pub(super) fn build_gtk_box(props: Map, children: Vec<WidgetNode>) -> Result<gtk::Box> {
    let orientation = props.get("orientation")
        .and_then(|v| v.clone().try_cast::<String>().ok())
        .map(|s| parse_orientation(&s)) // from widget_definitions_helper
        .transpose()?
        .unwrap_or(gtk::Orientation::Horizontal);

    let spacing = props.get("spacing")
        .and_then(|v| v.clone().try_cast::<i64>().ok())
        .unwrap_or(0) as i32;

    let space_evenly = props.get("space_evenly")
        .and_then(|v| v.clone().try_cast::<bool>().ok())
        .unwrap_or(true);

    let gtk_widget = gtk::Box::new(orientation, spacing);
    gtk_widget.set_homogeneous(space_evenly);

    for child in children {
        let child_widget = build_gtk_widget(child)?;
        gtk_widget.add(&child_widget);
    }

    Ok(gtk_widget)
}


pub(super) fn build_center_box(props: Map, children: Vec<WidgetNode>) -> Result<gtk::Box> {
    let gtk_widget = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    // def_widget!(bargs, _g, gtk_widget, {
    //     // @prop orientation - orientation of the centerbox. possible values: $orientation
    //     prop(orientation: as_string) { gtk_widget.set_orientation(parse_orientation(&orientation)?) },
    // });

    // match bargs.widget_use.children.len().cmp(&3) {
    //     Ordering::Less => {
    //         Err(DiagError(gen_diagnostic!("centerbox must contain exactly 3 elements", bargs.widget_use.span)).into())
    //     }
    //     Ordering::Greater => {
    //         let (_, additional_children) = bargs.widget_use.children.split_at(3);
    //         // we know that there is more than three children, so unwrapping on first and left here is fine.
    //         let first_span = additional_children.first().unwrap().span();
    //         let last_span = additional_children.last().unwrap().span();
    //         Err(DiagError(gen_diagnostic!("centerbox must contain exactly 3 elements, but got more", first_span.to(last_span)))
    //             .into())
    //     }
    //     Ordering::Equal => {
    //         let mut children = bargs.widget_use.children.iter().map(|child| {
    //             build_gtk_widget(
    //                 bargs.scope_graph,
    //                 bargs.widget_defs.clone(),
    //                 bargs.calling_scope,
    //                 child.clone(),
    //                 bargs.custom_widget_invocation.clone(),
    //             )
    //         });
    //         // we know that we have exactly three children here, so we can unwrap here.
    //         let (first, center, end) = children.next_tuple().unwrap();
    //         let (first, center, end) = (first?, center?, end?);
    //         gtk_widget.pack_start(&first, true, true, 0);
    //         gtk_widget.set_center_widget(Some(&center));
    //         gtk_widget.pack_end(&end, true, true, 0);
    //         first.show();
    //         center.show();
    //         end.show();
    //         Ok(gtk_widget)
    //     }
    // }
    Ok(gtk_widget)
}

pub(super) fn build_gtk_event_box(props: Map, children: Vec<WidgetNode>) -> Result<gtk::EventBox> {
    let gtk_widget = gtk::EventBox::new();

    // // Support :hover selector
    // gtk_widget.connect_enter_notify_event(|gtk_widget, evt| {
    //     if evt.detail() != NotifyType::Inferior {
    //         gtk_widget.set_state_flags(gtk::StateFlags::PRELIGHT, false);
    //     }
    //     glib::Propagation::Proceed
    // });

    // gtk_widget.connect_leave_notify_event(|gtk_widget, evt| {
    //     if evt.detail() != NotifyType::Inferior {
    //         gtk_widget.unset_state_flags(gtk::StateFlags::PRELIGHT);
    //     }
    //     glib::Propagation::Proceed
    // });

    // // Support :active selector
    // gtk_widget.connect_button_press_event(|gtk_widget, _| {
    //     gtk_widget.set_state_flags(gtk::StateFlags::ACTIVE, false);
    //     glib::Propagation::Proceed
    // });

    // gtk_widget.connect_button_release_event(|gtk_widget, _| {
    //     gtk_widget.unset_state_flags(gtk::StateFlags::ACTIVE);
    //     glib::Propagation::Proceed
    // });

    // def_widget!(bargs, _g, gtk_widget, {
    //     // @prop timeout - timeout of the command. Default: "200ms"
    //     // @prop onscroll - event to execute when the user scrolls with the mouse over the widget. The placeholder `{}` used in the command will be replaced with either `up` or `down`.
    //     prop(timeout: as_duration = Duration::from_millis(200), onscroll: as_string) {
    //         gtk_widget.add_events(gdk::EventMask::SCROLL_MASK);
    //         gtk_widget.add_events(gdk::EventMask::SMOOTH_SCROLL_MASK);
    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_scroll_event(move |_, evt| {
    //             let delta = evt.delta().1;
    //             if delta != 0f64 { // Ignore the first event https://bugzilla.gnome.org/show_bug.cgi?id=675959
    //                 run_command(timeout, &onscroll, &[if delta < 0f64 { "up" } else { "down" }]);
    //             }
    //             glib::Propagation::Proceed
    //         }));
    //     },
    //     // @prop timeout - timeout of the command. Default: "200ms"
    //     // @prop onhover - event to execute when the user hovers over the widget
    //     prop(timeout: as_duration = Duration::from_millis(200), onhover: as_string) {
    //         gtk_widget.add_events(gdk::EventMask::ENTER_NOTIFY_MASK);
    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_enter_notify_event(move |_, evt| {
    //             if evt.detail() != NotifyType::Inferior {
    //                 run_command(timeout, &onhover, &[evt.position().0, evt.position().1]);
    //             }
    //             glib::Propagation::Proceed
    //         }));
    //     },
    //     // @prop timeout - timeout of the command. Default: "200ms"
    //     // @prop onhoverlost - event to execute when the user losts hovers over the widget
    //     prop(timeout: as_duration = Duration::from_millis(200), onhoverlost: as_string) {
    //         gtk_widget.add_events(gdk::EventMask::LEAVE_NOTIFY_MASK);
    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_leave_notify_event(move |_, evt| {
    //             if evt.detail() != NotifyType::Inferior {
    //                 run_command(timeout, &onhoverlost, &[evt.position().0, evt.position().1]);
    //             }
    //             glib::Propagation::Proceed
    //         }));
    //     },
    //     // @prop cursor - Cursor to show while hovering (see [gtk3-cursors](https://docs.gtk.org/gdk3/ctor.Cursor.new_from_name.html) for possible names)
    //     prop(cursor: as_string) {
    //         gtk_widget.add_events(gdk::EventMask::ENTER_NOTIFY_MASK);
    //         gtk_widget.add_events(gdk::EventMask::LEAVE_NOTIFY_MASK);

    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_enter_notify_event(move |widget, _evt| {
    //             if _evt.detail() != NotifyType::Inferior {
    //                 let display = gdk::Display::default();
    //                 let gdk_window = widget.window();
    //                 if let (Some(display), Some(gdk_window)) = (display, gdk_window) {
    //                     gdk_window.set_cursor(gdk::Cursor::from_name(&display, &cursor).as_ref());
    //                 }
    //             }
    //             glib::Propagation::Proceed
    //         }));
    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_leave_notify_event(move |widget, _evt| {
    //             if _evt.detail() != NotifyType::Inferior {
    //                 let gdk_window = widget.window();
    //                 if let Some(gdk_window) = gdk_window {
    //                     gdk_window.set_cursor(None);
    //                 }
    //             }
    //             glib::Propagation::Proceed
    //         }));
    //     },
    //     // @prop timeout - timeout of the command. Default: "200ms"
    //     // @prop ondropped - Command to execute when something is dropped on top of this element. The placeholder `{}` used in the command will be replaced with the uri to the dropped thing.
    //     prop(timeout: as_duration = Duration::from_millis(200), ondropped: as_string) {
    //         gtk_widget.drag_dest_set(
    //             DestDefaults::ALL,
    //             &[
    //                 TargetEntry::new("text/uri-list", gtk::TargetFlags::OTHER_APP | gtk::TargetFlags::OTHER_WIDGET, 0),
    //                 TargetEntry::new("text/plain", gtk::TargetFlags::OTHER_APP | gtk::TargetFlags::OTHER_WIDGET, 0)
    //             ],
    //             gdk::DragAction::COPY,
    //         );
    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_drag_data_received(move |_, _, _x, _y, selection_data, _target_type, _timestamp| {
    //             if let Some(data) = selection_data.uris().first(){
    //                 run_command(timeout, &ondropped, &[data.to_string(), "file".to_string()]);
    //             } else if let Some(data) = selection_data.text(){
    //                 run_command(timeout, &ondropped, &[data.to_string(), "text".to_string()]);
    //             }
    //         }));
    //     },

    //     // @prop dragvalue - URI that will be provided when dragging from this widget
    //     // @prop dragtype - Type of value that should be dragged from this widget. Possible values: $dragtype
    //     prop(dragvalue: as_string, dragtype: as_string = "file") {
    //         let dragtype = parse_dragtype(&dragtype)?;
    //         if dragvalue.is_empty() {
    //             gtk_widget.drag_source_unset();
    //         } else {
    //             let target_entry = match dragtype {
    //                 DragEntryType::File => TargetEntry::new("text/uri-list", gtk::TargetFlags::OTHER_APP | gtk::TargetFlags::OTHER_WIDGET, 0),
    //                 DragEntryType::Text => TargetEntry::new("text/plain", gtk::TargetFlags::OTHER_APP | gtk::TargetFlags::OTHER_WIDGET, 0),
    //             };
    //             gtk_widget.drag_source_set(
    //                 ModifierType::BUTTON1_MASK,
    //                 &[target_entry.clone()],
    //                 gdk::DragAction::COPY | gdk::DragAction::MOVE,
    //             );
    //             gtk_widget.drag_source_set_target_list(Some(&TargetList::new(&[target_entry])));
    //         }

    //         connect_signal_handler!(gtk_widget, if !dragvalue.is_empty(), gtk_widget.connect_drag_data_get(move |_, _, data, _, _| {
    //             match dragtype {
    //                 DragEntryType::File => data.set_uris(&[&dragvalue]),
    //                 DragEntryType::Text => data.set_text(&dragvalue),
    //             };
    //         }));
    //     },
    //     prop(
    //         // @prop timeout - timeout of the command. Default: "200ms"
    //         timeout: as_duration = Duration::from_millis(200),
    //         // @prop onclick - command to run when the widget is clicked
    //         onclick: as_string = "",
    //         // @prop onmiddleclick - command to run when the widget is middleclicked
    //         onmiddleclick: as_string = "",
    //         // @prop onrightclick - command to run when the widget is rightclicked
    //         onrightclick: as_string = ""
    //     ) {
    //         gtk_widget.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_button_release_event(move |_, evt| {
    //             match evt.button() {
    //                 1 => run_command(timeout, &onclick, &[] as &[&str]),
    //                 2 => run_command(timeout, &onmiddleclick, &[] as &[&str]),
    //                 3 => run_command(timeout, &onrightclick, &[] as &[&str]),
    //                 _ => {},
    //             }
    //             glib::Propagation::Proceed
    //         }));
    //     }
    // });
    Ok(gtk_widget)
}

pub(super) fn build_graph(props: Map) -> Result<super::graph::Graph> {
    let w = super::graph::Graph::new();
    // def_widget!(bargs, _g, w, {
    //     // @prop value - the value, between 0 - 100
    //     prop(value: as_f64) {
    //         if value.is_nan() || value.is_infinite() {
    //             return Err(DiagError(gen_diagnostic!(
    //                 format!("Graph's value should never be NaN or infinite")
    //             )).into());
    //         }
    //         w.set_property("value", value);
    //     },
    //     // @prop thickness - the thickness of the line
    //     prop(thickness: as_f64) { w.set_property("thickness", thickness); },
    //     // @prop time-range - the range of time to show
    //     prop(time_range: as_duration) { w.set_property("time-range", time_range.as_millis() as u64); },
    //     // @prop min - the minimum value to show (defaults to 0 if value_max is provided)
    //     // @prop max - the maximum value to show
    //     prop(min: as_f64 = 0, max: as_f64 = 100) {
    //         if min > max {
    //             return Err(DiagError(gen_diagnostic!(
    //                 format!("Graph's min ({min}) should never be higher than max ({max})")
    //             )).into());
    //         }
    //         w.set_property("min", min);
    //         w.set_property("max", max);
    //     },
    //     // @prop dynamic - whether the y range should dynamically change based on value
    //     prop(dynamic: as_bool) { w.set_property("dynamic", dynamic); },
    //     // @prop line-style - changes the look of the edges in the graph. Values: "miter" (default), "round",
    //     // "bevel"
    //     prop(line_style: as_string) { w.set_property("line-style", line_style); },
    //     // @prop flip-x - whether the x axis should go from high to low
    //     prop(flip_x: as_bool) { w.set_property("flip-x", flip_x); },
    //     // @prop flip-y - whether the y axis should go from high to low
    //     prop(flip_y: as_bool) { w.set_property("flip-y", flip_y); },
    //     // @prop vertical - if set to true, the x and y axes will be exchanged
    //     prop(vertical: as_bool) { w.set_property("vertical", vertical); },
    // });
    Ok(w)
}

pub(super) fn build_gtk_progress(props: Map) -> Result<gtk::ProgressBar> {
    let gtk_widget = gtk::ProgressBar::new();
    // def_widget!(bargs, _g, gtk_widget, {
    //     // @prop flipped - flip the direction
    //     prop(flipped: as_bool) { gtk_widget.set_inverted(flipped) },

    //     // @prop value - value of the progress bar (between 0-100)
    //     prop(value: as_f64) { gtk_widget.set_fraction(value / 100f64) },

    //     // @prop orientation - orientation of the progress bar. possible values: $orientation
    //     prop(orientation: as_string) { gtk_widget.set_orientation(parse_orientation(&orientation)?) },
    // });

    Ok(gtk_widget)
}

pub(super) fn build_gtk_image(props: Map) -> Result<gtk::Image> {
    let gtk_widget = gtk::Image::new();
    // def_widget!(bargs, _g, gtk_widget, {
    //     // @prop path - path to the image file
    //     // @prop image-width - width of the image
    //     // @prop image-height - height of the image
    //     // @prop preserve-aspect-ratio - whether to keep the aspect ratio when resizing an image. Default: true, false doesn't work for all image types
    //     // @prop fill-svg - sets the color of svg images
    //     prop(path: as_string, image_width: as_i32 = -1, image_height: as_i32 = -1, preserve_aspect_ratio: as_bool = true, fill_svg: as_string = "") {
    //         if !path.ends_with(".svg") && !fill_svg.is_empty() {
    //             log::warn!("Fill attribute ignored, file is not an svg image");
    //         }

    //         if path.ends_with(".gif") {
    //             let pixbuf_animation = gtk::gdk_pixbuf::PixbufAnimation::from_file(std::path::PathBuf::from(path))?;
    //             gtk_widget.set_from_animation(&pixbuf_animation);
    //         } else {
    //             let pixbuf;
    //             // populate the pixel buffer
    //             if path.ends_with(".svg") && !fill_svg.is_empty() {
    //                 let svg_data = std::fs::read_to_string(std::path::PathBuf::from(path.clone()))?;
    //                 // The fastest way to add/change fill color
    //                 let svg_data = if svg_data.contains("fill=") {
    //                     let reg = regex::Regex::new(r#"fill="[^"]*""#)?;
    //                     reg.replace(&svg_data, &format!("fill=\"{}\"", fill_svg))
    //                 } else {
    //                     let reg = regex::Regex::new(r"<svg")?;
    //                     reg.replace(&svg_data, &format!("<svg fill=\"{}\"", fill_svg))
    //                 };
    //                 let stream = gtk::gio::MemoryInputStream::from_bytes(&gtk::glib::Bytes::from(svg_data.as_bytes()));
    //                 pixbuf = gtk::gdk_pixbuf::Pixbuf::from_stream_at_scale(&stream, image_width, image_height, preserve_aspect_ratio, None::<&gtk::gio::Cancellable>)?;
    //                 stream.close(None::<&gtk::gio::Cancellable>)?;
    //             } else {
    //                 pixbuf = gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(std::path::PathBuf::from(path), image_width, image_height, preserve_aspect_ratio)?;
    //             }
    //             gtk_widget.set_from_pixbuf(Some(&pixbuf));
    //         }
    //     },
    //     // @prop icon - name of a theme icon
    //     // @prop icon-size - size of the theme icon
    //     prop(icon: as_string, icon_size: as_string = "button") {
    //         gtk_widget.set_from_icon_name(Some(&icon), parse_icon_size(&icon_size)?);
    //     },
    // });
    Ok(gtk_widget)
}

pub(super) fn build_gtk_button(props: Map, children: Vec<WidgetNode>) -> Result<gtk::Button> {
    let gtk_widget = gtk::Button::new();

    // def_widget!(bargs, _g, gtk_widget, {
    //     prop(
    //         // @prop timeout - timeout of the command. Default: "200ms"
    //         timeout: as_duration = Duration::from_millis(200),
    //         // @prop onclick - command to run when the button is activated either by leftclicking or keyboard
    //         onclick: as_string = "",
    //         // @prop onmiddleclick - command to run when the button is middleclicked
    //         onmiddleclick: as_string = "",
    //         // @prop onrightclick - command to run when the button is rightclicked
    //         onrightclick: as_string = ""
    //     ) {
    //         // animate button upon right-/middleclick (if gtk theme supports it)
    //         // since we do this, we can't use `connect_clicked` as that would always run `onclick` as well
    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_button_press_event(move |button, _| {
    //             button.emit_activate();
    //             glib::Propagation::Proceed
    //         }));
    //         let onclick_ = onclick.clone();
    //         // mouse click events
    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_button_release_event(move |_, evt| {
    //             match evt.button() {
    //                 1 => run_command(timeout, &onclick, &[] as &[&str]),
    //                 2 => run_command(timeout, &onmiddleclick, &[] as &[&str]),
    //                 3 => run_command(timeout, &onrightclick, &[] as &[&str]),
    //                 _ => {},
    //             }
    //             glib::Propagation::Proceed
    //         }));
    //         // keyboard events
    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_key_release_event(move |_, evt| {
    //             match evt.scancode() {
    //                 // return
    //                 36 => run_command(timeout, &onclick_, &[] as &[&str]),
    //                 // space
    //                 65 => run_command(timeout, &onclick_, &[] as &[&str]),
    //                 _ => {},
    //             }
    //             glib::Propagation::Proceed
    //         }));
    //     }
    // });
    Ok(gtk_widget)
}

pub(super) fn build_gtk_label(props: Map) -> Result<gtk::Label> {
    let gtk_widget = gtk::Label::new(None);

    // def_widget!(bargs, _g, gtk_widget, {
    //     // @prop text - the text to display
    //     // @prop truncate - whether to truncate text (or pango markup). If `show-truncated` is `false`, or if `limit-width` has a value, this property has no effect and truncation is enabled.
    //     // @prop limit-width - maximum count of characters to display
    //     // @prop truncate-left - whether to truncate on the left side
    //     // @prop show-truncated - show whether the text was truncated. Disabling it will also disable dynamic truncation (the labels won't be truncated more than `limit-width`, even if there is not enough space for them), and will completly disable truncation on pango markup.
    //     // @prop unindent - whether to remove leading spaces
    //     prop(text: as_string, truncate: as_bool = false, limit_width: as_i32 = i32::MAX, truncate_left: as_bool = false, show_truncated: as_bool = true, unindent: as_bool = true) {
    //         let text = if show_truncated {
    //             // gtk does weird thing if we set max_width_chars to i32::MAX
    //             if limit_width == i32::MAX {
    //                 gtk_widget.set_max_width_chars(-1);
    //             } else {
    //                 gtk_widget.set_max_width_chars(limit_width);
    //             }
    //             if truncate || limit_width != i32::MAX {
    //                 if truncate_left {
    //                     gtk_widget.set_ellipsize(pango::EllipsizeMode::Start);
    //                 } else {
    //                     gtk_widget.set_ellipsize(pango::EllipsizeMode::End);
    //                 }
    //             } else {
    //                 gtk_widget.set_ellipsize(pango::EllipsizeMode::None);
    //             }

    //             text
    //         } else {
    //             gtk_widget.set_ellipsize(pango::EllipsizeMode::None);

    //             let limit_width = limit_width as usize;
    //             let char_count = text.chars().count();
    //             if char_count > limit_width {
    //                 if truncate_left {
    //                     text.chars().skip(char_count - limit_width).collect()
    //                 } else {
    //                     text.chars().take(limit_width).collect()
    //                 }
    //             } else {
    //                 text
    //             }
    //         };

    //         let text = unescape::unescape(&text).context(format!("Failed to unescape label text {}", &text))?;
    //         let text = if unindent { util::unindent(&text) } else { text };
    //         gtk_widget.set_text(&text);
    //     },
    //     // @prop markup - Pango markup to display
    //     // @prop truncate - whether to truncate text (or pango markup). If `show-truncated` is `false`, or if `limit-width` has a value, this property has no effect and truncation is enabled.
    //     // @prop limit-width - maximum count of characters to display
    //     // @prop truncate-left - whether to truncate on the left side
    //     // @prop show-truncated - show whether the text was truncated. Disabling it will also disable dynamic truncation (the labels won't be truncated more than `limit-width`, even if there is not enough space for them), and will completly disable truncation on pango markup.
    //     prop(markup: as_string, truncate: as_bool = false, limit_width: as_i32 = i32::MAX, truncate_left: as_bool = false, show_truncated: as_bool = true) {
    //         if (truncate || limit_width != i32::MAX) && show_truncated {
    //             // gtk does weird thing if we set max_width_chars to i32::MAX
    //             if limit_width == i32::MAX {
    //                 gtk_widget.set_max_width_chars(-1);
    //             } else {
    //                 gtk_widget.set_max_width_chars(limit_width);
    //             }

    //             if truncate_left {
    //                 gtk_widget.set_ellipsize(pango::EllipsizeMode::Start);
    //             } else {
    //                 gtk_widget.set_ellipsize(pango::EllipsizeMode::End);
    //             }
    //         } else {
    //             gtk_widget.set_ellipsize(pango::EllipsizeMode::None);
    //         }

    //         gtk_widget.set_markup(&markup);
    //     },
    //     // @prop wrap - Wrap the text. This mainly makes sense if you set the width of this widget.
    //     prop(wrap: as_bool) { gtk_widget.set_line_wrap(wrap) },
    //     // @prop angle - the angle of rotation for the label (between 0 - 360)
    //     prop(angle: as_f64 = 0) { gtk_widget.set_angle(angle) },
    //     // @prop gravity - the gravity of the string (south, east, west, north, auto). Text will want to face the direction of gravity.
    //     prop(gravity: as_string = "south") {
    //         gtk_widget.pango_context().set_base_gravity(parse_gravity(&gravity)?);
    //     },
    //     // @prop xalign - the alignment of the label text on the x axis (between 0 - 1, 0 -> left, 0.5 -> center, 1 -> right)
    //     prop(xalign: as_f64 = 0.5) { gtk_widget.set_xalign(xalign as f32) },
    //     // @prop yalign - the alignment of the label text on the y axis (between 0 - 1, 0 -> bottom, 0.5 -> center, 1 -> top)
    //     prop(yalign: as_f64 = 0.5) { gtk_widget.set_yalign(yalign as f32) },
    //     // @prop justify - the justification of the label text (left, right, center, fill)
    //     prop(justify: as_string = "left") {
    //         gtk_widget.set_justify(parse_justification(&justify)?);
    //     },
    //     // @prop wrap-mode - how text is wrapped. possible options: $wrap_mode
    //     prop(wrap_mode: as_string = "word") {
    //         gtk_widget.set_wrap_mode(parse_wrap_mode(&wrap_mode)?);
    //     },
    //     // @prop lines - maximum number of lines to display (only works when `limit-width` has a value). A value of -1 (default) disables the limit.
    //     prop(lines: as_i32 = -1) {
    //         gtk_widget.set_lines(lines);
    //     }
    // });
    Ok(gtk_widget)
}

pub(super) fn build_gtk_input(props: Map) -> Result<gtk::Entry> {
    let gtk_widget = gtk::Entry::new();
    // def_widget!(bargs, _g, gtk_widget, {
    //     // @prop value - the content of the text field
    //     prop(value: as_string) {
    //         gtk_widget.set_text(&value);
    //     },
    //     // @prop onchange - Command to run when the text changes. The placeholder `{}` will be replaced by the value
    //     // @prop timeout - timeout of the command. Default: "200ms"
    //     prop(timeout: as_duration = Duration::from_millis(200), onchange: as_string) {
    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_changed(move |gtk_widget| {
    //             run_command(timeout, &onchange, &[gtk_widget.text().to_string()]);
    //         }));
    //     },
    //     // @prop onaccept - Command to run when the user hits return in the input field. The placeholder `{}` will be replaced by the value
    //     // @prop timeout - timeout of the command. Default: "200ms"
    //     prop(timeout: as_duration = Duration::from_millis(200), onaccept: as_string) {
    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_activate(move |gtk_widget| {
    //             run_command(timeout, &onaccept, &[gtk_widget.text().to_string()]);
    //         }));
    //     },
    //     // @prop password - if the input is obscured
    //     prop(password: as_bool = false) {
    //         gtk_widget.set_visibility(!password);
    //     }
    // });
    Ok(gtk_widget)
}

pub(super) fn build_gtk_calendar(props: Map) -> Result<gtk::Calendar> {
    let gtk_widget = gtk::Calendar::new();
    // def_widget!(bargs, _g, gtk_widget, {
    //     // @prop day - the selected day
    //     prop(day: as_f64) {
    //         if !(1f64..=31f64).contains(&day) {
    //             log::warn!("Calendar day is not a number between 1 and 31");
    //         } else {
    //             gtk_widget.set_day(day as i32)
    //         }
    //     },
    //     // @prop month - the selected month
    //     prop(month: as_f64) {
    //         if !(1f64..=12f64).contains(&month) {
    //             log::warn!("Calendar month is not a number between 1 and 12");
    //         } else {
    //             gtk_widget.set_month(month as i32 - 1)
    //         }
    //     },
    //     // @prop year - the selected year
    //     prop(year: as_f64) { gtk_widget.set_year(year as i32) },
    //     // @prop show-details - show details
    //     prop(show_details: as_bool) { gtk_widget.set_show_details(show_details) },
    //     // @prop show-heading - show heading line
    //     prop(show_heading: as_bool) { gtk_widget.set_show_heading(show_heading) },
    //     // @prop show-day-names - show names of days
    //     prop(show_day_names: as_bool) { gtk_widget.set_show_day_names(show_day_names) },
    //     // @prop show-week-numbers - show week numbers
    //     prop(show_week_numbers: as_bool) { gtk_widget.set_show_week_numbers(show_week_numbers) },
    //     // @prop onclick - command to run when the user selects a date. The `{0}` placeholder will be replaced by the selected day, `{1}` will be replaced by the month, and `{2}` by the year.
    //     // @prop timeout - timeout of the command. Default: "200ms"
    //     prop(timeout: as_duration = Duration::from_millis(200), onclick: as_string) {
    //         connect_signal_handler!(gtk_widget, gtk_widget.connect_day_selected(move |w| {
    //             run_command(
    //                 timeout,
    //                 &onclick,
    //                 &[w.day(), w.month(), w.year()]
    //             )
    //         }));
    //     }

    // });

    Ok(gtk_widget)
}

pub(super) fn build_gtk_revealer(props: Map, children: Vec<WidgetNode>) -> Result<gtk::Revealer> {
    let gtk_widget = gtk::Revealer::new();
    // def_widget!(bargs, _g, gtk_widget, {
    //     // @prop transition - the name of the transition. Possible values: $transition
    //     prop(transition: as_string = "crossfade") { gtk_widget.set_transition_type(parse_revealer_transition(&transition)?); },
    //     // @prop reveal - sets if the child is revealed or not
    //     prop(reveal: as_bool) { gtk_widget.set_reveal_child(reveal); },
    //     // @prop duration - the duration of the reveal transition. Default: "500ms"
    //     prop(duration: as_duration = Duration::from_millis(500)) { gtk_widget.set_transition_duration(duration.as_millis() as u32); },
    // });
    Ok(gtk_widget)
}

pub(super) fn build_gtk_scrolledwindow(props: Map, children: Vec<WidgetNode>) -> Result<gtk::ScrolledWindow> {
    // I don't have single idea of what those two generics are supposed to be, but this works.
    let gtk_widget = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);

    // def_widget!(bargs, _g, gtk_widget, {
    //     // @prop hscroll - scroll horizontally
    //     // @prop vscroll - scroll vertically
    //     prop(hscroll: as_bool = true, vscroll: as_bool = true) {
    //         gtk_widget.set_policy(
    //             if hscroll { gtk::PolicyType::Automatic } else { gtk::PolicyType::Never },
    //             if vscroll { gtk::PolicyType::Automatic } else { gtk::PolicyType::Never },
    //         )
    //     },
    // });

    Ok(gtk_widget)
}