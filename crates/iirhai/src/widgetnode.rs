use rhai::Map;

#[derive(Debug, Clone)]
pub enum WidgetNode {
    Label { props: Map },
    Box { props: Map, children: Vec<WidgetNode> },
    CenterBox { props: Map, children: Vec<WidgetNode> },
    Button { props: Map },
    Image { props: Map },
    Input { props: Map },
    Progress { props: Map },
    ComboBoxText { props: Map },
    Slider { props: Map },
    Checkbox { props: Map },
    Revealer { props: Map, children: Vec<WidgetNode> },
    Scroll { props: Map, children: Vec<WidgetNode> },
    Calendar { props: Map },
    Graph { props: Map },
    Include(String),
    DefStyle(String),
    EventBox { props: Map, children: Vec<WidgetNode> },

    // Top-level macros
    DefWindow { name: String, props: Map, node: Box<WidgetNode> },
    // Poll { var: String, interval: String, cmd: String, initial: String },
    // Listen { var: String, signal: String },
    Poll { var: String, props: Map },
    Listen { var: String, props: Map },
    Enter(Vec<WidgetNode>),
}
