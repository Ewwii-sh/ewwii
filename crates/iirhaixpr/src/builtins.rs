use rhai::{Engine, Array};

// the node tree
use crate::widgetnode::WidgetNode;

pub fn register_all_widgets(engine: &mut Engine) {
    engine.register_type::<WidgetNode>();

    engine.register_fn("label", |text: &str| {
        WidgetNode::Label(text.to_string())
    });

    engine.register_fn("row", |children: Array| {
        let children = children
            .into_iter()
            .map(|v| v.cast::<WidgetNode>())
            .collect::<Vec<_>>();

        WidgetNode::Row(children)
    });

    engine.register_fn("box", |dir: &str, children: Array| {
        let children = children
            .into_iter()
            .map(|v| v.cast::<WidgetNode>())
            .collect::<Vec<_>>();

        WidgetNode::Box {
            dir: dir.to_string(),
            children,
        }
    });
}
