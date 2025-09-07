use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{self, termcolor::Buffer};
use rhai::{Engine, EvalAltResult, ParseError};
use rhai_trace::{BetterError, Span};

/// Return a formatted Rhai evaluation error.
pub fn format_eval_error(error: &EvalAltResult, code: &str, engine: &Engine) -> String {
    let better_error =
        BetterError::improve_eval_error(error, code, engine, None).unwrap_or(BetterError {
            message: error.to_string(),
            help: None,
            hint: None,
            note: None,
            span: Span::new(0, 0, 0, 0),
        });
    format_codespan_error(better_error, code)
}

/// Return a formatted Rhai parse error.
pub fn format_parse_error(error: &ParseError, code: &str) -> String {
    let better_error = BetterError::improve_parse_error(error, code).unwrap_or(BetterError {
        message: error.to_string(),
        help: None,
        hint: None,
        note: None,
        span: Span::new(0, 0, 0, 0),
    });
    format_codespan_error(better_error, code)
}

/// Return a formatted error as a String
pub fn format_codespan_error(be: BetterError, code: &str) -> String {
    let mut files = SimpleFiles::new();
    let file_id = files.add("<rhai>", code);

    // build the notes
    let mut notes = Vec::new();
    if let Some(help) = &be.help {
        notes.push(format!("help: {}", help));
    }
    if let Some(hint) = &be.hint {
        notes.push(format!("hint: {}", hint));
    }
    if let Some(note) = &be.note {
        notes.push(format!("note: {}", note));
    }

    // build the diagnostic error
    let diagnostic = Diagnostic::error()
        .with_message(&be.message)
        .with_labels(vec![
            Label::primary(file_id, be.span.start()..be.span.end()).with_message(&be.message)
        ])
        .with_notes(notes);

    let mut buffer = Buffer::ansi();
    let config = term::Config::default();

    term::emit(&mut buffer, &config, &files, &diagnostic).unwrap();

    // Convert buffer to string
    String::from_utf8(buffer.into_inner()).unwrap()
}
