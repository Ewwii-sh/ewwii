use colored::*;
use rhai::{EvalAltResult, Position};

/// Format a Rhai evaluation error with code context, Rust-style.
pub fn format_rhai_error(error: &EvalAltResult, code: &str) -> String {
    let mut msg = format!("\n{}: {}", "error".red().bold(), error.to_string());
    
    let pos = get_deepest_position(error);

    if let Some(line_num) = pos.line() {
        let col_num = pos.position().unwrap_or(1);
        let line_text = code.lines().nth(line_num.saturating_sub(1)).unwrap_or("");

        let filename = None; // DUMMY
        let file = filename.unwrap_or("<rhai>");

        msg.push_str(&format!("\n  --> {}:{}:{}", file, line_num, col_num));
        msg.push_str("\n   |\n");

        msg.push_str(&format!("{:>3} | {}\n", line_num, line_text));
        msg.push_str(&format!("    | {:>width$}{}\n", "", "^".red().bold(), width = col_num.saturating_sub(1)));
    }

    msg
}

fn get_deepest_position(error: &EvalAltResult) -> Position {
    match error {
        EvalAltResult::ErrorInFunctionCall(_, _, inner, _) => get_deepest_position(inner),
        _ => error.position(),
    }
}