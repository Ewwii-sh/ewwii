use colored::Colorize;
use rhai::{Engine, EvalAltResult, Position};

/// A little helper to carry all the pieces of a single‐line diagnostic.
struct Diagnostic<'a> {
    severity: &'a str,
    message: String,
    file: &'a str,
    line: usize,
    column: usize,
    line_text: &'a str,
    help: Option<String>,
    hint: Option<String>,
    note: Option<String>,
}

impl<'a> Diagnostic<'a> {
    fn render(&self) -> String {
        let num_width = self.line.to_string().len();

        // header (error: error_mssg)
        let mut out = format!("\n{}: {}\n", self.severity.red().bold(), self.message);

        // arrow location
        out.push_str(&format!(
            "  {arrow} {file}:{line}:{col}\n",
            arrow = "-->".dimmed(),
            file = self.file,
            line = self.line,
            col = self.column,
        ));

        // bar seperator
        out.push_str(&format!("   {bar}\n", bar = "|".dimmed()));

        out.push_str(&format!("{:>width$} {sep} {}\n", self.line, self.line_text, width = num_width, sep = "|".dimmed(),));

        // The caret line, pointing at the column
        let caret_padding = " ".repeat(self.column.saturating_sub(1));
        out.push_str(&format!(
            "{:>width$} {sep} {padding}{}\n",
            "",
            "^".red().bold(),
            width = num_width,
            sep = "|".dimmed(),
            padding = caret_padding,
        ));

        // separator before notes
        out.push_str(&format!("   {bar}\n", bar = "|".dimmed()));

        // Optional help, hint and note
        if let Some(help) = &self.help {
            out.push_str(&format!("{eq} help: {}\n", help.cyan(), eq = "=".cyan().bold(),));
        }
        if let Some(hint) = &self.hint {
            out.push_str(&format!("{eq} hint: {}\n", hint.cyan(), eq = "=".cyan().bold(),));
        }
        if let Some(note) = &self.note {
            let label = " note ";
            let term_width = termsize::get().map(|size| size.cols as usize).unwrap_or(80);
            let wrap_width = term_width.saturating_sub(4);
            let note_lines: Vec<_> = textwrap::wrap(note, wrap_width);
            let width = note_lines.iter().map(|l| l.len()).max().unwrap_or(0);

            out.push_str(&format!(
                "{tl}{label}{tr}\n",
                tl = "╭─".green().bold(),
                label = label.green().bold(),
                // adding 1 to width because idk, without it it looks off for some reason
                tr = format!("{:─<1$}╮", "", (width + 1) - label.len()).green().bold()
            ));

            for line in note_lines {
                out.push_str(&format!("{v} {line:<width$} {v}\n", v = "│".green().bold(), line = line.green(), width = width));
            }

            out.push_str(&format!(
                "{bl}{line}╯\n",
                bl = "╰─".green().bold(),
                // same reason as about, without it it looks off
                line = "─".repeat(width + 1).green().bold()
            ));
        }

        out
    }
}

pub fn format_rhai_error(error: &EvalAltResult, code: &str, engine: &Engine) -> String {
    let pos = get_deepest_position(error);
    let line = pos.line().unwrap_or(0);
    let column = pos.position().unwrap_or(1);
    let line_text = code.lines().nth(line.saturating_sub(1)).unwrap_or("");

    let filename = "<rhai>"; // DUMMY
    let help_hint = get_error_info(get_root_cause(error), error, engine, code);

    let diag = Diagnostic {
        severity: "error",
        message: error.to_string(),
        file: filename,
        line,
        column,
        line_text,
        help: if help_hint.help.is_empty() { None } else { Some(help_hint.help) },
        hint: if help_hint.hint.is_empty() { None } else { Some(help_hint.hint) },
        note: if help_hint.note.is_empty() { None } else { Some(help_hint.note) },
    };

    diag.render()
}

