use colored::*;
use rhai::{EvalAltResult, Position};

/// Format a Rhai evaluation error with code context, Rust-style.
pub fn format_rhai_error(error: &EvalAltResult, code: &str) -> String {
    let mut msg = format!(
        "\n{}: {}",
        "error".red().bold(),
        error.to_string()
    );

    if let Some(line_num) = error.position().line() {
        let col_num = error.position().position().unwrap_or(1);
        let line_text = code
            .lines()
            .nth(line_num.saturating_sub(1))
            .unwrap_or("");

        let filename = None; // DUMMY
        let file = filename.unwrap_or("<rhai>");

        msg.push_str(&format!("\n  --> {}:{}:{}", file, line_num, col_num));
        msg.push_str("\n   |\n");

        msg.push_str(&format!("{:>3} | {}\n", line_num, line_text));
        msg.push_str(&format!(
            "    | {:>width$}{}\n",
            "",
            "^".red().bold(),
            width = col_num.saturating_sub(1)
        ));
    }

    msg
}
