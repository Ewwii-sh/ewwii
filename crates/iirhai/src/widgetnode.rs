use ahash::AHasher;
use anyhow::Result;
use rhai::Map;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub enum WidgetNode {
    Label { props: Map },
    Box { props: Map, children: Vec<WidgetNode> },
    CenterBox { props: Map, children: Vec<WidgetNode> },
    Button { props: Map },
    Image { props: Map },
    Input { props: Map },
    Progress { props: Map },
    ComboBoxText { props: Map },
    Slider { props: Map },
    Checkbox { props: Map },
    Expander { props: Map, children: Vec<WidgetNode> },
    Revealer { props: Map, children: Vec<WidgetNode> },
    Scroll { props: Map, children: Vec<WidgetNode> },
    OverLay { children: Vec<WidgetNode> },
    Stack { props: Map, children: Vec<WidgetNode> },
    Calendar { props: Map },
    ColorButton { props: Map },
    ColorChooser { props: Map },
    CircularProgress { props: Map },
    Graph { props: Map },
    Transform { props: Map },
    EventBox { props: Map, children: Vec<WidgetNode> },
    ToolTip { children: Vec<WidgetNode> },

    // Top-level macros
    DefWindow { name: String, props: Map, node: Box<WidgetNode> },
    // Poll { var: String, interval: String, cmd: String, initial: String },
    // Listen { var: String, signal: String },
    Poll { var: String, props: Map },
    Listen { var: String, props: Map },
    Enter(Vec<WidgetNode>),
}

#[derive(Clone)]
pub struct WidgetInfo {
    pub node: WidgetNode,
    pub props: Map,
    pub widget_type: String,
    pub children: Vec<WidgetNode>,
    pub parent_id: Option<u64>,
}

pub fn get_id_to_widget_info(
    node: &WidgetNode,
    id_to_props: &mut HashMap<u64, WidgetInfo>,
    parent_id: Option<u64>,
) -> Result<()> {
    match node {
        WidgetNode::Box { props, children } => {
            let id = hash_props_and_type(props, "Box");
            insert_wdgt_info(node, props, "Box", children.clone(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::CenterBox { props, children } => {
            let id = hash_props_and_type(props, "CenterBox");
            insert_wdgt_info(node, props, "CenterBox", children.clone(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::EventBox { props, children } => {
            let id = hash_props_and_type(props, "EventBox");
            insert_wdgt_info(node, props, "EventBox", children.clone(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::CircularProgress { props } => {
            // let id = hash_props_and_type(props, "CircularProgress");
            insert_wdgt_info(node, props, "CircularProgress", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::Graph { props } => {
            // let id = hash_props_and_type(props, "Graph");
            insert_wdgt_info(node, props, "Graph", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::Transform { props } => {
            // let id = hash_props_and_type(props, "Transform");
            insert_wdgt_info(node, props, "Transform", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::Slider { props } => {
            // let id = hash_props_and_type(props, "Slider");
            insert_wdgt_info(node, props, "Slider", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::Progress { props } => {
            // let id = hash_props_and_type(props, "Progress");
            insert_wdgt_info(node, props, "Progress", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::Image { props } => {
            // let id = hash_props_and_type(props, "Image");
            insert_wdgt_info(node, props, "Image", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::Button { props } => {
            // let id = hash_props_and_type(props, "Button");
            insert_wdgt_info(node, props, "Button", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::Label { props } => {
            // let id = hash_props_and_type(props, "Label");
            insert_wdgt_info(node, props, "Label", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::Input { props } => {
            // let id = hash_props_and_type(props, "Input");
            insert_wdgt_info(node, props, "Input", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::Calendar { props } => {
            // let id = hash_props_and_type(props, "Calendar");
            insert_wdgt_info(node, props, "Calendar", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::ColorButton { props } => {
            // let id = hash_props_and_type(props, "ColorButton");
            insert_wdgt_info(node, props, "ColorButton", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::Expander { props, children } => {
            let id = hash_props_and_type(props, "Expander");
            insert_wdgt_info(node, props, "Expander", children.clone(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::ToolTip { children } => {
            for child in children {
                get_id_to_widget_info(child, id_to_props, parent_id)?;
            }
        }
        WidgetNode::ColorChooser { props } => {
            // let id = hash_props_and_type(props, "ColorChooser");
            insert_wdgt_info(node, props, "ColorChooser", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::ComboBoxText { props } => {
            // let id = hash_props_and_type(props, "ComboBoxText");
            insert_wdgt_info(node, props, "ComboBoxText", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::Checkbox { props } => {
            // let id = hash_props_and_type(props, "Checkbox");
            insert_wdgt_info(node, props, "Checkbox", vec![], parent_id, id_to_props)?;
        }
        WidgetNode::Revealer { props, children } => {
            let id = hash_props_and_type(props, "Revealer");
            insert_wdgt_info(node, props, "Revealer", children.clone(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::Scroll { props, children } => {
            let id = hash_props_and_type(props, "Scroll");
            insert_wdgt_info(node, props, "Scroll", children.clone(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        WidgetNode::OverLay { children } => {
            for child in children {
                get_id_to_widget_info(child, id_to_props, parent_id)?;
            }
        }
        WidgetNode::Stack { props, children } => {
            let id = hash_props_and_type(props, "Stack");
            insert_wdgt_info(node, props, "Stack", children.clone(), parent_id, id_to_props)?;
            for child in children {
                get_id_to_widget_info(child, id_to_props, Some(id))?;
            }
        }
        _ => { /* do nothing */ }
    }

    Ok(())
}

fn insert_wdgt_info(
    node: &WidgetNode,
    props: &Map,
    widget_type: &str,
    children: Vec<WidgetNode>,
    parent_id: Option<u64>,
    id_to_info: &mut HashMap<u64, WidgetInfo>,
) -> Result<()> {
    let id = hash_props_and_type(props, widget_type);
    let info = WidgetInfo { node: node.clone(), props: props.clone(), widget_type: widget_type.to_string(), children, parent_id };
    id_to_info.insert(id, info);
    Ok(())
}

pub fn hash_props_and_type(props: &Map, widget_type_str: &str) -> u64 {
    let mut hasher = AHasher::default();

    widget_type_str.hash(&mut hasher);

    props.len().hash(&mut hasher);

    let get_string_fn = ewwii_shared_util::general_helper::get_string_prop;
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
