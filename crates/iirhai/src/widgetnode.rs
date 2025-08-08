use rhai::{Dynamic, Map, Array};
use anyhow::{Result, bail};
use ahash::AHasher;
use std::hash::{Hasher, Hash};
use std::collections::HashMap;

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

pub fn get_id_to_props_map(root_node: &WidgetNode) -> Result<HashMap<u64, Map>> {
    let mut id_to_props = HashMap::new();

    let mut insert_props = |props: &Map, widget_type: &str| -> Result<()> {
        let id = hash_props_and_type(props, widget_type);
        id_to_props.insert(id, props.clone());
        Ok(())
    };

    match root_node {
        WidgetNode::Box { props, .. } => insert_props(props, "Box")?,
        WidgetNode::CenterBox { props, .. } => insert_props(props, "CenterBox")?,
        WidgetNode::EventBox { props, .. } => insert_props(props, "EventBox")?,
        WidgetNode::CircularProgress { props } => insert_props(props, "CircularProgress")?,
        WidgetNode::Graph { props } => insert_props(props, "Graph")?,
        WidgetNode::Transform { props } => insert_props(props, "Transform")?,
        WidgetNode::Slider { props } => insert_props(props, "Slider")?,
        WidgetNode::Progress { props } => insert_props(props, "Progress")?,
        WidgetNode::Image { props } => insert_props(props, "Image")?,
        WidgetNode::Button { props } => insert_props(props, "Button")?,
        WidgetNode::Label { props } => insert_props(props, "Label")?,
        WidgetNode::Input { props } => insert_props(props, "Input")?,
        WidgetNode::Calendar { props } => insert_props(props, "Calendar")?,
        WidgetNode::ColorButton { props } => insert_props(props, "ColorButton")?,
        WidgetNode::Expander { props, .. } => insert_props(props, "Expander")?,
        WidgetNode::ColorChooser { props } => insert_props(props, "ColorChooser")?,
        WidgetNode::ComboBoxText { props } => insert_props(props, "ComboBoxText")?,
        WidgetNode::Checkbox { props } => insert_props(props, "Checkbox")?,
        WidgetNode::Revealer { props, .. } => insert_props(props, "Revealer")?,
        WidgetNode::Scroll { props, .. } => insert_props(props, "Scroll")?,
        WidgetNode::Stack { props, .. } => insert_props(props, "Stack")?,
        _ => {
            // do nothing for now ig?
        }
    }

    Ok(id_to_props)
}

pub fn hash_props_and_type(props: &Map, widget_type_str: &str) -> u64 {
    let mut hasher = AHasher::default();

    widget_type_str.hash(&mut hasher);

    let mut kv_pairs: Vec<_> = props.iter().collect();
    kv_pairs.sort_by_key(|(k, _)| k.clone());

    for (k, v) in kv_pairs {
        k.hash(&mut hasher);
        let val_str = serialize_value(v);
        val_str.hash(&mut hasher);
    }

    hasher.finish()
}

fn serialize_value(value: &Dynamic) -> String {
    if value.is::<String>() {
        value.clone_cast::<String>()
    } else if value.is::<bool>() {
        value.clone_cast::<bool>().to_string()
    } else if value.is::<i64>() {
        value.clone_cast::<i64>().to_string()
    } else if value.is::<f64>() {
        value.clone_cast::<f64>().to_string()
    } else if value.is::<Array>() {
        let arr = value.clone_cast::<Array>();
        let serialized_items: Vec<String> = arr.iter().map(|v| serialize_value(v)).collect();
        format!("[{}]", serialized_items.join(","))
    } else if value.is::<Map>() {
        let map = value.clone_cast::<Map>();
        let mut kvs: Vec<_> = map.iter().collect();
        kvs.sort_by_key(|(k, _)| k.clone());
        let serialized_pairs: Vec<String> = kvs.iter()
            .map(|(k, v)| format!("{}:{}", k, serialize_value(v)))
            .collect();
        format!("{{{}}}", serialized_pairs.join(","))
    } else {
        format!("{:?}", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::{Map, Dynamic};

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
        assert_ne!(hash1, hash3, "Hashes should differ when props change");

        // Different widget type string
        let hash4 = hash_props_and_type(&props, "Button");
        assert_ne!(hash1, hash4, "Hashes should differ for different widget types");
    }
}
