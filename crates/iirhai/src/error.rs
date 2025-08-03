use colored::Colorize;
use rhai::{EvalAltResult, Position};

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
}

impl<'a> Diagnostic<'a> {
    fn render(&self) -> String {
        let num_width = self.line.to_string().len();

        // header (error: error_mssg)
        let mut out = format!(
            "\n{}: {}\n",
            self.severity.red().bold(),
            self.message
        );

        // arrow location
        out.push_str(&format!(
            "  {arrow} {file}:{line}:{col}\n",
            arrow = "-->".dimmed(),
            file  = self.file,
            line  = self.line,
            col   = self.column,
        ));

        // bar seperator
        out.push_str(&format!("   {bar}\n", bar = "|".dimmed()));

        out.push_str(&format!(
            "{:>width$} {sep} {}\n",
            self.line,
            self.line_text,
            width = num_width,
            sep   = "|".dimmed(),
        ));

        // The caret line, pointing at the column
        let caret_padding = " ".repeat(self.column.saturating_sub(1));
        out.push_str(&format!(
            "{:>width$} {sep} {padding}{}\n",
            "",
            "^".red().bold(),
            width   = num_width,
            sep     = "|".dimmed(),
            padding = caret_padding,
        ));

        // separator before notes
        out.push_str(&format!("   {bar}\n", bar = "|".dimmed()));

        // Optional help and hint
        if let Some(help) = &self.help {
            out.push_str(&format!(
                "{eq} help: {}\n",
                help.cyan(),
                eq = "=".cyan().bold(),
            ));
        }
        if let Some(hint) = &self.hint {
            out.push_str(&format!(
                "{eq} hint: {}\n",
                hint.cyan(),
                eq = "=".cyan().bold(),
            ));
        }

        out
    }
}

pub fn format_rhai_error(error: &EvalAltResult, code: &str) -> String {
    let pos = get_deepest_position(error);
    let line = pos.line().unwrap_or(0);
    let column = pos.position().unwrap_or(1);
    let line_text = code
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("");

    let filename = "<rhai>"; // DUMMY
    let help_hint = get_error_hint_help(error);

    let diag = Diagnostic {
        severity: "error",
        message: error.to_string(),
        file: filename,
        line,
        column,
        line_text,
        help:   if help_hint.help.is_empty() { None } else { Some(help_hint.help) },
        hint:   if help_hint.hint.is_empty() { None } else { Some(help_hint.hint) },
    };

    diag.render()
}


fn get_deepest_position(error: &EvalAltResult) -> Position {
    match error {
        EvalAltResult::ErrorInFunctionCall(_, _, inner, _) => get_deepest_position(inner),
        _ => error.position(),
    }
}

