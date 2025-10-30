use ahash::AHasher;
use anyhow::Result;
use rhai::Map;
use scan_prop_proc::scan_prop;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
#[scan_prop]
pub enum WidgetNode {
    Label { props: Map },
    Box { props: Map, children: Vec<WidgetNode> },
    FlowBox { props: Map, children: Vec<WidgetNode> },
    Button { props: Map },
    Image { props: Map },
    Icon { props: Map },
    Input { props: Map },
    Progress { props: Map },
    ComboBoxText { props: Map },
    Slider { props: Map },
    Checkbox { props: Map },
    Expander { props: Map, children: Vec<WidgetNode> },
    Revealer { props: Map, children: Vec<WidgetNode> },
    Scroll { props: Map, children: Vec<WidgetNode> },
    OverLay { props: Map, children: Vec<WidgetNode> },
    Stack { props: Map, children: Vec<WidgetNode> },
    Calendar { props: Map },
    ColorButton { props: Map },
    ColorChooser { props: Map },
    CircularProgress { props: Map },
    Graph { props: Map },
    Transform { props: Map },
    EventBox { props: Map, children: Vec<WidgetNode> },
    ToolTip { props: Map, children: Vec<WidgetNode> },

    // Top-level macros
    DefWindow { name: String, props: Map, node: Box<WidgetNode> },
    // Poll { var: String, interval: String, cmd: String, initial: String },
    // Listen { var: String, signal: String },
    Poll { var: String, props: Map },
    Listen { var: String, props: Map },
    Enter(Vec<WidgetNode>),
}

#[derive(Clone)]
pub struct WidgetInfo<'a> {
    pub node: &'a WidgetNode,
    pub props: &'a Map,
    pub widget_type: &'a str,
    pub children: Vec<&'a WidgetNode>,
    pub parent_id: Option<u64>,
}

