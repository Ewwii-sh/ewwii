use crate::ast::WidgetNode;
use rhai::{Dynamic, Map};

impl WidgetNode {
    /// A very important implementation of [`WidgetNode`].
    /// This function implements dyn_id property to widgets.
    pub fn setup_dyn_ids(&self, parent_path: &str) -> Self {
        // fn to assign dyn_id to a node
        fn with_dyn_id(mut props: Map, dyn_id: &str) -> Map {
            props.insert("dyn_id".into(), Dynamic::from(dyn_id.to_string()));
            props
        }

        // fn to process children of a container node
        fn process_children(
            children: &[WidgetNode],
            parent_path: &str,
            kind: &str,
        ) -> Vec<WidgetNode> {
            children
                .iter()
                .enumerate()
                .map(|(idx, child)| {
                    let child_path = format!("{}_{}_{}", parent_path, kind, idx);
                    child.setup_dyn_ids(&child_path)
                })
                .collect()
        }

        match self {
            WidgetNode::DefWindow { name, props, node } => WidgetNode::DefWindow {
                name: name.clone(),
                props: props.clone(),
                node: Box::new(node.setup_dyn_ids(name)),
            },

            // == Containers with children ==
            WidgetNode::Box { props, children } => WidgetNode::Box {
                props: with_dyn_id(props.clone(), parent_path),
                children: process_children(children, parent_path, "box"),
            },
            WidgetNode::FlowBox { props, children } => WidgetNode::FlowBox {
                props: with_dyn_id(props.clone(), parent_path),
                children: process_children(children, parent_path, "flowbox"),
            },
            WidgetNode::Expander { props, children } => WidgetNode::Expander {
                props: with_dyn_id(props.clone(), parent_path),
                children: process_children(children, parent_path, "expander"),
            },
            WidgetNode::Revealer { props, children } => WidgetNode::Revealer {
                props: with_dyn_id(props.clone(), parent_path),
                children: process_children(children, parent_path, "revealer"),
            },
            WidgetNode::Scroll { props, children } => WidgetNode::Scroll {
                props: with_dyn_id(props.clone(), parent_path),
                children: process_children(children, parent_path, "scroll"),
            },
            WidgetNode::OverLay { props, children } => WidgetNode::OverLay {
                props: with_dyn_id(props.clone(), parent_path),
                children: process_children(children, parent_path, "overlay"),
            },
            WidgetNode::Stack { props, children } => WidgetNode::Stack {
                props: with_dyn_id(props.clone(), parent_path),
                children: process_children(children, parent_path, "stack"),
            },
            WidgetNode::EventBox { props, children } => WidgetNode::EventBox {
                props: with_dyn_id(props.clone(), parent_path),
                children: process_children(children, parent_path, "eventbox"),
            },
            WidgetNode::ToolTip { props, children } => WidgetNode::ToolTip {
                props: with_dyn_id(props.clone(), parent_path),
                children: process_children(children, parent_path, "tooltip"),
            },
            WidgetNode::LocalBind { props, children } => WidgetNode::LocalBind {
                props: with_dyn_id(props.clone(), parent_path),
                children: process_children(children, parent_path, "localbind"),
            },
            WidgetNode::WidgetAction { props, children } => WidgetNode::WidgetAction {
                props: with_dyn_id(props.clone(), parent_path),
                children: process_children(children, parent_path, "widget_action"),
            },

            // == Top-level container for multiple widgets ==
            WidgetNode::Enter(children) => {
                WidgetNode::Enter(process_children(children, parent_path, "enter"))
            }

            // == Poll/Listen nodes ==
            WidgetNode::Poll { var, props } => WidgetNode::Poll {
                var: var.clone(),
                props: with_dyn_id(props.clone(), &format!("{}_poll_{}", parent_path, var)),
            },
            WidgetNode::Listen { var, props } => WidgetNode::Listen {
                var: var.clone(),
                props: with_dyn_id(props.clone(), &format!("{}_listen_{}", parent_path, var)),
            },

            // == Leaf nodes ==
            node @ WidgetNode::Label { props }
            | node @ WidgetNode::Button { props }
            | node @ WidgetNode::Image { props }
            | node @ WidgetNode::Icon { props }
            | node @ WidgetNode::Input { props }
            | node @ WidgetNode::Progress { props }
            | node @ WidgetNode::ComboBoxText { props }
            | node @ WidgetNode::Scale { props }
            | node @ WidgetNode::Checkbox { props }
            | node @ WidgetNode::Calendar { props }
            | node @ WidgetNode::ColorButton { props }
            | node @ WidgetNode::ColorChooser { props }
            | node @ WidgetNode::CircularProgress { props }
            | node @ WidgetNode::Graph { props }
            | node @ WidgetNode::GtkUI { props }
            | node @ WidgetNode::Transform { props } => {
                let new_props = with_dyn_id(props.clone(), parent_path);
                match node {
                    WidgetNode::Label { .. } => WidgetNode::Label { props: new_props },
                    WidgetNode::Button { .. } => WidgetNode::Button { props: new_props },
                    WidgetNode::Image { .. } => WidgetNode::Image { props: new_props },
                    WidgetNode::Icon { .. } => WidgetNode::Icon { props: new_props },
                    WidgetNode::Input { .. } => WidgetNode::Input { props: new_props },
                    WidgetNode::Progress { .. } => WidgetNode::Progress { props: new_props },
                    WidgetNode::ComboBoxText { .. } => {
                        WidgetNode::ComboBoxText { props: new_props }
                    }
                    WidgetNode::Scale { .. } => WidgetNode::Scale { props: new_props },
                    WidgetNode::Checkbox { .. } => WidgetNode::Checkbox { props: new_props },
                    WidgetNode::Calendar { .. } => WidgetNode::Calendar { props: new_props },
                    WidgetNode::ColorButton { .. } => WidgetNode::ColorButton { props: new_props },
                    WidgetNode::ColorChooser { .. } => {
                        WidgetNode::ColorChooser { props: new_props }
                    }
                    WidgetNode::CircularProgress { .. } => {
                        WidgetNode::CircularProgress { props: new_props }
                    }
                    WidgetNode::Graph { .. } => WidgetNode::Graph { props: new_props },
                    WidgetNode::GtkUI { .. } => WidgetNode::GtkUI { props: new_props },
                    WidgetNode::Transform { .. } => WidgetNode::Transform { props: new_props },
                    _ => unreachable!(),
                }
            }
        }
    }
}
