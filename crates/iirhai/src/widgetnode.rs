#[derive(Debug, Clone)]
pub enum WidgetNode {
    Label(String),
    Row(Vec<WidgetNode>),
    Box { dir: String, children: Vec<WidgetNode> },
}