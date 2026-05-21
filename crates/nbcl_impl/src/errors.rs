use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{self, termcolor::Buffer};
use nbcl::error::{NbclError, Span};

pub fn handle_nbcl_err(err: NbclError, code: &str, file_id: Option<&str>) -> String {
    match err {
        NbclError::Parse { message, hint, span } =>
            format_codespan_error("Parse Error", code, message, hint, span.unwrap_or(Span::dummy()), file_id),
        NbclError::Ast { message, hint, span } =>
            format_codespan_error("Ast Error", code, message, hint, span.unwrap_or(Span::dummy()), file_id),
        NbclError::IO { message, hint, path: _ } =>
            format_codespan_error("IO Error", code, message, hint, Span::dummy(), file_id),
        NbclError::Runtime { message, hint, span } =>
            format_codespan_error("Runtime Error", code, message, hint, span.unwrap_or(Span::dummy()), file_id),
    }
}

/// Return a formatted error as a String
fn format_codespan_error(
    err_label: &str,
    code: &str,
    message: String,
    hint: Option<String>,
    span: Span,
    file_id: Option<&str>
) -> String {
    let mut files = SimpleFiles::new();
    let file_id = files.add(file_id.unwrap_or("<nbcl>"), code);

    // build the notes
    let mut notes = Vec::new();
    if let Some(hint) = hint {
        notes.push(format!("hint: {}", hint));
    }

    let new_msg = format!("[{}] at {}:{}: {}", err_label, span.line, span.col, &message);

    // build the diagnostic error
    let mut labels = Vec::new();
    if span.start != span.end {
        labels.push(
            Label::primary(file_id, span.start..span.end).with_message(&new_msg),
        );
    }

    let diagnostic =
        Diagnostic::error().with_message(&new_msg).with_labels(labels).with_notes(notes);

    let mut buffer = Buffer::ansi();
    let config = term::Config::default();

    term::emit(&mut buffer, &config, &files, &diagnostic).unwrap();

    // Convert buffer to string
    String::from_utf8(buffer.into_inner()).unwrap()
}
