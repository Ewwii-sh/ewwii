use anyhow::Result;
use gtk4::gdk::prelude::Cast;

use crate::config::WindowDefinition;
use crate::widgets::widget_definitions::*;

use rhai_impl::ast::WidgetNode;

/// Widget input allows us to pass either a widgetnode or a window_def
/// this is important to make build_gtk_widget standalone without having to
/// make build_gtk_widget_from_node public
pub enum WidgetInput<'a> {
    Node(WidgetNode),
    BorrowedNode(&'a WidgetNode),
    Window(WindowDefinition),
}

pub fn build_gtk_widget<'a>(
    input: &'a WidgetInput<'a>,
    widget_reg: &mut WidgetRegistry,
) -> Result<gtk4::Widget> {
    let node: &'a WidgetNode = match input {
        WidgetInput::Node(n) => n,
        WidgetInput::BorrowedNode(n) => n,
        WidgetInput::Window(w) => w.root_widget.as_ref(),
    };
    build_gtk_widget_from_node(node, widget_reg)
}

// TODO: implement the commented lines
fn build_gtk_widget_from_node(
    root_node: &WidgetNode,
    widget_reg: &mut WidgetRegistry,
) -> Result<gtk4::Widget> {
    /*
        When a a new widget is added to the build process,
        make sure to update get_id_to_props_map() found in
        `iirhai/widgetnode.rs`. It is crutial to presrve
        dynamic update system in ewwii.
    */

    let gtk_widget = match root_node {
        WidgetNode::Box { props, children } => build_gtk_box(props, children, widget_reg)?.upcast(),
        WidgetNode::FlowBox { props, children } => {
            build_gtk_flowbox(props, children, widget_reg)?.upcast()
        }
        WidgetNode::EventBox { props, children } => {
            build_event_box(props, children, widget_reg)?.upcast()
        }
        WidgetNode::ToolTip { props, children } => {
            build_tooltip(props, children, widget_reg)?.upcast()
        }
        WidgetNode::LocalBind { props, children } => {
            build_localbind_util(props, children, widget_reg)?.upcast()
        }
        WidgetNode::CircularProgress { props } => {
            build_circular_progress_bar(props, widget_reg)?.upcast()
        }
        // WidgetNode::Graph { props } => build_graph(props, widget_reg)?.upcast(),
        // WidgetNode::Transform { props } => build_transform(props, widget_reg)?.upcast(),
        WidgetNode::Slider { props } => build_gtk_scale(props, widget_reg)?.upcast(),
        WidgetNode::Progress { props } => build_gtk_progress(props, widget_reg)?.upcast(),
        WidgetNode::Image { props } => build_image(props, widget_reg)?.upcast(),
        WidgetNode::Icon { props } => build_icon(props, widget_reg)?.upcast(),
        WidgetNode::Button { props } => build_gtk_button(props, widget_reg)?.upcast(),
        WidgetNode::Label { props } => build_gtk_label(props, widget_reg)?.upcast(),
        // WIDGET_NAME_LITERAL => build_gtk_literal(node)?.upcast(),
        WidgetNode::Input { props } => build_gtk_input(props, widget_reg)?.upcast(),
        WidgetNode::Calendar { props } => build_gtk_calendar(props, widget_reg)?.upcast(),
        WidgetNode::ColorButton { props } => build_gtk_color_button(props, widget_reg)?.upcast(),
        WidgetNode::Expander { props, children } => {
            build_gtk_expander(props, children, widget_reg)?.upcast()
        }
        WidgetNode::ColorChooser { props } => build_gtk_color_chooser(props, widget_reg)?.upcast(),
        WidgetNode::ComboBoxText { props } => build_gtk_combo_box_text(props, widget_reg)?.upcast(),
        WidgetNode::Checkbox { props } => build_gtk_checkbox(props, widget_reg)?.upcast(),
        WidgetNode::Revealer { props, children } => {
            build_gtk_revealer(props, children, widget_reg)?.upcast()
        }
        WidgetNode::Scroll { props, children } => {
            build_gtk_scrolledwindow(props, children, widget_reg)?.upcast()
        }
        WidgetNode::OverLay { props, children } => {
            build_gtk_overlay(props, children, widget_reg)?.upcast()
        }
        WidgetNode::Stack { props, children } => {
            build_gtk_stack(props, children, widget_reg)?.upcast()
        }
        // WIDGET_NAME_SYSTRAY => build_systray(node)?.upcast(),
        unknown => {
            return Err(anyhow::anyhow!("Cannot build GTK widget from node: {:?}", unknown));
        }
    };

    Ok(gtk_widget)
}
