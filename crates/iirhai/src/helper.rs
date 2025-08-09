use crate::error::format_rhai_error;
use anyhow::Result;
use rhai::Engine;

// TODO: use the cmd of poll as the initial value of initial is not found.
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
    let code_bytes = code.as_bytes();
    let len = code.len();

    while i < len {
        if code[i..].starts_with("poll(") || code[i..].starts_with("listen(") {
            let start = i;
            let mut depth = 0;
            let mut j = i;

            while j < len {
                match code.as_bytes()[j] as char {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            j += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }

            let end = j;
            if let Some(expr) = code.get(start..end) {
                exprs.push(expr.to_string());
            }
            i = j;
        } else {
            i += code[i..].chars().next().unwrap().len_utf8();
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
