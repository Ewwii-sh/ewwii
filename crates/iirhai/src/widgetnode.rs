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

pub fn get_id_to_props_map(root_node: &WidgetNode, id_to_props: &mut HashMap<u64, Map>) -> Result<()> {
    match root_node {
        WidgetNode::Box { props, children } => {
            insert_props(props, "Box", id_to_props)?;
            for child in children {
                get_id_to_props_map(child, id_to_props)?;
            }
        }
        WidgetNode::CenterBox { props, children } => {
            insert_props(props, "CenterBox", id_to_props)?;
            for child in children {
                get_id_to_props_map(child, id_to_props)?;
            }
        }
        WidgetNode::EventBox { props, children } => {
            insert_props(props, "EventBox", id_to_props)?;
            for child in children {
                get_id_to_props_map(child, id_to_props)?;
            }
        }
        WidgetNode::CircularProgress { props } => {
            insert_props(props, "CircularProgress", id_to_props)?;
        }
        WidgetNode::Graph { props } => {
            insert_props(props, "Graph", id_to_props)?;
        }
        WidgetNode::Transform { props } => {
            insert_props(props, "Transform", id_to_props)?;
        }
        WidgetNode::Slider { props } => {
            insert_props(props, "Slider", id_to_props)?;
        }
        WidgetNode::Progress { props } => {
            insert_props(props, "Progress", id_to_props)?;
        }
        WidgetNode::Image { props } => {
            insert_props(props, "Image", id_to_props)?;
        }
        WidgetNode::Button { props } => {
            insert_props(props, "Button", id_to_props)?;
        }
        WidgetNode::Label { props } => {
            insert_props(props, "Label", id_to_props)?;
        }
        WidgetNode::Input { props } => {
            insert_props(props, "Input", id_to_props)?;
        }
        WidgetNode::Calendar { props } => {
            insert_props(props, "Calendar", id_to_props)?;
        }
        WidgetNode::ColorButton { props } => {
            insert_props(props, "ColorButton", id_to_props)?;
        }
        WidgetNode::Expander { props, children } => {
            insert_props(props, "Expander", id_to_props)?;
            for child in children {
                get_id_to_props_map(child, id_to_props)?;
            }
        }
        WidgetNode::ToolTip { children } => {
            for child in children {
                get_id_to_props_map(child, id_to_props)?;
            }
        }
        WidgetNode::ColorChooser { props } => {
            insert_props(props, "ColorChooser", id_to_props)?;
        }
        WidgetNode::ComboBoxText { props } => {
            insert_props(props, "ComboBoxText", id_to_props)?;
        }
        WidgetNode::Checkbox { props } => {
            insert_props(props, "Checkbox", id_to_props)?;
        }
        WidgetNode::Revealer { props, children } => {
            insert_props(props, "Revealer", id_to_props)?;
            for child in children {
                get_id_to_props_map(child, id_to_props)?;
            }
        }
        WidgetNode::Scroll { props, children } => {
            insert_props(props, "Scroll", id_to_props)?;

            for child in children {
                get_id_to_props_map(child, id_to_props)?;
            }
        }
        WidgetNode::OverLay { children } => {
            for child in children {
                get_id_to_props_map(child, id_to_props)?;
            }
        }
        WidgetNode::Stack { props, children } => {
            insert_props(props, "Stack", id_to_props)?;
            for child in children {
                get_id_to_props_map(child, id_to_props)?;
            }
        }
        _ => {
            // do nothing for now ig?
        }
    }

    Ok(())
}

fn insert_props(props: &Map, widget_type: &str, id_to_props: &mut HashMap<u64, Map>) -> Result<()> {
    let id = hash_props_and_type(props, widget_type);
    id_to_props.insert(id, props.clone());
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
