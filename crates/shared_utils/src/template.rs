use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use jaq_interpret::FilterT;

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum TemplateExpr {
    Literal(String),
    Var(String),
    Concat(Vec<TemplateExpr>),
    IfElse {
        condition: Box<TemplateExpr>,
        if_true: Box<TemplateExpr>,
        if_false: Box<TemplateExpr>,
    },
    BinOp {
        op: TemplateOp,
        left: Box<TemplateExpr>,
        right: Box<TemplateExpr>,
    },
    Index {
        expr: Box<TemplateExpr>,
        key: Box<TemplateExpr>,
    },
    FunctionCall {
        name: String,
        args: Vec<TemplateExpr>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum TemplateOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Mod,
    Gt,
    Lt,
    Gte,
    Lte,
    And,
    Or,
    Elvis,
    RegexMatch,
}

impl TemplateExpr {
    /// Collect all variable names this expression depends on
    pub fn collect_vars(&self) -> Vec<String> {
        match self {
            TemplateExpr::Var(name) => vec![name.clone()],
            TemplateExpr::Literal(_) => vec![],
            TemplateExpr::Concat(parts) => parts.iter().flat_map(|p| p.collect_vars()).collect(),
            TemplateExpr::IfElse {
                condition,
                if_true,
                if_false,
            } => {
                let mut vars = condition.collect_vars();
                vars.extend(if_true.collect_vars());
                vars.extend(if_false.collect_vars());
                vars
            }
            TemplateExpr::BinOp { left, right, .. } => {
                let mut vars = left.collect_vars();
                vars.extend(right.collect_vars());
                vars
            }
            TemplateExpr::Index { expr, key } => {
                let mut vars = expr.collect_vars();
                vars.extend(key.collect_vars());
                vars
            }
            TemplateExpr::FunctionCall { args, .. } => {
                args.iter().flat_map(|arg| arg.collect_vars()).collect()
            }
        }
    }

    /// Evaluate the expression given a map of variable values
    pub fn eval(&self, vars: &HashMap<String, String>) -> Result<String, String> {
        match self {
            TemplateExpr::Literal(s) => Ok(s.clone()),

            TemplateExpr::Var(name) => {
                vars.get(name).cloned().ok_or_else(|| format!("Variable not found: {}", name))
            }

            TemplateExpr::Concat(parts) => parts
                .iter()
                .map(|p| p.eval(vars))
                .collect::<Result<Vec<_>, _>>()
                .map(|parts| parts.join("")),

            TemplateExpr::IfElse {
                condition,
                if_true,
                if_false,
            } => {
                let cond = condition.eval_as_bool(vars)?;
                if cond {
                    if_true.eval(vars)
                } else {
                    if_false.eval(vars)
                }
            }

            TemplateExpr::BinOp { op, left, right } => {
                // try numeric first, fall back to string
                let l = left.eval(vars)?;
                let r = right.eval(vars)?;

                match op {
                    TemplateOp::Add => {
                        if let (Ok(lf), Ok(rf)) = (l.parse::<f64>(), r.parse::<f64>()) {
                            Ok(format_number(lf + rf))
                        } else {
                            Ok(format!("{}{}", l, r))
                        }
                    }
                    TemplateOp::Sub => eval_numeric(l, r, |a, b| a - b),
                    TemplateOp::Mul => eval_numeric(l, r, |a, b| a * b),
                    TemplateOp::Div => {
                        let (lf, rf) = parse_numeric(&l, &r)?;
                        if rf == 0.0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(format_number(lf / rf))
                        }
                    }
                    TemplateOp::Eq => Ok((l == r).to_string()),
                    TemplateOp::NotEq => Ok((l != r).to_string()),
                    TemplateOp::Gt => eval_cmp(l, r, |a, b| a > b),
                    TemplateOp::Lt => eval_cmp(l, r, |a, b| a < b),
                    TemplateOp::Gte => eval_cmp(l, r, |a, b| a >= b),
                    TemplateOp::Lte => eval_cmp(l, r, |a, b| a <= b),
                    TemplateOp::And => {
                        let lb = left.eval_as_bool(vars)?;
                        let rb = right.eval_as_bool(vars)?;
                        Ok((lb && rb).to_string())
                    }
                    TemplateOp::Or => {
                        let lb = left.eval_as_bool(vars)?;
                        let rb = right.eval_as_bool(vars)?;
                        Ok((lb || rb).to_string())
                    }
                    TemplateOp::Mod => eval_numeric(l, r, |a, b| a % b),
                    TemplateOp::Elvis => {
                        if !l.is_empty() && l != "false" && l != "0" {
                            Ok(l)
                        } else {
                            Ok(r)
                        }
                    }
                    TemplateOp::RegexMatch => {
                        let re =
                            regex::Regex::new(&r).map_err(|e| format!("Invalid regex: {}", e))?;
                        Ok(re.is_match(&l).to_string())
                    }
                }
            }
            TemplateExpr::Index { expr, key } => {
                let base = expr.eval(vars)?;
                let key_str = key.eval(vars)?;

                let json: serde_json::Value =
                    serde_json::from_str(&base).map_err(|e| format!("Not valid JSON: {}", e))?;

                let result = if let Ok(idx) = key_str.parse::<usize>() {
                    json.get(idx)
                } else {
                    json.get(&key_str)
                };

                match result {
                    Some(serde_json::Value::String(s)) => Ok(s.clone()),
                    Some(v) => Ok(v.to_string()),
                    None => Err(format!("Index '{}' not found", key_str)),
                }
            }
            TemplateExpr::FunctionCall { name, args } => {
                let evaluated_args = args
                    .iter()
                    .map(|arg| arg.eval(vars))
                    .collect::<Result<Vec<_>, _>>()?;

                eval_template_function(name, &evaluated_args)
            }
        }
    }

    fn eval_as_bool(&self, vars: &HashMap<String, String>) -> Result<bool, String> {
        let s = self.eval(vars)?;
        match s.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => s
                .parse::<f64>()
                .map(|n| n != 0.0)
                .map_err(|_| format!("Cannot interpret '{}' as bool", s)),
        }
    }

    pub fn parse(input: &str) -> Result<Self, String> {
        let mut exprs = Vec::new();
        let mut remaining = input;

        while !remaining.is_empty() {
            if let Some(start_idx) = remaining.find('{') {
                if start_idx > 0 {
                    exprs.push(TemplateExpr::Literal(remaining[..start_idx].to_string()));
                }

                remaining = &remaining[start_idx + 1..];

                if let Some(end_idx) = remaining.find('}') {
                    let inner_expr_str = &remaining[..end_idx];
                    remaining = &remaining[end_idx + 1..];

                    let inner_expr = Self::parse_inner_expression(inner_expr_str.trim())?;
                    exprs.push(inner_expr);
                } else {
                    return Err("Mismatched curly braces: missing closing '}'".to_string());
                }
            } else {
                exprs.push(TemplateExpr::Literal(remaining.to_string()));
                break;
            }
        }

        match exprs.len() {
            0 => Ok(TemplateExpr::Literal(String::new())),
            1 => Ok(exprs.pop().unwrap()),
            _ => Ok(TemplateExpr::Concat(exprs)),
        }
    }

    fn parse_inner_expression(input: &str) -> Result<Self, String> {
        let input = input.trim();
        if input.is_empty() {
            return Err("Empty expression inside brackets".to_string());
        }

        if input.starts_with('(') && input.ends_with(')') && is_balanced(&input[1..input.len() - 1]) {
            return Self::parse_inner_expression(&input[1..input.len() - 1]);
        }

        // Parse function calls: name(arg1, arg2)
        if let Some(open_paren) = input.find('(') {
            if input.ends_with(')') {
                let name = input[..open_paren].trim();
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let args_str = &input[open_paren + 1..input.len() - 1];
                    let args = parse_function_args(args_str)?;
                    return Ok(TemplateExpr::FunctionCall {
                        name: name.to_string(),
                        args,
                    });
                }
            }
        }

        // Handle ternary '?' and ':'
        if let Some((q_idx, c_idx)) = find_ternary_operators(input) {
            let condition_str = &input[..q_idx];
            let true_str = &input[q_idx + 1..c_idx];
            let false_str = &input[c_idx + 1..];

            return Ok(TemplateExpr::IfElse {
                condition: Box::new(Self::parse_inner_expression(condition_str)?),
                if_true: Box::new(Self::parse_inner_expression(true_str)?),
                if_false: Box::new(Self::parse_inner_expression(false_str)?),
            });
        }

        let op_groups = [
            vec![("||", TemplateOp::Or), ("&&", TemplateOp::And)],
            vec![("==", TemplateOp::Eq), ("!=", TemplateOp::NotEq)],
            vec![
                (">=", TemplateOp::Gte),
                ("<=", TemplateOp::Lte),
                (">", TemplateOp::Gt),
                ("<", TemplateOp::Lt),
            ],
            vec![("+", TemplateOp::Add), ("-", TemplateOp::Sub)],
            vec![("*", TemplateOp::Mul), ("/", TemplateOp::Div), ("%", TemplateOp::Mod)],
            vec![("?:", TemplateOp::Elvis), ("~=", TemplateOp::RegexMatch)],
        ];

        for ops in op_groups.iter() {
            if let Some((idx, op_str, op_enum)) = find_operator_outside_quotes(input, ops) {
                let left_str = &input[..idx];
                let right_str = &input[idx + op_str.len()..];

                return Ok(TemplateExpr::BinOp {
                    op: op_enum,
                    left: Box::new(Self::parse_inner_expression(left_str)?),
                    right: Box::new(Self::parse_inner_expression(right_str)?),
                });
            }
        }

        // Parse array/object index access: expr[key]
        if let Some(bracket_idx) = find_outer_index(input) {
            let base_str = &input[..bracket_idx];
            let key_str = &input[bracket_idx + 1..input.len() - 1];
            return Ok(TemplateExpr::Index {
                expr: Box::new(Self::parse_inner_expression(base_str)?),
                key: Box::new(Self::parse_inner_expression(key_str)?),
            });
        }

        // Parse dot index notation: expr.key
        if let Some(dot_idx) = find_outer_dot(input) {
            let base_str = &input[..dot_idx];
            let key_str = &input[dot_idx + 1..];
            return Ok(TemplateExpr::Index {
                expr: Box::new(Self::parse_inner_expression(base_str)?),
                key: Box::new(TemplateExpr::Literal(key_str.to_string())),
            });
        }

        if input.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return Ok(TemplateExpr::Literal(input.to_string()));
        }

        if ((input.starts_with('"') && input.ends_with('"'))
            || (input.starts_with('\'') && input.ends_with('\''))
            || (input.starts_with('`') && input.ends_with('`')))
            && input.len() >= 2
        {
            return Ok(TemplateExpr::Literal(input[1..input.len() - 1].to_string()));
        }

        Ok(TemplateExpr::Var(input.to_string()))
    }
}

