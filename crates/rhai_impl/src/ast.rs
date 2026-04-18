use ahash::AHasher;
use ewwii_shared_utils::prop::PropertyMap;
use ewwii_shared_utils::prop_utils::{get_string_prop, unwrap_static};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub enum WidgetNode {
    Label { props: PropertyMap },
    Box { props: PropertyMap, children: Vec<WidgetNode> },
    FlowBox { props: PropertyMap, children: Vec<WidgetNode> },
    Button { props: PropertyMap },
    Image { props: PropertyMap },
    Input { props: PropertyMap },
    Progress { props: PropertyMap },
    ComboBoxText { props: PropertyMap },
    Scale { props: PropertyMap },
    Checkbox { props: PropertyMap },
    Expander { props: PropertyMap, children: Vec<WidgetNode> },
    Revealer { props: PropertyMap, children: Vec<WidgetNode> },
    Scroll { props: PropertyMap, children: Vec<WidgetNode> },
    OverLay { props: PropertyMap, children: Vec<WidgetNode> },
    Stack { props: PropertyMap, children: Vec<WidgetNode> },
    Calendar { props: PropertyMap },
    ColorButton { props: PropertyMap },
    ColorChooser { props: PropertyMap },
    CircularProgress { props: PropertyMap },
    Graph { props: PropertyMap },
    Transform { props: PropertyMap },
    EventBox { props: PropertyMap, children: Vec<WidgetNode> },
    ToolTip { props: PropertyMap, children: Vec<WidgetNode> },

    // Special
    GtkUI { props: PropertyMap },

    // Top-level macros
    DefWindow { name: String, props: PropertyMap, node: Box<WidgetNode> },
    // Poll { var: String, interval: String, cmd: String, initial: String },
    // Listen { var: String, signal: String },
    Poll { var: String, props: PropertyMap },
    Listen { var: String, props: PropertyMap },
    Tree(Vec<WidgetNode>),
}

pub fn hash_props_and_type(props: &PropertyMap, widget_type_str: &str) -> u64 {
    let mut hasher = AHasher::default();

    widget_type_str.hash(&mut hasher);

    props.len().hash(&mut hasher);

    let dyn_id = get_string_prop(&props, "dyn_id", Some(""))
        .map(|p| unwrap_static("dyn_id", p))
        .unwrap_or_default();

    dyn_id.hash(&mut hasher);

    hasher.finish()
}

pub fn hash_props(props: &PropertyMap) -> u64 {
    let mut hasher = AHasher::default();

    props.hash(&mut hasher);

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ewwii_shared_utils::prop::{Property, PropertyMap};

    #[test]
    fn test_hash_props_and_type_consistency() {
        let mut props = PropertyMap::new();
        props.insert("class", Property::String("mywidget".into()));
        props.insert("enabled", Property::Bool(true));
        props.insert("count", Property::Int(42));
        props.insert("dyn_id", Property::String("mywidget_test".into()));

        // Nested map
        let mut nested = PropertyMap::new();
        nested.insert("nested_key", Property::String("value".into()));
        props.insert("nested", Property::Map(nested));

        // Array
        let arr = vec![Property::Int(1), Property::Int(2), Property::Int(3)];
        props.insert("arr", Property::Array(arr));

        let widget_type = "Box";

        // Hash checks
        let hash1 = hash_props_and_type(&props, widget_type);
        let hash2 = hash_props_and_type(&props, widget_type);

        assert_eq!(hash1, hash2, "Hashes should be consistent on same input");

        // Change one prop (non-identity prop)
        let mut props_modified = props.clone();
        props_modified.insert("count", Property::Int(43));

        let hash3 = hash_props_and_type(&props_modified, widget_type);
        assert_eq!(hash1, hash3, "Identity hash remains stable when data changes");

        // Different widget type string
        let hash4 = hash_props_and_type(&props, "Button");
        assert_ne!(hash1, hash4, "Hashes must differ for different widget types");
    }
}
