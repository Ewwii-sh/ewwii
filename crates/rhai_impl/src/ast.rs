use ahash::AHasher;
use rhai::Map;
use ewwii_shared_utils::prop_utils::{get_string_prop, unwrap_static};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub enum WidgetNode {
    Label { props: Map },
    Box { props: Map, children: Vec<WidgetNode> },
    FlowBox { props: Map, children: Vec<WidgetNode> },
    Button { props: Map },
    Image { props: Map },
    Input { props: Map },
    Progress { props: Map },
    ComboBoxText { props: Map },
    Scale { props: Map },
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

    // Special
    GtkUI { props: Map },

    // Top-level macros
    DefWindow { name: String, props: Map, node: Box<WidgetNode> },
    // Poll { var: String, interval: String, cmd: String, initial: String },
    // Listen { var: String, signal: String },
    Poll { var: String, props: Map },
    Listen { var: String, props: Map },
    Tree(Vec<WidgetNode>),
}

pub fn hash_props_and_type(props: &Map, widget_type_str: &str) -> u64 {
    let mut hasher = AHasher::default();

    widget_type_str.hash(&mut hasher);

    props.len().hash(&mut hasher);

    let dyn_id = get_string_prop(&props, "dyn_id", Some(""))
        .map(|p| unwrap_static("dyn_id", p))
        .unwrap_or_default();

    dyn_id.hash(&mut hasher);

    hasher.finish()
}

pub fn hash_props(props: &Map) -> u64 {
    let mut hasher = AHasher::default();

    props.hash(&mut hasher);

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
