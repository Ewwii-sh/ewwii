use crate::error::format_rhai_error;
use anyhow::{anyhow, Result};
use ewwii_shared_util::general_helper::*;
use rhai::Engine;
use std::collections::HashSet;

pub fn extract_poll_and_listen_vars(code: &str) -> Result<Vec<(String, Option<String>)>> {
    let mut results = Vec::new();
    let mut engine = Engine::new();

    register_temp_poll_listen(&mut engine);

    for expr in extract_poll_listen_exprs(code) {
        match engine.eval_expression::<TempSignal>(&expr) {
            Ok(sig) => {
                let initial = sig.props.get("initial").and_then(|v| v.clone().try_cast::<String>());
                results.push((sig.var, initial));
            }
            Err(e) => {
                return Err(anyhow::anyhow!(format_rhai_error(&e, code)));
            }
        }
    }

    Ok(results)
}

pub fn extract_poll_listen_exprs(code: &str) -> Vec<String> {
    let mut exprs = Vec::new();
    let mut i = 0;
    let chars: Vec<_> = code.chars().collect();

    while i < chars.len() {
        if code[i..].starts_with("poll(") || code[i..].starts_with("listen(") {
            let start = i;
            let mut depth = 0;

            while i < chars.len() {
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }

            let end = i;
            exprs.push(code[start..end].to_string());
        } else {
            i += 1;
        }
    }

    exprs
}

#[derive(Debug, Clone)]
struct TempSignal {
    pub var: String,
    pub props: rhai::Map,
}

fn register_temp_poll_listen(engine: &mut rhai::Engine) {
    engine.register_type::<TempSignal>();

    engine.register_fn("poll", |var: &str, props: rhai::Map| TempSignal { var: var.to_string(), props });

    engine.register_fn("listen", |var: &str, props: rhai::Map| TempSignal { var: var.to_string(), props });
}