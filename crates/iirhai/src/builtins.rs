use crate::widgetnode::WidgetNode;
use rhai::{Array, Engine, Map};

pub fn register_all_widgets(engine: &mut Engine) {
    engine.register_type::<WidgetNode>();

    // Primitive widgets
    engine.register_fn("label", |props: Map| WidgetNode::Label { props });

    engine.register_fn("box", |props: Map, children: Array| WidgetNode::Box {
        props,
        children: children.into_iter().map(|v| v.cast()).collect(),
    });

    engine.register_fn("centerbox", |props: Map, children: Array| WidgetNode::CenterBox {
        props,
        children: children.into_iter().map(|v| v.cast()).collect(),
    });

    engine.register_fn("button", |props: Map| WidgetNode::Button { props });

    engine.register_fn("image", |props: Map| WidgetNode::Image { props });

    engine.register_fn("input", |props: Map| WidgetNode::Input { props });

    engine.register_fn("progress", |props: Map| WidgetNode::Progress { props });

    engine.register_fn("combo_box_text", |props: Map| WidgetNode::ComboBoxText { props });

    engine.register_fn("slider", |props: Map| WidgetNode::Slider { props });

    engine.register_fn("checkbox", |props: Map| WidgetNode::Checkbox { props });

    engine.register_fn("expander", |props: Map, children: Array| WidgetNode::Expander {
        props,
        children: children.into_iter().map(|v| v.cast()).collect(),
    });

    engine.register_fn("revealer", |props: Map, children: Array| WidgetNode::Revealer {
        props,
        children: children.into_iter().map(|v| v.cast()).collect(),
    });

    engine.register_fn("scroll", |props: Map, children: Array| WidgetNode::Scroll {
        props,
        children: children.into_iter().map(|v| v.cast()).collect(),
    });

    engine.register_fn("color_button", |props: Map| WidgetNode::ColorButton { props });

    engine.register_fn("color_chooser", |props: Map| WidgetNode::ColorChooser { props });

    engine.register_fn("calendar", |props: Map| WidgetNode::Calendar { props });

    engine.register_fn("graph", |props: Map| WidgetNode::Graph { props });

    engine.register_fn("transform", |props: Map| WidgetNode::Transform { props });

    engine.register_fn("circular_progress", |props: Map| WidgetNode::CircularProgress { props });

    engine.register_fn("include", |path: &str| {
        // TODO: load and eval another config file
        WidgetNode::Include(path.to_string())
    });

    engine.register_fn("defstyle", |style: &str| WidgetNode::DefStyle(style.to_string()));

    engine.register_fn("eventbox", |props: Map, children: Array| WidgetNode::EventBox {
        props,
        children: children.into_iter().map(|v| v.cast()).collect(),
    });

    engine.register_fn("tooltip", |children: Array| WidgetNode::ToolTip {
        children: children.into_iter().map(|v| v.cast()).collect(),
    });

    // --- Top-level macros ---

    /*  defwidget is not needed in rhai because it is an imprative language.
        Because of this, functions are basically defwidget itself!
        There is no requirement to build a function just to match yucks syntax.
    */
    // engine.register_fn("defwidget", |name: &str, node: WidgetNode| {
    //     WidgetNode::DefWidget {
    //         name: name.to_string(),
    //         node: Box::new(node),
    //     }
    // });

    engine.register_fn("defwindow", |name: &str, props: Map, node: WidgetNode| WidgetNode::DefWindow {
        name: name.to_string(),
        props,
        node: Box::new(node),
    });

    engine.register_fn("poll", |var: &str, props: Map| WidgetNode::Poll { var: var.to_string(), props });

    engine.register_fn("listen", |var: &str, props: Map| WidgetNode::Listen { var: var.to_string(), props });

    engine.register_fn("enter", |children: Array| WidgetNode::Enter(children.into_iter().map(|v| v.cast()).collect()));
}
