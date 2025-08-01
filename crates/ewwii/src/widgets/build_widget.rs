use anyhow::Result;
// use codespan_reporting::diagnostic::Severity;
// use ewwii_shared_util::{AttrName, Spanned};
use gtk::{
    gdk::prelude::Cast,
    // prelude::{BoxExt, ContainerExt, WidgetExt},
    // Orientation,
};
// use maplit::hashmap;
// use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    config::WindowDefinition,
    // gen_diagnostic_macro,
    // error_handling_ctx,
    // dynval::DynVal,
    widgets::widget_definitions::*,
};

use iirhai::widgetnode::WidgetNode;

/// Widget input allows us to pass either a widgetnode or a window_def
/// this is important to make build_gtk_widget standalone without having to
/// make build_gtk_widget_from_node public
pub enum WidgetInput {
    Node(WidgetNode),
    Window(WindowDefinition),
}

pub fn build_gtk_widget(input: WidgetInput) -> Result<gtk::Widget> {
    let node = match input {
        WidgetInput::Node(n) => n,
        WidgetInput::Window(w) => w.root_widget,
    };
    build_gtk_widget_from_node(node)
}

// TODO: implement the commented lines
fn build_gtk_widget_from_node(root_node: WidgetNode) -> Result<gtk::Widget> {
    let root_node2 = root_node.clone();
    let gtk_widget = match root_node {
        WidgetNode::Box { props, children } => build_gtk_box(props, children)?.upcast(),
        WidgetNode::CenterBox { props, children } => build_center_box(props, children)?.upcast(),
        WidgetNode::EventBox { props, children } => build_gtk_event_box(props, children)?.upcast(),
        // WIDGET_NAME_TOOLTIP => build_tooltip(node)?.upcast(),
        // WIDGET_NAME_CIRCULAR_PROGRESS => build_circular_progress_bar(node)?.upcast(),
        WidgetNode::Graph { props } => build_graph(props)?.upcast(),
        // WIDGET_NAME_TRANSFORM => build_transform(node)?.upcast(),
        WidgetNode::Slider { props } => build_gtk_scale(props)?.upcast(),
        WidgetNode::Progress { props } => build_gtk_progress(props)?.upcast(),
        WidgetNode::Image { props } => build_gtk_image(props)?.upcast(),
        WidgetNode::Button { props } => build_gtk_button(props)?.upcast(),
        WidgetNode::Label { props } => build_gtk_label(props)?.upcast(),
        // WIDGET_NAME_LITERAL => build_gtk_literal(node)?.upcast(),
        WidgetNode::Input { props } => build_gtk_input(props)?.upcast(),
        WidgetNode::Calendar { props } => build_gtk_calendar(props)?.upcast(),
        // WIDGET_NAME_COLOR_BUTTON => build_gtk_color_button(node)?.upcast(),
        // WIDGET_NAME_EXPANDER => build_gtk_expander(node)?.upcast(),
        // WIDGET_NAME_COLOR_CHOOSER => build_gtk_color_chooser(node)?.upcast(),
        // WIDGET_NAME_COMBO_BOX_TEXT => build_gtk_combo_box_text(node)?.upcast(),
        // WIDGET_NAME_CHECKBOX => build_gtk_checkbox(node)?.upcast(),
        WidgetNode::Revealer { props, children } => build_gtk_revealer(props, children)?.upcast(),
        WidgetNode::Scroll { props, children } => build_gtk_scrolledwindow(props, children)?.upcast(),
        // WIDGET_NAME_OVERLAY => build_gtk_overlay(node)?.upcast(),
        // WIDGET_NAME_STACK => build_gtk_stack(node)?.upcast(),
        // WIDGET_NAME_SYSTRAY => build_systray(node)?.upcast(),
        unknown => {
            return Err(anyhow::anyhow!("Cannot build GTK widget from node: {:?}", unknown));
        }
    };
    resolve_rhai_widget_attrs(root_node2, &gtk_widget)?;
    Ok(gtk_widget)
}