fn get_deepest_position(error: &EvalAltResult) -> Position {
    match error {
        EvalAltResult::ErrorInFunctionCall(_, _, inner, _) => get_deepest_position(inner),
        EvalAltResult::ErrorInModule(_, inner, _) => get_deepest_position(inner),
        _ => error.position(),
    }
}

fn get_root_cause<'a>(err: &'a EvalAltResult) -> &'a EvalAltResult {
    match err {
        EvalAltResult::ErrorInFunctionCall(_, _, inner, _) => get_root_cause(inner),
        EvalAltResult::ErrorInModule(_, inner, _) => get_root_cause(inner),
        _ => err,
    }
}

fn get_error_info(root_err: &EvalAltResult, outer_err: &EvalAltResult, engine: &Engine, code: &str) -> ErrorHelp {
    let (help, hint) = match root_err {
        EvalAltResult::ErrorParsing(..) => (
            "Syntax error encountered while parsing.".into(),
            "Check for unmatched tokens, invalid constructs, or misplaced punctuation.".into(),
        ),
        EvalAltResult::ErrorVariableExists(name, ..) => {
            (format!("Variable '{}' is already defined.", name), "Remove or rename the duplicate declaration.".into())
        }
        EvalAltResult::ErrorForbiddenVariable(name, ..) => {
            (format!("Usage of forbidden variable '{}'.", name), "Avoid using reserved or protected variable names.".into())
        }
        EvalAltResult::ErrorVariableNotFound(name, ..) => {
            (format!("Unknown variable '{}'.", name), "Check for typos or ensure the variable is initialized before use.".into())
        }
        EvalAltResult::ErrorPropertyNotFound(name, ..) => (
            format!("Property '{}' not found on this object.", name),
            "Verify the property name and the object’s available fields.".into(),
        ),
        EvalAltResult::ErrorFunctionNotFound(fn_sig, ..) => {
            let base = fn_sig.split('(').next().unwrap_or(fn_sig).trim();

            // Might be a bit less performant but I gotta pay the price of
            // having "kinda good" errors with Rhai.
            let ast = match engine.compile(code) {
                Ok(ast) => ast,
                Err(err) => {
                    return ErrorHelp {
                        help: format!("Failed to compile code for suggestions: {}", err),
                        hint: String::new(),
                        note: String::new(),
                    };
                }
            };

            let candidates: Vec<String> = ast
                .iter_functions()
                .filter(|f| f.name == base)
                .map(|f| {
                    let params = f.params.join(", ");
                    format!("{}({})", f.name, params)
                })
                .collect();

            if !candidates.is_empty() {
                (
                    format!("Function '{}' not found with this argument list.", fn_sig),
                    format!("Did you mean one of:\n  {}", candidates.join("\n    ")),
                )
            } else {
                (format!("Function '{}' is not defined.", fn_sig), "Check spelling, module path, or argument count.".into())
            }
        }
        EvalAltResult::ErrorModuleNotFound(name, ..) => (
            format!("Module '{}' could not be located.", name),
            "Verify the module path and that it is included in your imports.".into(),
        ),
        EvalAltResult::ErrorInFunctionCall(fn_name, msg, ..) => (
            format!("Error inside function '{}': {}", fn_name, msg),
            "Inspect the function implementation and arguments passed.".into(),
        ),
        EvalAltResult::ErrorInModule(name, ..) => {
            (format!("Error while loading module '{}'.", name), "Check the module code for syntax or runtime errors.".into())
        }
        EvalAltResult::ErrorUnboundThis(..) => {
            ("`this` is unbound in this context.".into(), "Only use `this` inside methods or bound closures.".into())
        }
        EvalAltResult::ErrorMismatchDataType(found, expected, ..) => (
            format!("Data type mismatch: found '{}', expected '{}'.", found, expected),
            "Convert or cast values to the required type.".into(),
        ),
        EvalAltResult::ErrorMismatchOutputType(found, expected, ..) => (
            format!("Return type mismatch: found '{}', expected '{}'.", found, expected),
            "Ensure your function returns the correct type.".into(),
        ),
        EvalAltResult::ErrorIndexingType(typ, ..) => (
            format!("Cannot index into value of type '{}'.", typ),
            "Only arrays, maps, bitfields, or strings support indexing.".into(),
        ),
        EvalAltResult::ErrorArrayBounds(len, idx, ..) => {
            (format!("Array index {} out of bounds (0..{}).", idx, len), "Use a valid index within the array’s range.".into())
        }
        EvalAltResult::ErrorStringBounds(len, idx, ..) => (
            format!("String index {} out of bounds (0..{}).", idx, len),
            "Ensure you index only valid character positions.".into(),
        ),
        EvalAltResult::ErrorBitFieldBounds(len, idx, ..) => (
            format!("Bitfield index {} out of bounds (0..{}).", idx, len),
            "Use a valid bit position within the bitfield’s size.".into(),
        ),
        EvalAltResult::ErrorFor(..) => {
            ("`for` loop value is not iterable.".into(), "Iterate only over arrays, strings, ranges, or iterators.".into())
        }
        EvalAltResult::ErrorDataRace(name, ..) => {
            (format!("Data race detected on '{}'.", name), "Avoid shared mutable data or use synchronization primitives.".into())
        }
        EvalAltResult::ErrorAssignmentToConstant(name, ..) => {
            (format!("Cannot assign to constant '{}'.", name), "Constants cannot be reassigned after declaration.".into())
        }
        EvalAltResult::ErrorDotExpr(field, ..) => {
            (format!("Invalid member access '{}'.", field), "Verify the object has this member or method.".into())
        }
        EvalAltResult::ErrorArithmetic(msg, ..) => ("Arithmetic error encountered.".into(), msg.clone()),
        EvalAltResult::ErrorTooManyOperations(..) => (
            "Script exceeded the maximum number of operations.".into(),
            "Break complex expressions into smaller steps or increase the limit.".into(),
        ),
        EvalAltResult::ErrorTooManyModules(..) => {
            ("Too many modules have been loaded.".into(), "Use fewer modules or increase the module limit.".into())
        }
        EvalAltResult::ErrorStackOverflow(..) => {
            ("Call stack overflow detected.".into(), "Check for infinite recursion or deeply nested calls.".into())
        }
        EvalAltResult::ErrorDataTooLarge(name, ..) => {
            (format!("Data '{}' is too large to handle.", name), "Use smaller data sizes or adjust engine limits.".into())
        }
        EvalAltResult::ErrorTerminated(..) => {
            ("Script execution was terminated.".into(), "This occurs when a `stop` or external termination is triggered.".into())
        }
        EvalAltResult::ErrorCustomSyntax(msg, options, ..) => {
            (format!("Custom syntax error: {}.", msg), format!("Expected one of: {}.", options.join(", ")))
        }
        EvalAltResult::ErrorRuntime(..) => {
            ("Runtime error encountered.".into(), "Inspect the error message and script logic for issues.".into())
        }
        EvalAltResult::LoopBreak(..) => {
            ("`break` used outside of a loop.".into(), "Only use `break` inside `for` or `while` loops.".into())
        }
        EvalAltResult::Return(..) => {
            ("`return` statement encountered.".into(), "Script terminated with an explicit return value.".into())
        }
        _ => ("Unknown error".into(), "No additional information available for this error.".into()),
    };

    let note = match outer_err {
        EvalAltResult::ErrorInFunctionCall(fn_name, ..) =>
            format!("This error occurred during a call to '{}'. Inspecting the function implementation and arguments passed may help solve this error.", fn_name),

        EvalAltResult::ErrorInModule(mod_name, ..) =>
            format!("This happened while loading the module '{}'. Tip: Check the module code for syntax or runtime errors", mod_name),

        EvalAltResult::ErrorRuntime(..) =>
            "A runtime error bubbled up from a lower-level operation.".into(),

        _ => "".into(),
    };

    ErrorHelp { help, hint, note }
}

struct ErrorHelp {
    help: String,
    hint: String,
    note: String,
}
