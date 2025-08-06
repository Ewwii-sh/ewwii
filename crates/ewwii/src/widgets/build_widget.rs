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
        WidgetNode::ToolTip { children } => build_tooltip(children)?.upcast(),
        WidgetNode::CircularProgress { props } => build_circular_progress_bar(props)?.upcast(),
        WidgetNode::Graph { props } => build_graph(props)?.upcast(),
        WidgetNode::Transform { props } => build_transform(props)?.upcast(),
        WidgetNode::Slider { props } => build_gtk_scale(props)?.upcast(),
        WidgetNode::Progress { props } => build_gtk_progress(props)?.upcast(),
        WidgetNode::Image { props } => build_gtk_image(props)?.upcast(),
        WidgetNode::Button { props } => build_gtk_button(props)?.upcast(),
        WidgetNode::Label { props } => build_gtk_label(props)?.upcast(),
        // WIDGET_NAME_LITERAL => build_gtk_literal(node)?.upcast(),
        WidgetNode::Input { props } => build_gtk_input(props)?.upcast(),
        WidgetNode::Calendar { props } => build_gtk_calendar(props)?.upcast(),
        WidgetNode::ColorButton { props } => build_gtk_color_button(props)?.upcast(),
        WidgetNode::Expander { props, children } => build_gtk_expander(props, children)?.upcast(),
        WidgetNode::ColorChooser { props } => build_gtk_color_chooser(props)?.upcast(),
        WidgetNode::ComboBoxText { props } => build_gtk_combo_box_text(props)?.upcast(),
        WidgetNode::Checkbox { props } => build_gtk_checkbox(props)?.upcast(),
        WidgetNode::Revealer { props, children } => build_gtk_revealer(props, children)?.upcast(),
        WidgetNode::Scroll { props, children } => build_gtk_scrolledwindow(props, children)?.upcast(),
        WidgetNode::OverLay { children } => build_gtk_overlay(children)?.upcast(),
        WidgetNode::Stack { props, children } => build_gtk_stack(props, children)?.upcast(),
        // WIDGET_NAME_SYSTRAY => build_systray(node)?.upcast(),
        unknown => {
            return Err(anyhow::anyhow!("Cannot build GTK widget from node: {:?}", unknown));
        }
    };
    resolve_rhai_widget_attrs(root_node2, &gtk_widget)?;
    Ok(gtk_widget)
}
