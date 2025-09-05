use crate::ast::WidgetNode;
use rhai::{Array, Engine, EvalAltResult, Map, NativeCallContext};
use std::rc::Rc;
use std::cell::RefCell;

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
    register_primitive!("slider", Slider);
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
    register_with_children!("centerbox", CenterBox);
    register_with_children!("expander", Expander);
    register_with_children!("revealer", Revealer);
    register_with_children!("scroll", Scroll);
    register_with_children!("overlay", OverLay);
    register_with_children!("stack", Stack);
    register_with_children!("eventbox", EventBox);
    register_with_children!("tooltip", ToolTip);

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
        "enter",
        move |ctx: NativeCallContext, children: Array| -> Result<(), Box<EvalAltResult>> {
            let children_vec = children_to_vec(children, &ctx)?;
            let node = WidgetNode::Enter(children_vec);

            all_nodes_clone.borrow_mut().push(node);

            Ok(())
        },
    );
}
