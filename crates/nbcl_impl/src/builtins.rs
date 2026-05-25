use nbcl::{NativeNodeSchema, NbclEngine, PropValidation, Type, Value};
use std::collections::HashMap;

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
        child_count: Some((0, 0)),
    });

    // == Top-level macros ==
    let mut poll_args = HashMap::new();
    let mut listen_args = HashMap::new();

    poll_args.insert("cmd".to_string(), Type::Str);
    poll_args.insert("initial".to_string(), Type::Str);
    poll_args.insert("interval".to_string(), Type::Str);
    poll_args.insert("skip_unchanged".to_string(), Type::Bool);

    listen_args.insert("cmd".to_string(), Type::Str);
    listen_args.insert("initial".to_string(), Type::Str);

    engine.register_node(NativeNodeSchema {
        type_name: "Poll".into(),
        enforce_id: true,
        validation: PropValidation::Strict(poll_args),
        child_count: Some((0, 0)),
    });

    engine.register_node(NativeNodeSchema {
        type_name: "Listen".into(),
        enforce_id: true,
        validation: PropValidation::Strict(listen_args),
        child_count: Some((0, 0)),
    });

    engine.register_node(NativeNodeSchema {
        type_name: "Window".into(),
        enforce_id: true,
        validation: PropValidation::Loose,
        child_count: Some((1, 1)),
    });
}

pub fn register_all_fns(engine: &mut NbclEngine) {
    engine.register_native_fn(
        "global",
        vec![Type::Str],
        Type::Object("GlobalVar".to_string()),
        |mut args| {
            let mut data = Vec::new();

            data.push(args.remove(0));
            data.push(Value::Str(String::new()));
            data.push(Value::Str(String::new()));

            Ok(Value::Object("GlobalVar".to_string(), Box::new(Value::List(data))))
        },
    );

    engine.register_native_fn(
        "mutate",
        vec![Type::Object("GlobalVar".to_string()), Type::Str],
        Type::Object("GlobalVar".to_string()),
        |mut args| {
            let mut glob_var = args.remove(0);
            let str_interpol = args.remove(0);

            match glob_var {
                Value::Object(_, ref mut data) => {
                    if let Value::List(ref mut inner_data) = **data {
                        inner_data[2] = str_interpol;
                    }
                    Ok(glob_var)
                }
                _ => Ok(glob_var),
            }
        },
    );
}
