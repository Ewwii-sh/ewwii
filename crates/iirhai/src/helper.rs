use std::collections::HashSet;
use ewwii_shared_util::general_helper::*;
use anyhow::{anyhow, Result};
use crate::error::format_rhai_error;
use rhai::Engine;

pub fn extract_poll_and_listen_vars(code: &str) -> Result<Vec<(String, Option<String>)>> {
    let mut results = Vec::new();
    let mut engine = Engine::new();

    register_temp_poll_listen(&mut engine);

    for expr in extract_poll_listen_exprs(code) {
        match engine.eval_expression::<TempSignal>(&expr) {
            Ok(sig) => {
                let initial = sig
                    .props
                    .get("initial")
                    .and_then(|v| v.clone().try_cast::<String>());
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

    engine.register_fn("poll", |var: &str, props: rhai::Map| {
        TempSignal { var: var.to_string(), props }
    });

    engine.register_fn("listen", |var: &str, props: rhai::Map| {
        TempSignal { var: var.to_string(), props }
    });
}

#[cfg(test)]
mod tests {
    use crate::helper::extract_poll_and_listen_vars;
    #[test]
    fn poll_listen_regex_test() {
        let result = extract_poll_and_listen_vars(
            r#"
            fn widget1() {
                return box(#{
                    class: 'widget1',
                    orientation: 'h',
                    space_evenly: true,
                    halign: 'start',
                    spacing: 5
                }, [
                label(#{ text: 'Hello Ewwii!' }),
                slider(#{ min: 0, max: 101, value: 3, onchange: 'echo hi' }), 
                button(#{ onclick: 'notify-send 'hello there!'', label: 'greet' }),
                label(#{ text: cpu_usage }),
                ]);
            };

            enter([
                poll('cpu_usage', #{ 
                    interval: '1s', 
                    cmd: 'echo hi',  
                    initial: 'initial' 
                }),
                listen('net_speed', #{ 
                    cmd: 'while true; do date +%T; sleep 1; done'
                }),

                defwindow('main_window', #{
                    monitor: 0,
                    windowtype: 'dock',
                    geometry: #{ x: '0px', y: '0px', width: '10px', height: '20px' },
                }, widget1())
            ]);

    "#,
        );
        println!("{:#?}", result);
        assert!(result.contains("cpu_usage"));
        assert!(result.contains("net_speed"));
    }
}
