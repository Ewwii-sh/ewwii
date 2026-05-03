use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum TemplateExpr {
    Literal(String),
    Var(String),
    Concat(Vec<TemplateExpr>),
    IfElse { condition: Box<TemplateExpr>, if_true: Box<TemplateExpr>, if_false: Box<TemplateExpr> },
    BinOp { op: TemplateOp, left: Box<TemplateExpr>, right: Box<TemplateExpr> },
    Index { expr: Box<TemplateExpr>, key: Box<TemplateExpr> },
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
            TemplateExpr::IfElse { condition, if_true, if_false } => {
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

            TemplateExpr::IfElse { condition, if_true, if_false } => {
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

                let json: serde_json::Value = serde_json::from_str(&base)
                    .map_err(|e| format!("Not valid JSON: {}", e))?;

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
}

// helpers
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