pub fn get_id_to_widget_info<'a>(
    node: &'a WidgetNode,
    id_to_props: &mut HashMap<u64, WidgetInfo<'a>>,
    parent_id: Option<u64>,
) -> Result<()> {
    match node {
        // == Special Cases == //
        WidgetNode::Enter(children) => {
            for child in children {
                get_id_to_widget_info(child, id_to_props, parent_id)?;
            }
        }
        WidgetNode::DefWindow { node: child, .. } => {
            get_id_to_widget_info(&**child, id_to_props, parent_id)?
        }

        // == Normal Widgets == //
        WidgetNode::Box { props, children } => {
            let id = hash_props_and_type(props, "Box");
            insert_wdgt_info(node, props, "Box", children.as_slice(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::FlowBox { props, children } => {
            let id = hash_props_and_type(props, "FlowBox");
            insert_wdgt_info(node, props, "FlowBox", children.as_slice(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::EventBox { props, children } => {
            let id = hash_props_and_type(props, "EventBox");
            insert_wdgt_info(node, props, "EventBox", children.as_slice(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::CircularProgress { props } => {
            // let id = hash_props_and_type(props, "CircularProgress");
            insert_wdgt_info(node, props, "CircularProgress", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Graph { props } => {
            // let id = hash_props_and_type(props, "Graph");
            insert_wdgt_info(node, props, "Graph", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Transform { props } => {
            // let id = hash_props_and_type(props, "Transform");
            insert_wdgt_info(node, props, "Transform", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Slider { props } => {
            // let id = hash_props_and_type(props, "Slider");
            insert_wdgt_info(node, props, "Slider", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Progress { props } => {
            // let id = hash_props_and_type(props, "Progress");
            insert_wdgt_info(node, props, "Progress", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Image { props } => {
            // let id = hash_props_and_type(props, "Image");
            insert_wdgt_info(node, props, "Image", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Icon { props } => {
            // let id = hash_props_and_type(props, "Icon");
            insert_wdgt_info(node, props, "Icon", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Button { props } => {
            // let id = hash_props_and_type(props, "Button");
            insert_wdgt_info(node, props, "Button", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Label { props } => {
            // let id = hash_props_and_type(props, "Label");
            insert_wdgt_info(node, props, "Label", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Input { props } => {
            // let id = hash_props_and_type(props, "Input");
            insert_wdgt_info(node, props, "Input", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Calendar { props } => {
            // let id = hash_props_and_type(props, "Calendar");
            insert_wdgt_info(node, props, "Calendar", &[], parent_id, id_to_props)?;
        }
        WidgetNode::ColorButton { props } => {
            // let id = hash_props_and_type(props, "ColorButton");
            insert_wdgt_info(node, props, "ColorButton", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Expander { props, children } => {
            let id = hash_props_and_type(props, "Expander");
            insert_wdgt_info(node, props, "Expander", children.as_slice(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::ToolTip { props, children } => {
            let id = hash_props_and_type(props, "ToolTip");
            insert_wdgt_info(node, props, "ToolTip", children.as_slice(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::ColorChooser { props } => {
            // let id = hash_props_and_type(props, "ColorChooser");
            insert_wdgt_info(node, props, "ColorChooser", &[], parent_id, id_to_props)?;
        }
        WidgetNode::ComboBoxText { props } => {
            // let id = hash_props_and_type(props, "ComboBoxText");
            insert_wdgt_info(node, props, "ComboBoxText", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Checkbox { props } => {
            // let id = hash_props_and_type(props, "Checkbox");
            insert_wdgt_info(node, props, "Checkbox", &[], parent_id, id_to_props)?;
        }
        WidgetNode::Revealer { props, children } => {
            let id = hash_props_and_type(props, "Revealer");
            insert_wdgt_info(node, props, "Revealer", children.as_slice(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::Scroll { props, children } => {
            let id = hash_props_and_type(props, "Scroll");
            insert_wdgt_info(node, props, "Scroll", children.as_slice(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::OverLay { props, children } => {
            let id = hash_props_and_type(props, "OverLay");
            insert_wdgt_info(node, props, "OverLay", children.as_slice(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::Stack { props, children } => {
            let id = hash_props_and_type(props, "Stack");
            insert_wdgt_info(node, props, "Stack", children.as_slice(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        _ => { /* do nothing */ }
    }

    Ok(())
}

fn insert_wdgt_info<'a>(
    node: &'a WidgetNode,
    props: &'a Map,
    widget_type: &'a str,
    children: &'a [WidgetNode],
    parent_id: Option<u64>,
    id_to_info: &mut HashMap<u64, WidgetInfo<'a>>,
) -> Result<()> {
    let id = hash_props_and_type(props, widget_type);
    let info =
        WidgetInfo { node, props, widget_type, children: children.iter().collect(), parent_id };
    id_to_info.insert(id, info);
    Ok(())
}

pub fn hash_props_and_type(props: &Map, widget_type_str: &str) -> u64 {
    let mut hasher = AHasher::default();

    widget_type_str.hash(&mut hasher);

    props.len().hash(&mut hasher);

    let get_string_fn = shared_utils::extract_props::get_string_prop;
    let dyn_id = get_string_fn(&props, "dyn_id", Some("")).unwrap_or("".to_string());

    dyn_id.hash(&mut hasher);

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::{Dynamic, Map};

    #[test]
    fn test_hash_props_and_type_consistency() {
        let mut props = Map::new();
        props.insert("class".into(), Dynamic::from("mywidget"));
        props.insert("enabled".into(), Dynamic::from(true));
        props.insert("count".into(), Dynamic::from(42_i64));
        //? IMPORTANT
        props.insert("dyn_id".into(), Dynamic::from("mywidget_test"));

        // Nested map
        let mut nested = Map::new();
        nested.insert("nested_key".into(), Dynamic::from("value"));
        props.insert("nested".into(), Dynamic::from(nested));

        // Array
        let arr = vec![Dynamic::from(1_i64), Dynamic::from(2_i64), Dynamic::from(3_i64)];
        props.insert("arr".into(), Dynamic::from(arr));

        let widget_type = "Box";

        let hash1 = hash_props_and_type(&props, widget_type);
        let hash2 = hash_props_and_type(&props, widget_type);

        assert_eq!(hash1, hash2, "Hashes should be consistent on same input");

        // Change one prop and expect different hash
        let mut props_modified = props.clone();
        props_modified.insert("count".into(), Dynamic::from(43_i64));

        let hash3 = hash_props_and_type(&props_modified, widget_type);
        assert_eq!(hash1, hash3, "Hashes should be consistent even when props change");

        // Different widget type string
        let hash4 = hash_props_and_type(&props, "Button");
        assert_ne!(hash1, hash4, "Hashes should differ for different widget types");
    }
}