fn get_error_hint_help(err: &EvalAltResult) -> ErrorHelp {
    match err {
        EvalAltResult::ErrorParsing(..) => ErrorHelp {
            help: "Syntax error encountered while parsing.".into(),
            hint: "Check for unmatched tokens, invalid constructs, or misplaced punctuation.".into(),
        },
        EvalAltResult::ErrorVariableExists(name, ..) => ErrorHelp {
            help: format!("Variable '{}' is already defined.", name),
            hint: "Remove or rename the duplicate declaration.".into(),
        },
        EvalAltResult::ErrorForbiddenVariable(name, ..) => ErrorHelp {
            help: format!("Usage of forbidden variable '{}'.", name),
            hint: "Avoid using reserved or protected variable names.".into(),
        },
        EvalAltResult::ErrorVariableNotFound(name, ..) => ErrorHelp {
            help: format!("Unknown variable '{}'.", name),
            hint: "Check for typos or ensure the variable is initialized before use.".into(),
        },
        EvalAltResult::ErrorPropertyNotFound(name, ..) => ErrorHelp {
            help: format!("Property '{}' not found on this object.", name),
            hint: "Verify the property name and the object’s available fields.".into(),
        },
        EvalAltResult::ErrorFunctionNotFound(name, ..) => ErrorHelp {
            help: format!("Function '{}' is not defined.", name),
            hint: "Check spelling or ensure the function is registered or imported.".into(),
        },
        EvalAltResult::ErrorModuleNotFound(name, ..) => ErrorHelp {
            help: format!("Module '{}' could not be located.", name),
            hint: "Verify the module path and that it is included in your imports.".into(),
        },
        EvalAltResult::ErrorInFunctionCall(fn_name, msg, ..) => ErrorHelp {
            help: format!("Error inside function '{}': {}", fn_name, msg),
            hint: "Inspect the function implementation and arguments passed.".into(),
        },
        EvalAltResult::ErrorInModule(name, ..) => ErrorHelp {
            help: format!("Error while loading module '{}'.", name),
            hint: "Check the module code for syntax or runtime errors.".into(),
        },
        EvalAltResult::ErrorUnboundThis(..) => ErrorHelp {
            help: "`this` is unbound in this context.".into(),
            hint: "Only use `this` inside methods or bound closures.".into(),
        },
        EvalAltResult::ErrorMismatchDataType(found, expected, ..) => ErrorHelp {
            help: format!("Data type mismatch: found '{}', expected '{}'.", found, expected),
            hint: "Convert or cast values to the required type.".into(),
        },
        EvalAltResult::ErrorMismatchOutputType(found, expected, ..) => ErrorHelp {
            help: format!("Return type mismatch: found '{}', expected '{}'.", found, expected),
            hint: "Ensure your function returns the correct type.".into(),
        },
        EvalAltResult::ErrorIndexingType(typ, ..) => ErrorHelp {
            help: format!("Cannot index into value of type '{}'.", typ),
            hint: "Only arrays, maps, bitfields, or strings support indexing.".into(),
        },
        EvalAltResult::ErrorArrayBounds(len, idx, ..) => ErrorHelp {
            help: format!("Array index {} out of bounds (0..{}).", idx, len),
            hint: "Use a valid index within the array’s range.".into(),
        },
        EvalAltResult::ErrorStringBounds(len, idx, ..) => ErrorHelp {
            help: format!("String index {} out of bounds (0..{}).", idx, len),
            hint: "Ensure you index only valid character positions.".into(),
        },
        EvalAltResult::ErrorBitFieldBounds(len, idx, ..) => ErrorHelp {
            help: format!("Bitfield index {} out of bounds (0..{}).", idx, len),
            hint: "Use a valid bit position within the bitfield’s size.".into(),
        },
        EvalAltResult::ErrorFor(..) => ErrorHelp {
            help: "`for` loop value is not iterable.".into(),
            hint: "Iterate only over arrays, strings, ranges, or iterators.".into(),
        },
        EvalAltResult::ErrorDataRace(name, ..) => ErrorHelp {
            help: format!("Data race detected on '{}'.", name),
            hint: "Avoid shared mutable data or use synchronization primitives.".into(),
        },
        EvalAltResult::ErrorAssignmentToConstant(name, ..) => ErrorHelp {
            help: format!("Cannot assign to constant '{}'.", name),
            hint: "Constants cannot be reassigned after declaration.".into(),
        },
        EvalAltResult::ErrorDotExpr(field, ..) => ErrorHelp {
            help: format!("Invalid member access '{}'.", field),
            hint: "Verify the object has this member or method.".into(),
        },
        EvalAltResult::ErrorArithmetic(msg, ..) => ErrorHelp {
            help: "Arithmetic error encountered.".into(),
            hint: msg.clone(),
        },
        EvalAltResult::ErrorTooManyOperations(..) => ErrorHelp {
            help: "Script exceeded the maximum number of operations.".into(),
            hint: "Break complex expressions into smaller steps or increase the limit.".into(),
        },
        EvalAltResult::ErrorTooManyModules(..) => ErrorHelp {
            help: "Too many modules have been loaded.".into(),
            hint: "Use fewer modules or increase the module limit.".into(),
        },
        EvalAltResult::ErrorStackOverflow(..) => ErrorHelp {
            help: "Call stack overflow detected.".into(),
            hint: "Check for infinite recursion or deeply nested calls.".into(),
        },
        EvalAltResult::ErrorDataTooLarge(name, ..) => ErrorHelp {
            help: format!("Data '{}' is too large to handle.", name),
            hint: "Use smaller data sizes or adjust engine limits.".into(),
        },
        EvalAltResult::ErrorTerminated(..) => ErrorHelp {
            help: "Script execution was terminated.".into(),
            hint: "This occurs when a `stop` or external termination is triggered.".into(),
        },
        EvalAltResult::ErrorCustomSyntax(msg, options, ..) => ErrorHelp {
            help: format!("Custom syntax error: {}.", msg),
            hint: format!("Expected one of: {}.", options.join(", ")),
        },
        EvalAltResult::ErrorRuntime(..) => ErrorHelp {
            help: "Runtime error encountered.".into(),
            hint: "Inspect the error message and script logic for issues.".into(),
        },
        EvalAltResult::LoopBreak(..) => ErrorHelp {
            help: "`break` used outside of a loop.".into(),
            hint: "Only use `break` inside `for` or `while` loops.".into(),
        },
        EvalAltResult::Return(..) => ErrorHelp {
            help: "`return` statement encountered.".into(),
            hint: "Script terminated with an explicit return value.".into(),
        },
        _ => ErrorHelp {
            help: "Unknown error".into(),
            hint: "No additional information available for this error.".into(),
        }
    }
}

struct ErrorHelp {
    help: String,
    hint: String,
}
