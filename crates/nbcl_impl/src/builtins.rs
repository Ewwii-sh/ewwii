use nbcl::{NbclEngine, NativeNodeSchema, PropValidation};

pub fn register_all_nodes(engine: &mut NbclEngine) {
    // == Primitive nodes (nodes that does not take in children) ==
    macro_rules! register_primitive {
        ($name:expr) => {
            engine.register_node(NativeNodeSchema {
                type_name: $name.to_string(),
                enforce_id: false,
                validation: PropValidation::Loose,
                child_count: Some((0, 0)),
            });
        };
    }

    register_primitive!("Label");
    register_primitive!("Button");
    register_primitive!("Image");
    register_primitive!("Input");
    register_primitive!("Progress");
    register_primitive!("ComboBoxText");
    register_primitive!("Scale");
    register_primitive!("Checkbox");
    register_primitive!("Calendar");
    register_primitive!("Graph");
    register_primitive!("Transform");
    register_primitive!("CircularProgress");
    register_primitive!("ColorButton");
    register_primitive!("ColorChooser");

    // == Nodes with children ==
    macro_rules! register_with_children {
        ($name:expr, $child_count:expr) => {
            engine.register_node(NativeNodeSchema {
                type_name: $name.to_string(),
                enforce_id: false,
                validation: PropValidation::Loose,
                child_count: $child_count,
            });
        };
    }

    register_with_children!("Box", None);
    register_with_children!("FlowBox", None);
    register_with_children!("Expander", Some((1, 1)));
    register_with_children!("Revealer", Some((0, 1)));
    register_with_children!("Scroll", Some((1, 1)));
    register_with_children!("OverLay", None);
    register_with_children!("Stack", None);
    register_with_children!("EventBox", None);
    register_with_children!("ToolTip", Some((2, 2)));

    // == Special widget & tools ==
    engine.register_node(NativeNodeSchema {
        type_name: "GtkUI".into(),
        enforce_id: false,
        validation: PropValidation::Loose,
        child_count: Some((0, 0))
    });

    // Got rid if it during the migration to nbcl.
    // engine.register_fn(
    //     "bound",
    //     |variables: Array, closure: FnPtr| -> Result<GlobalCompare, Box<EvalAltResult>> {
    //         let handle = rand::random::<u64>();
    //         let unique_name = format!("\0__globalbound__{}", &handle);
    //         let vars = variables.into_iter().map(Property::from_dynamic).collect();

    //         crate::callback::register_callback(handle, closure.clone());

    //         let callback = Callback { name: closure.fn_name().to_string(), handle: Some(handle) };

    //         Ok(GlobalCompare { name: unique_name, vars, closure: callback })
    //     },
    // );

    // == Top-level macros ==
    engine.register_node(NativeNodeSchema {
        type_name: "Window".into(),
        enforce_id: true,
        validation: PropValidation::Loose,
        child_count: Some((1, 1))
    });

    // Need to think about it.
    //
    // engine.register_fn("poll", |var: &str, props: Map| -> Result<WidgetNode, Box<EvalAltResult>> {
    //     let prop_map = PropertyMap::from_rhai(props);
    //     Ok(WidgetNode::Poll { var: var.to_string(), props: prop_map })
    // });

    // engine.register_fn(
    //     "listen",
    //     |var: &str, props: Map| -> Result<WidgetNode, Box<EvalAltResult>> {
    //         let prop_map = PropertyMap::from_rhai(props);
    //         Ok(WidgetNode::Listen { var: var.to_string(), props: prop_map })
    //     },
    // );
}
