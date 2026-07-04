use ewwii_plugin_api::{IpcRequest, WidgetControlType};
use nbcl::{NativeNodeSchema, NbclEngine, PropValidation, Type, Value};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;

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
    register_with_children!("Animation", Some((1, 1)));

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
    let mut script_args = HashMap::new();

    poll_args.insert("cmd".to_string(), Type::Str);
    poll_args.insert("initial".to_string(), Type::Str);
    poll_args.insert("interval".to_string(), Type::Str);
    poll_args.insert("skip_unchanged".to_string(), Type::Bool);

    listen_args.insert("cmd".to_string(), Type::Str);
    listen_args.insert("initial".to_string(), Type::Str);

    script_args.insert("every".to_string(), Type::Str);
    script_args.insert("on".to_string(), Type::Str);
    script_args.insert("run".to_string(), Type::Lambda);

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
        type_name: "Script".into(),
        enforce_id: false,
        validation: PropValidation::Strict(script_args),
        child_count: Some((0, 0)),
    });

    engine.register_node(NativeNodeSchema {
        type_name: "Window".into(),
        enforce_id: true,
        validation: PropValidation::Loose,
        child_count: Some((1, 1)),
    });
}

pub fn register_all_fns(
    engine: &mut NbclEngine,
    ipc_tx: UnboundedSender<IpcRequest>,
) {
    engine.register_native_fn(
        "global",
        vec![Type::Str],
        Type::Object("GlobalVar".to_string()),
        |mut args| {
            let data = vec![
                // name
                args.remove(0),
                // template
                Value::Str(String::new()),
                // initial
                Value::Str(String::new()),
                // mutation
                Value::Str(String::new()),
            ];

            Ok(Value::Object("GlobalVar".to_string(), Box::new(Value::List(data))))
        },
    );

    engine.register_native_fn(
        "mutate",
        vec![Type::Object("GlobalVar".to_string()), Type::Lambda],
        Type::Object("GlobalVar".to_string()),
        |mut args| {
            let mut glob_var = args.remove(0);
            let lamda = args.remove(0);
            let Value::Lambda(lamda_name) = lamda else {
                return Err(crate::runtime_err!("expected second param of mutate() to be lamda"));
            };

            match glob_var {
                Value::Object(_, ref mut data) => {
                    if let Value::List(ref mut inner_data) = **data {
                        inner_data[3] = Value::Str(lamda_name);
                    }
                    Ok(glob_var)
                }
                _ => Err(crate::runtime_err!("unexpected value shape in concat()")),
            }
        }
    );

    engine.register_native_fn(
        "template",
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
                _ => Err(crate::runtime_err!("unexpected value shape in concat()")),
            }
        },
    );

    // Widget control stuff
    engine.register_native_fn(
        "find",
        vec![Type::Object("WidgetCtrl".into()), Type::Str],
        Type::Object("WidgetCtrl".into()),
        |mut args: Vec<Value>| {
            let mut global_var = args.remove(0);
            let name = args.remove(0);

            match global_var {
                Value::Object(_, ref mut data) => {
                    if let Value::Str(ref mut inner_data) = **data {
                        if let Value::Str(new_val) = name {
                            *inner_data = new_val;
                        }
                    }
                    Ok(global_var)
                }
                _ => Ok(global_var),
            }
        },
    );

    let tx = ipc_tx.clone();
    engine.register_native_fn(
        "set_property",
        vec![Type::Object("WidgetCtrl".into()), Type::Str, Type::Str],
        Type::Object("WidgetCtrl".into()),
        move |mut args: Vec<Value>| {
            let global_var = args.remove(0);
            let prop = args.remove(0);
            let value = args.remove(0);

            match global_var {
                Value::Object(_, ref data) => {
                    let prop_str = match prop {
                        Value::Str(s) => s,
                        _ => return Err(crate::runtime_err!("expected string")),
                    };
                    let value_str = match value {
                        Value::Str(s) => s,
                        _ => return Err(crate::runtime_err!("expected string")),
                    };

                    let data = match &**data {
                        Value::Str(s) => s,
                        _ => return Err(crate::runtime_err!("expected string")),
                    }
                    .clone();

                    let req = IpcRequest::WidgetControl(WidgetControlType::PropertyUpdate {
                        prop: prop_str,
                        value: value_str,
                        widget: data,
                    });

                    let _ = tx.send(req);

                    Ok(global_var)
                }
                _ => Ok(global_var),
            }
        },
    );

    let tx = ipc_tx.clone();
    engine.register_native_fn(
        "add_class",
        vec![Type::Object("WidgetCtrl".into()), Type::Str],
        Type::Object("WidgetCtrl".into()),
        move |mut args: Vec<Value>| {
            let global_var = args.remove(0);
            let class = args.remove(0);

            match global_var {
                Value::Object(_, ref data) => {
                    let class_str = match class {
                        Value::Str(s) => s,
                        _ => return Err(crate::runtime_err!("expected string")),
                    };

                    let data = match &**data {
                        Value::Str(s) => s,
                        _ => return Err(crate::runtime_err!("expected string")),
                    }
                    .clone();

                    let req = IpcRequest::WidgetControl(WidgetControlType::AddClass {
                        class: class_str,
                        widget: data,
                    });

                    let _ = tx.send(req);

                    Ok(global_var)
                }
                _ => Ok(global_var),
            }
        },
    );

    engine.register_native_fn(
        "remove_class",
        vec![Type::Object("WidgetCtrl".into()), Type::Str],
        Type::Object("WidgetCtrl".into()),
        move |mut args: Vec<Value>| {
            let global_var = args.remove(0);
            let class = args.remove(0);

            match global_var {
                Value::Object(_, ref data) => {
                    let class_str = match class {
                        Value::Str(s) => s,
                        _ => return Err(crate::runtime_err!("expected string")),
                    };

                    let data = match &**data {
                        Value::Str(s) => s,
                        _ => return Err(crate::runtime_err!("expected string")),
                    }
                    .clone();

                    let req = IpcRequest::WidgetControl(WidgetControlType::RemoveClass {
                        class: class_str,
                        widget: data,
                    });

                    let _ = ipc_tx.send(req);

                    Ok(global_var)
                }
                _ => Ok(global_var),
            }
        },
    );
}
