use rhai::{Engine, Scope};
use crate::builtins::register_all_widgets;

// the node tree
use crate::widgetnode::WidgetNode;

pub fn parse_widget_code(code: &str) -> Result<WidgetNode, Box<rhai::EvalAltResult>> {
    let mut engine = Engine::new();
    register_all_widgets(&mut engine);
    let mut scope = Scope::new();
    match engine.eval_with_scope::<WidgetNode>(&mut scope, code) {
        Ok(widget) => Ok(widget),
        Err(e) => {
            eprintln!("Error: {}", e);
            let pos = e.position();
            if !pos.is_none() {
                eprintln!("At: line {}, column {}", pos.line().unwrap_or(0), pos.position().unwrap_or(0));
            }

            Err(e)
        }
    }
}
