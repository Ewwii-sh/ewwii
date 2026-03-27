use crate::ast::WidgetNode;
use crate::updates::variable::VarWatcherAPI;
use rhai::{Array, Dynamic, Engine, EvalAltResult, FnPtr, Map, NativeCallContext};
use shared_utils::variables::GlobalVar;
use std::cell::RefCell;
use std::rc::Rc;

/// Converts a Dynamic array into a Vec<WidgetNode>, returning proper errors with position.
fn children_to_vec(
    children: Array,
    ctx: &NativeCallContext,
) -> Result<Vec<WidgetNode>, Box<EvalAltResult>> {
    children
        .into_iter()
        .map(|v| {
            let type_name = v.type_name();
            v.try_cast::<WidgetNode>().ok_or_else(|| {
                Box::new(EvalAltResult::ErrorRuntime(
                    format!("Expected WidgetNode in children array, found {}", type_name).into(),
                    ctx.call_position(),
                ))
            })
        })
        .collect()
}

pub fn register_all_widgets(engine: &mut Engine, all_nodes: &Rc<RefCell<Vec<WidgetNode>>>) {
    engine.register_type::<WidgetNode>();

    // == Primitive widgets ==
    macro_rules! register_primitive {
        ($name:expr, $variant:ident) => {
            engine.register_fn($name, |props: Map| -> Result<WidgetNode, Box<EvalAltResult>> {
                Ok(WidgetNode::$variant { props })
            });
        };
    }

    register_primitive!("label", Label);
    register_primitive!("button", Button);
    register_primitive!("image", Image);
    register_primitive!("input", Input);
    register_primitive!("progress", Progress);
    register_primitive!("combo_box_text", ComboBoxText);
    register_primitive!("scale", Scale);
    register_primitive!("checkbox", Checkbox);
    register_primitive!("calendar", Calendar);
    register_primitive!("graph", Graph);
    register_primitive!("transform", Transform);
    register_primitive!("circular_progress", CircularProgress);
    register_primitive!("color_button", ColorButton);
    register_primitive!("color_chooser", ColorChooser);

    // == Widgets with children ==
    macro_rules! register_with_children {
        ($name:expr, $variant:ident) => {
            engine.register_fn(
                $name,
                |ctx: NativeCallContext,
                 props: Map,
                 children: Array|
                 -> Result<WidgetNode, Box<EvalAltResult>> {
                    let children_vec = children_to_vec(children, &ctx)?;
                    Ok(WidgetNode::$variant { props, children: children_vec })
                },
            );
        };
    }

    register_with_children!("box", Box);
    register_with_children!("flowbox", FlowBox);
    register_with_children!("expander", Expander);
    register_with_children!("revealer", Revealer);
    register_with_children!("scroll", Scroll);
    register_with_children!("overlay", OverLay);
    register_with_children!("stack", Stack);
    register_with_children!("eventbox", EventBox);
    register_with_children!("tooltip", ToolTip);

    // == Special widget & tools ==
    engine.register_fn(
        "gtk_ui",
        |path: &str, load: &str| -> Result<WidgetNode, Box<EvalAltResult>> {
            let mut props = Map::new();
            props.insert("file".into(), path.into());
            props.insert("id".into(), load.into());
            Ok(WidgetNode::GtkUI { props })
        },
    );

    engine.register_fn(
        "bound",
        |variables: Array, closure: FnPtr| -> Result<GlobalVar, Box<EvalAltResult>> {
            let unique_name = format!("__bound_{:032x}", rand::random::<u128>());
            let mut bindings = Map::new();
            let mut global_var_entries: Vec<(usize, String)> = vec![];
            let mut static_values: Map = Map::new();

            bindings.insert("closure".into(), Dynamic::from(closure.clone()));

            for (idx, variable) in variables.iter().enumerate() {
                let key = idx.to_string();
                if let Some(global_var) = shared_utils::prop_utils::try_get_global_var(variable) {
                    let mut entry = Map::new();
                    entry.insert("type".into(), Dynamic::from("global"));
                    entry.insert("name".into(), Dynamic::from(global_var.name.clone()));
                    bindings.insert(key.into(), Dynamic::from(entry));
                    global_var_entries.push((idx, global_var.name.clone()));
                } else {
                    bindings.insert(key.into(), variable.clone());
                    static_values.insert(idx.to_string().into(), variable.clone());
                }
            }

            // Registration
            VarWatcherAPI::register(&unique_name, String::new());

            let unique_name_clone = unique_name.clone();
            let total_len = variables.len();

            tokio::spawn(async move {
                // Lazy subscribe to all global vars
                let mut receivers: Vec<(usize, watch::Receiver<String>)> = futures::future::join_all(
                    global_var_entries.iter().map(|(idx, name)| {
                        let idx = *idx;
                        async move {
                            let rx = VarWatcherAPI::lazy_subscribe(name).await;
                            (idx, rx)
                        }
                    })
                ).await;

                // Build initial value array
                let mut current_values: Vec<Dynamic> = vec![Dynamic::from(""); total_len];

                // Fill in static values
                for (key, val) in &static_values {
                    if let Ok(idx) = key.parse::<usize>() {
                        current_values[idx] = val.clone();
                    }
                }

                // Fill in current global var values
                for (idx, rx) in &receivers {
                    current_values[*idx] = Dynamic::from(rx.borrow().clone());
                }

                loop {
                    // Wait for any receiver to change
                    let changed_idx = {
                        let futs: Vec<_> = receivers.iter_mut().map(|(idx, rx)| {
                            let idx = *idx;
                            Box::pin(async move {
                                rx.changed().await.ok();
                                idx
                            })
                        }).collect();

                        let (idx, _, _) = futures::future::select_all(futs).await;
                        idx
                    };

                    // Update the changed value
                    if let Some((_, rx)) = receivers.iter().find(|(i, _)| *i == changed_idx) {
                        current_values[changed_idx] = Dynamic::from(rx.borrow().clone());
                    }

                    // Call the closure with the current array
                    let args_array: Array = current_values.clone();
                    // TODO: call closure with args_array and handle result
                    // e.g. update the bound var:
                    // let result = closure.call::<Dynamic>(&engine, &ast, (args_array,));
                    // VarWatcherAPI::update(&unique_name_clone, result.to_string());
                }
            });

            Ok(GlobalVar {
                name: unique_name,
                initial: Dynamic::from(""),
                additional: Some(bindings),
            })
        },
    );

    // == Top-level macros ==
    engine.register_fn(
        "defwindow",
        |name: &str, props: Map, node: WidgetNode| -> Result<WidgetNode, Box<EvalAltResult>> {
            Ok(WidgetNode::DefWindow { name: name.to_string(), props, node: Box::new(node) })
        },
    );

    engine.register_fn("poll", |var: &str, props: Map| -> Result<WidgetNode, Box<EvalAltResult>> {
        Ok(WidgetNode::Poll { var: var.to_string(), props })
    });

    engine.register_fn(
        "listen",
        |var: &str, props: Map| -> Result<WidgetNode, Box<EvalAltResult>> {
            Ok(WidgetNode::Listen { var: var.to_string(), props })
        },
    );

    let all_nodes_clone = all_nodes.clone();
    engine.register_fn(
        "tree",
        move |ctx: NativeCallContext, children: Array| -> Result<(), Box<EvalAltResult>> {
            let children_vec = children_to_vec(children, &ctx)?;
            let node = WidgetNode::Tree(children_vec);

            all_nodes_clone.borrow_mut().push(node);

            Ok(())
        },
    );
}
