use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{self, termcolor::Buffer};
use nbcl::context::EvalContext;
use nbcl::error::{NbclError, Span};

pub fn handle_nbcl_err(
    err: NbclError,
    code: &str,
    file_id: Option<&str>,
    maybe_ctx: Option<EvalContext>,
) -> String {
    let mut file_id: Option<String> = file_id.map(str::to_string);
    if let Some(ctx) = maybe_ctx {
        if let Some(new) = ctx.get_current_file().and_then(|f| f.to_str().map(str::to_string)) {
            file_id = Some(new);
        }
    }

    match err {
        NbclError::Parse { message, hint, span } => format_codespan_error(
            "Parse Error",
            code,
            message,
            hint,
            span.unwrap_or(Span::dummy()),
            file_id,
        ),
        NbclError::Ast { message, hint, span } => format_codespan_error(
            "Ast Error",
            code,
            message,
            hint,
            span.unwrap_or(Span::dummy()),
            file_id,
        ),
        NbclError::IO { message, hint, path: _ } => {
            format_codespan_error("IO Error", code, message, hint, Span::dummy(), file_id)
        }
        NbclError::Runtime { message, hint, span } => format_codespan_error(
            "Runtime Error",
            code,
            message,
            hint,
            span.unwrap_or(Span::dummy()),
            file_id,
        ),
    }
}

/// Return a formatted error as a String
fn format_codespan_error(
    err_label: &str,
    code: &str,
    message: String,
    hint: Option<String>,
    span: Span,
    file_id: Option<String>,
) -> String {
    let mut files = SimpleFiles::new();
    let file_id = files.add(file_id.unwrap_or("<nbcl>".to_string()), code);

    // build the notes
    let mut notes = Vec::new();
    if let Some(hint) = hint {
        notes.push(format!("hint: {}", hint));
    }

    let new_msg = format!("[{}] at {}:{}: {}", err_label, span.line, span.col, &message);

    // build the diagnostic error
    let mut labels = Vec::new();
    if span.start != span.end {
        labels.push(Label::primary(file_id, span.start..span.end).with_message(&new_msg));
    } else {
        let range = line_span(code, span.line);
        labels.push(Label::primary(file_id, range).with_message(&new_msg));
    }

    let diagnostic =
        Diagnostic::error().with_message(&new_msg).with_labels(labels).with_notes(notes);

    let mut buffer = Buffer::ansi();
    let config = term::Config::default();

    term::emit(&mut buffer, &config, &files, &diagnostic).unwrap();

    // Convert buffer to string
    String::from_utf8(buffer.into_inner()).unwrap()
}

fn line_span(code: &str, line_number: usize) -> std::ops::Range<usize> {
    let mut offset = 0;
    for (i, line) in code.lines().enumerate() {
        if i == line_number - 1 {
            return offset..offset + line.len();
        }
        offset += line.len() + 1;
    }
    0..code.len()
}
