use anyhow::{Context, Result};
use ewwii_shared_utils::ast::WidgetNode;
use ewwii_shared_utils::prop::{Property, PropertyMap};
use nbcl::ast::resolved::ResolvedNode;

macro_rules! handle_primitive {
    ($variant:ident, $node:expr) => {{
        let mut props = PropertyMap::from_nbcl($node.props);
        if let Some(id) = $node.id {
            props.insert("widget_name", Property::String(id));
        }
        WidgetNode::$variant { props }
    }};
}

macro_rules! handle_with_children {
    ($variant:ident, $node:expr) => {{
        let mut props = PropertyMap::from_nbcl($node.props);
        let children = to_widgetnode($node.children)?;
        if let Some(id) = $node.id {
            props.insert("widget_name", Property::String(id));
        }
        WidgetNode::$variant { props, children }
    }};
}

pub fn to_widgetnode(nodes: Vec<ResolvedNode>) -> Result<Vec<WidgetNode>> {
    let mut widgets = Vec::new();

    for node in nodes {
        let widget = match node.type_name.as_ref() {
            // Primitives
            "Label" => handle_primitive!(Label, node),
            "Button" => handle_primitive!(Button, node),
            "Image" => handle_primitive!(Image, node),
            "Input" => handle_primitive!(Input, node),
            "Progress" => handle_primitive!(Progress, node),
            "ComboBoxText" => handle_primitive!(ComboBoxText, node),
            "Scale" => handle_primitive!(Scale, node),
            "Checkbox" => handle_primitive!(Checkbox, node),
            "Calendar" => handle_primitive!(Calendar, node),
            "Graph" => handle_primitive!(Graph, node),
            "Transform" => handle_primitive!(Transform, node),
            "CircularProgress" => handle_primitive!(CircularProgress, node),
            "ColorButton" => handle_primitive!(ColorButton, node),
            "ColorChooser" => handle_primitive!(ColorChooser, node),

            // w/ children
            "Box" => handle_with_children!(Box, node),
            "FlowBox" => handle_with_children!(FlowBox, node),
            "Expander" => handle_with_children!(Expander, node),
            "Revealer" => handle_with_children!(Revealer, node),
            "Scroll" => handle_with_children!(Scroll, node),
            "OverLay" => handle_with_children!(OverLay, node),
            "AspectFrame" => handle_with_children!(AspectFrame, node),
            "Stack" => handle_with_children!(Stack, node),
            "EventBox" => handle_with_children!(EventBox, node),
            "ToolTip" => handle_with_children!(ToolTip, node),
            "Animation" => handle_with_children!(Animation, node),

            // Special
            "GtkUI" => handle_primitive!(GtkUI, node),

            // Main
            "Poll" => {
                let name = node.id.with_context(|| "Poll has no <id>".to_string())?;
                let props = PropertyMap::from_nbcl(node.props);

                WidgetNode::Poll { var: name, props }
            }

            "Listen" => {
                let name = node.id.with_context(|| "Listen has no <id>".to_string())?;
                let props = PropertyMap::from_nbcl(node.props);

                WidgetNode::Listen { var: name, props }
            }

            "Script" => {
                let props = PropertyMap::from_nbcl(node.props);
                WidgetNode::Script { props }
            }

            "Window" => {
                let name = node.id.with_context(|| "Window has no <id>".to_string())?;
                let props = PropertyMap::from_nbcl(node.props);
                // we ensured that only 1 child is
                // provided in builtins.rs file.
                let child = to_widgetnode(node.children)?.remove(0);

                WidgetNode::DefWindow { name, props, node: Box::new(child) }
            }
            other => {
                anyhow::bail!("Unknown widget found: {other}");
            }
        };
        widgets.push(widget);
    }

    Ok(widgets)
}
