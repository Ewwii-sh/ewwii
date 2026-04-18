use crate::ast::WidgetNode;
use rhai::{Array, Engine, EvalAltResult, FnPtr, Map, NativeCallContext};
use ewwii_shared_utils::variables::GlobalCompare;
use ewwii_shared_utils::prop::{Property, PropertyMap, Callback};
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
    engine.register_type::<GlobalCompare>();

    // == Primitive widgets ==
    macro_rules! register_primitive {
        ($name:expr, $variant:ident) => {
            engine.register_fn($name, |props: Map| -> Result<WidgetNode, Box<EvalAltResult>> {
                let prop_map = PropertyMap::from_rhai(props);
                Ok(WidgetNode::$variant { props: prop_map })
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
                    let prop_map = PropertyMap::from_rhai(props);
                    Ok(WidgetNode::$variant { props: prop_map, children: children_vec })
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
            let mut props = PropertyMap::new();
            props.insert("file", path.into());
            props.insert("id", load.into());
            Ok(WidgetNode::GtkUI { props })
        },
    );

    engine.register_fn(
        "bound",
        |variables: Array, closure: FnPtr| -> Result<GlobalCompare, Box<EvalAltResult>> {
            let handle = rand::random::<u64>();
            let unique_name = format!("\0__globalbound__{}", &handle);
            let vars = variables
                .into_iter()
                .map(Property::from_dynamic)
                .collect();

            crate::callback::register_callback(handle, closure.clone());

            let callback = Callback {
                name: closure.fn_name().to_string(),
                handle: Some(handle),
            };

            Ok(GlobalCompare {
                name: unique_name,
                vars,
                closure: callback,
            })
        }
    );

    // == Top-level macros ==
    engine.register_fn(
        "defwindow",
        |name: &str, props: Map, node: WidgetNode| -> Result<WidgetNode, Box<EvalAltResult>> {
            let prop_map = PropertyMap::from_rhai(props);
            Ok(WidgetNode::DefWindow { 
                name: name.to_string(), 
                props: prop_map, 
                node: Box::new(node) 
            })
        },
    );

    engine.register_fn("poll", |var: &str, props: Map| -> Result<WidgetNode, Box<EvalAltResult>> {
        let prop_map = PropertyMap::from_rhai(props);
        Ok(WidgetNode::Poll { var: var.to_string(), props: prop_map })
    });

    engine.register_fn(
        "listen",
        |var: &str, props: Map| -> Result<WidgetNode, Box<EvalAltResult>> {
            let prop_map = PropertyMap::from_rhai(props);
            Ok(WidgetNode::Listen { var: var.to_string(), props: prop_map })
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