pub fn eval_template_function(name: &str, args: &[String]) -> Result<String, String> {
    match name {
        "get_env" => match args {
            [var_name] => Ok(std::env::var(var_name).unwrap_or_default()),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "round" => match args {
            [num_str, digits_str] => {
                let num = parse_f64(num_str)?;
                let digits = parse_i32(digits_str)? as usize;
                Ok(format!("{:.1$}", num, digits))
            }
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "floor" => match args {
            [num_str] => Ok(format_number(parse_f64(num_str)?.floor())),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "ceil" => match args {
            [num_str] => Ok(format_number(parse_f64(num_str)?.ceil())),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "min" => match args {
            [a, b] => Ok(format_number(f64::min(parse_f64(a)?, parse_f64(b)?))),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "max" => match args {
            [a, b] => Ok(format_number(f64::max(parse_f64(a)?, parse_f64(b)?))),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "powi" => match args {
            [num, n] => Ok(format_number(f64::powi(parse_f64(num)?, parse_i32(n)?))),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "powf" => match args {
            [num, n] => Ok(format_number(f64::powf(parse_f64(num)?, parse_f64(n)?))),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "sin" => match args {
            [num] => Ok(format_number(parse_f64(num)?.sin())),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "cos" => match args {
            [num] => Ok(format_number(parse_f64(num)?.cos())),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "tan" => match args {
            [num] => Ok(format_number(parse_f64(num)?.tan())),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "cot" => match args {
            [num] => Ok(format_number(1.0 / parse_f64(num)?.tan())),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "degtorad" => match args {
            [num] => Ok(format_number(parse_f64(num)?.to_radians())),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "radtodeg" => match args {
            [num] => Ok(format_number(parse_f64(num)?.to_degrees())),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "matches" => match args {
            [string, pattern] => {
                let re = regex::Regex::new(pattern).map_err(|e| e.to_string())?;
                Ok(re.is_match(string).to_string())
            }
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "replace" => match args {
            [string, pattern, replacement] => {
                let re = regex::Regex::new(pattern).map_err(|e| e.to_string())?;
                let clean_repl = replacement.replace('$', "$$").replace('\\', "$");
                Ok(re.replace_all(string, clean_repl.as_str()).into_owned())
            }
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "substring" => match args {
            [string, start, len] => {
                let start_idx = parse_i32(start)?.max(0) as usize;
                let length = parse_i32(len)?.max(0) as usize;
                let result: String = string.chars().skip(start_idx).take(length).collect();
                Ok(result)
            }
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "search" => match args {
            [string, pattern] => {
                let re = regex::Regex::new(pattern).map_err(|e| e.to_string())?;
                let matches: Vec<serde_json::Value> = re
                    .find_iter(string)
                    .map(|m| serde_json::Value::String(m.as_str().to_string()))
                    .collect();
                Ok(serde_json::Value::Array(matches).to_string())
            }
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "captures" => match args {
            [string, pattern] => {
                let re = regex::Regex::new(pattern).map_err(|e| e.to_string())?;
                let captures: Vec<serde_json::Value> = re
                    .captures_iter(string)
                    .map(|cap| {
                        let inner = cap
                            .iter()
                            .flatten()
                            .map(|m| serde_json::Value::String(m.as_str().to_string()))
                            .collect();
                        serde_json::Value::Array(inner)
                    })
                    .collect();
                Ok(serde_json::Value::Array(captures).to_string())
            }
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "strlength" => match args {
            [string] => Ok(string.len().to_string()),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "arraylength" => match args {
            [json_str] => {
                let val: serde_json::Value = serde_json::from_str(json_str).map_err(|e| e.to_string())?;
                let arr = val.as_array().ok_or_else(|| "Value is not a JSON array".to_string())?;
                Ok(arr.len().to_string())
            }
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "objectlength" => match args {
            [json_str] => {
                let val: serde_json::Value = serde_json::from_str(json_str).map_err(|e| e.to_string())?;
                let obj = val.as_object().ok_or_else(|| "Value is not a JSON object".to_string())?;
                Ok(obj.len().to_string())
            }
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "jq" => match args {
            [json_str, code] => run_jaq_adapter(json_str, code, ""),
            [json_str, code, jq_args] => run_jaq_adapter(json_str, code, jq_args),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "formattime" => match args {
            [timestamp, format, timezone] => {
                let ts_sec = parse_i64(timestamp)?;

                let ts = jiff::Timestamp::from_second(ts_sec)
                    .map_err(|_| "Invalid UNIX timestamp".to_string())?;

                let tz = jiff::tz::TimeZone::get(timezone)
                    .map_err(|_| "Invalid timezone".to_string())?;

                Ok(ts.to_zoned(tz).strftime(format).to_string())
            }
            [timestamp, format] => {
                let ts_sec = parse_i64(timestamp)?;

                let ts = jiff::Timestamp::from_second(ts_sec)
                    .map_err(|_| "Invalid UNIX timestamp".to_string())?;

                let tz = jiff::tz::TimeZone::system();

                Ok(ts.to_zoned(tz).strftime(format).to_string())
            }
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "log" => match args {
            [num, n] => Ok(format_number(f64::log(parse_f64(num)?, parse_f64(n)?))),
            _ => Err(format!("Wrong arg count for function '{}'", name)),
        },
        "formatbytes" => {
            let (bytes, short, mode) = match args {
                [bytes] => (parse_i64(bytes)?, false, "iec"),
                [bytes, short] => (parse_i64(bytes)?, parse_bool(short)?, "iec"),
                [bytes, short, mode] => (parse_i64(bytes)?, parse_bool(short)?, mode.as_str()),
                _ => return Err(format!("Wrong arg count for function '{}'", name)),
            };
            let neg = bytes < 0;
            let byte_size = bytes.unsigned_abs();

            let formatted = match mode {
                "iec" => format_bytes_iec(byte_size, short),
                "si" => format_bytes_si(byte_size, short),
                _ => return Err(format!("Invalid byte format mode: {}", mode)),
            };

            Ok(if neg { format!("-{}", formatted) } else { formatted })
        }
        _ => Err(format!("Unknown function: {}", name)),
    }
}

// Adapts `run_jaq_function` output into a standard Result<String, String>
fn run_jaq_adapter(json_str: &str, code: &str, args: &str) -> Result<String, String> {
    let json_val: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON input for jq: {}", e))?;

    let results = run_jaq_function(json_val, code.to_string(), args)
        .map_err(|e| format!("jaq execution error: {:?}", e))?;

    if results.is_empty() {
        Ok(String::new())
    } else if results.len() == 1 {
        Ok(results[0].to_string())
    } else {
        Ok(serde_json::to_string(&results).unwrap_or_default())
    }
}

#[cached(size = 10, result = true, sync_writes = true)]
fn prepare_jaq_filter(code: String) -> Result<Arc<jaq_interpret::Filter>, EvalError> {
    let (filter, mut errors) = jaq_parse::parse(&code, jaq_parse::main());
    let filter = match filter {
        Some(x) => x,
        None => {
            let err_msg = errors.pop().map(|e| e.to_string());
            return Err(EvalError::JaqParseError(Box::new(JaqParseError(err_msg))));
        }
    };
    let mut defs = jaq_interpret::ParseCtx::new(Vec::new());
    defs.insert_natives(jaq_core::core());
    defs.insert_defs(jaq_std::std());

    let filter = defs.compile(filter);

    if let Some(error) = errors.pop() {
        return Err(EvalError::JaqParseError(Box::new(JaqParseError(Some(error.to_string())))));
    }
    Ok(Arc::new(filter))
}

fn run_jaq_function(json: serde_json::Value, code: String, args: &str) -> Result<Vec<DynVal>, EvalError> {
    use jaq_interpret::{Ctx, RcIter, Val};
    prepare_jaq_filter(code)?
        .run((Ctx::new([], &RcIter::new(std::iter::empty())), Val::from(json)))
        .map(|r| r.map(Into::<serde_json::Value>::into))
        .map(|x| {
            x.map(|val| match (args, val) {
                ("r", serde_json::Value::String(s)) => DynVal::from_string(s),
                // invalid arguments are silently ignored
                (_, v) => DynVal::from_string(serde_json::to_string(&v).unwrap()),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| EvalError::JaqError(e.to_string()))
}

fn format_bytes_iec(bytes: u64, short: bool) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    const SHORT_UNITS: &[&str] = &["B", "K", "M", "G", "T", "P"];

    format_units(bytes, 1024.0, if short { SHORT_UNITS } else { UNITS })
}

fn format_bytes_si(bytes: u64, short: bool) -> String {
    const UNITS: &[&str] = &["B", "kB", "MB", "GB", "TB", "PB"];
    const SHORT_UNITS: &[&str] = &["B", "k", "M", "G", "T", "P"];

    format_units(bytes, 1000.0, if short { SHORT_UNITS } else { UNITS })
}

fn format_units(bytes: u64, base: f64, units: &[&str]) -> String {
    if bytes == 0 {
        return format!("0 {}", units[0]);
    }

    let i = (bytes as f64).log(base).floor() as usize;
    let i = i.min(units.len() - 1);
    let val = (bytes as f64) / base.powi(i as i32);

    format!("{:.2} {}", val, units[i])
}

fn parse_f64(val: &str) -> Result<f64, String> {
    val.parse::<f64>().map_err(|_| format!("'{}' is not a valid float", val))
}

fn parse_i32(val: &str) -> Result<i32, String> {
    val.parse::<i32>().map_err(|_| format!("'{}' is not a valid integer", val))
}

fn parse_i64(val: &str) -> Result<i64, String> {
    val.parse::<i64>().map_err(|_| format!("'{}' is not a valid 64-bit integer", val))
}

fn parse_bool(val: &str) -> Result<bool, String> {
    match val {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(format!("'{}' is not a valid boolean", val)),
    }
}

fn parse_numeric(l: &str, r: &str) -> Result<(f64, f64), String> {
    let lf = l.parse::<f64>().map_err(|_| format!("'{}' is not a number", l))?;
    let rf = r.parse::<f64>().map_err(|_| format!("'{}' is not a number", r))?;
    Ok((lf, rf))
}

fn eval_numeric(l: String, r: String, op: impl Fn(f64, f64) -> f64) -> Result<String, String> {
    let (lf, rf) = parse_numeric(&l, &r)?;
    Ok(format_number(op(lf, rf)))
}

fn eval_cmp(l: String, r: String, op: impl Fn(f64, f64) -> bool) -> Result<String, String> {
    if let (Ok(lf), Ok(rf)) = (l.parse::<f64>(), r.parse::<f64>()) {
        Ok(op(lf, rf).to_string())
    } else {
        Ok(op(l.len() as f64, r.len() as f64).to_string())
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

fn is_balanced(s: &str) -> bool {
    let mut depth = 0;
    let mut in_quote = None;

    for c in s.chars() {
        match (c, in_quote) {
            ('"' | '\'' | '`', None) => in_quote = Some(c),
            (q, Some(cur)) if q == cur => in_quote = None,
            ('(', None) => depth += 1,
            (')', None) => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0 && in_quote.is_none()
}

fn find_operator_outside_quotes<'a>(
    input: &str,
    ops: &[(&'a str, TemplateOp)],
) -> Option<(usize, &'a str, TemplateOp)> {
    let mut parens = 0;
    let mut in_quote = None;
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();

    for i in (0..len).rev() {
        let c = chars[i];
        match (c, in_quote) {
            ('"' | '\'' | '`', None) => in_quote = Some(c),
            (q, Some(cur)) if q == cur => in_quote = None,
            (')', None) => parens += 1,
            ('(', None) => parens -= 1,
            _ if in_quote.is_none() && parens == 0 => {
                for (op_str, op_enum) in ops {
                    if input[i..].starts_with(op_str) {
                        return Some((i, op_str, op_enum.clone()));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn find_ternary_operators(input: &str) -> Option<(usize, usize)> {
    let mut parens = 0;
    let mut in_quote = None;
    let mut q_idx = None;

    for (i, c) in input.char_indices() {
        match (c, in_quote) {
            ('"' | '\'' | '`', None) => in_quote = Some(c),
            (q, Some(cur)) if q == cur => in_quote = None,
            ('(', None) => parens += 1,
            (')', None) => parens -= 1,
            ('?', None) if parens == 0 && q_idx.is_none() => q_idx = Some(i),
            (':', None) if parens == 0 && q_idx.is_some() => return Some((q_idx.unwrap(), i)),
            _ => {}
        }
    }
    None
}

fn find_outer_index(input: &str) -> Option<usize> {
    if !input.ends_with(']') {
        return None;
    }
    let mut parens = 0;
    let mut brackets = 0;
    let mut in_quote = None;
    let mut escaped = false;
    let chars: Vec<(usize, char)> = input.char_indices().collect();

    for (i, c) in chars {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && in_quote.is_some() {
            escaped = true;
            continue;
        }

        match (c, in_quote) {
            ('\'' | '"' | '`', None) => in_quote = Some(c),
            (q, Some(cur)) if q == cur => in_quote = None,
            ('[', None) => {
                if brackets == 0 && parens == 0 && i > 0 {
                    // Check if this outer bracket set spans to the end
                    if is_matching_outer_bracket(input, i) {
                        return Some(i);
                    }
                }
                brackets += 1;
            }
            (']', None) => brackets -= 1,
            ('(', None) => parens += 1,
            (')', None) => parens -= 1,
            _ => {}
        }
    }
    None
}

fn is_matching_outer_bracket(input: &str, start_idx: usize) -> bool {
    let mut depth = 0;
    let mut in_quote = None;
    let mut escaped = false;

    for (i, c) in input[start_idx..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && in_quote.is_some() {
            escaped = true;
            continue;
        }
        match (c, in_quote) {
            ('\'' | '"' | '`', None) => in_quote = Some(c),
            (q, Some(cur)) if q == cur => in_quote = None,
            ('[', None) => depth += 1,
            (']', None) => {
                depth -= 1;
                if depth == 0 {
                    return start_idx + i == input.len() - 1;
                }
            }
            _ => {}
        }
    }
    false
}

fn find_outer_dot(input: &str) -> Option<usize> {
    let mut parens = 0;
    let mut brackets = 0;
    let mut in_quote = None;
    let bytes = input.as_bytes();

    for i in (0..bytes.len()).rev() {
        let b = bytes[i] as char;
        match (b, in_quote) {
            ('"' | '\'' | '`', None) => in_quote = Some(b),
            (q, Some(cur)) if q == cur => in_quote = None,
            _ if in_quote.is_some() => {}
            (')', None) => parens += 1,
            ('(', None) => parens -= 1,
            (']', None) => brackets += 1,
            ('[', None) => brackets -= 1,
            ('.', None) if parens == 0 && brackets == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

fn parse_function_args(input: &str) -> Result<Vec<TemplateExpr>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut parens = 0;
    let mut in_quote = None;

    for c in input.chars() {
        match (c, in_quote) {
            ('"' | '\'' | '`', None) => {
                in_quote = Some(c);
                current.push(c);
            }
            (q, Some(cur)) if q == cur => {
                in_quote = None;
                current.push(c);
            }
            ('(', None) => {
                parens += 1;
                current.push(c);
            }
            (')', None) => {
                parens -= 1;
                current.push(c);
            }
            (',', None) if parens == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    args.push(TemplateExpr::parse_inner_expression(trimmed)?);
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        args.push(TemplateExpr::parse_inner_expression(trimmed)?);
    }

    Ok(args)
}

// placeholder definitions for local compilation checks
#[derive(Debug)]
pub enum EvalError {
    JaqParseError(Box<JaqParseError>),
    JaqError(String),
}

#[derive(Debug)]
pub struct JaqParseError(pub Option<String>);

#[derive(Debug, Clone)]
pub struct DynVal(String);

impl DynVal {
    pub fn from_string(s: String) -> Self {
        DynVal(s)
    }
}

impl std::fmt::Display for DynVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl serde::Serialize for DynVal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}


