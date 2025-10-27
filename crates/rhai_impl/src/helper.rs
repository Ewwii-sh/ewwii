use crate::error::format_eval_error;
use anyhow::Result;
use rhai::Engine;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

pub fn extract_poll_and_listen_vars(code: &str) -> Result<Vec<(String, Option<String>)>> {
    extract_poll_and_listen_vars_inner(code, &mut HashSet::new())
}

fn extract_poll_and_listen_vars_inner(
    code: &str,
    visited: &mut HashSet<PathBuf>,
) -> Result<Vec<(String, Option<String>)>> {
    let mut results = Vec::new();
    let mut engine = Engine::new();
    register_temp_poll_listen(&mut engine);

    // Handle imports manually
    for import_path in extract_import_paths(code)? {
        let resolved = resolve_import_path(&import_path)?;

        // Prevent infinite recursion
        let canonical = fs::canonicalize(&resolved).unwrap_or(resolved.clone());
        if visited.contains(&canonical) {
            continue;
        }

        visited.insert(canonical.clone());

        if resolved.exists() {
            let imported_code = fs::read_to_string(&resolved)?;
            let inner = extract_poll_and_listen_vars_inner(&imported_code, visited)?;
            results.extend(inner);
        }
    }

    // Process this file’s own poll/listen calls
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
                return Err(anyhow::anyhow!(format_eval_error(
                    &e,
                    code,
                    &engine,
                    None
                )));
            }
        }
    }

    Ok(results)
}

/// Extract import paths from the Rhai source code
fn extract_import_paths(code: &str) -> Result<Vec<String>> {
    let mut imports = Vec::new();

    for line in code.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("import ") {
            // Example: import "widgets/foo" as _foo;
            if let Some(start) = trimmed.find('"') {
                if let Some(end_rel) = trimmed[start + 1..].find('"') {
                    let end = start + 1 + end_rel;
                    let path = &trimmed[start + 1..end];
                    imports.push(path.to_string());
                }
            }
        }
    }

    Ok(imports)
}

/// Resolve relative and absolute import paths.
fn resolve_import_path(import_path: &str) -> Result<PathBuf> {
    let path = Path::new(import_path);
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    let abs = if abs.extension().is_none() {
        abs.with_extension("rhai")
    } else {
        abs
    };

    Ok(abs)
}

pub fn extract_poll_listen_exprs(code: &str) -> Vec<String> {
    let mut exprs = Vec::new();
    let mut i = 0;
    let code_bytes = code.as_bytes();
    let len = code.len();

    while i < len {
        // skipping comments
        if code[i..].starts_with("//") {
            while i < len && code_bytes[i] as char != '\n' {
                i += 1;
            }
            i += 1; // skip a full line
            continue;
        }

        if code[i..].starts_with("/*") {
            i += 2;
            while i + 1 < len && &code[i..i + 2] != "*/" {
                i += 1;
            }
            i += 2; // skip till `*/` closing
            continue;
        }

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

    engine.register_fn("poll", |var: &str, props: rhai::Map| TempSignal {
        var: var.to_string(),
        props,
    });

    engine.register_fn("listen", |var: &str, props: rhai::Map| TempSignal {
        var: var.to_string(),
        props,
    });
}
