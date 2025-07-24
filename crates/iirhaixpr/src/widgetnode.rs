#[derive(Debug, Clone)]
pub enum WidgetNode {
    Label(Spanned<String>),
    Row(Spanned<Vec<WidgetNode>>),
    Box { dir: Spanned<String>, children: Spanned<Vec<WidgetNode>> },
}