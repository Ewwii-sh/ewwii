use codespan_reporting::term;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::diagnostic::{Diagnostic, Label};

fn render_error(code: &str, error_pos: usize) {
    let mut files = SimpleFiles::new();
    let file_id = files.add("widget.rhai", code);

    let diagnostic = Diagnostic::error()
        .with_message("Syntax error")
        .with_labels(vec![
            Label::primary(file_id, error_pos..error_pos + 1)
                .with_message("Unexpected token here"),
        ]);

    let writer = std::io::stderr();
    let config = term::Config::default();
    term::emit(&mut std::io::stderr().lock(), &config, &files, &diagnostic).unwrap();
}
