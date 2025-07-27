#[macro_export]
macro_rules! gen_diagnostic {
    ( $(kind = $kind:expr,)?
      $(msg = $msg:expr)?
      $(, label = $span:expr $(=> $label:expr)?)?
      $(, note = $note:expr)? $(,)?
    ) => {
        ::codespan_reporting::diagnostic::Diagnostic::new(gen_diagnostic! {
            @macro_fallback $({$kind})? {::codespan_reporting::diagnostic::Severity::Error}
        })
            $(.with_message($msg.to_string()))?
            $(.with_labels(vec![
                ::codespan_reporting::diagnostic::Label::primary($span.2, $span.0..$span.1)
                    $(.with_message($label))?
            ]))?
            $(.with_notes(vec![$note.to_string()]))?
    };
    ($msg:expr $(, $span:expr $(,)?)?) => {{
        ::codespan_reporting::diagnostic::Diagnostic::error()
            .with_message($msg.to_string())
            $(.with_labels(vec![::codespan_reporting::diagnostic::Label::primary($span.2, $span.0..$span.1)]))?
    }};


    (@macro_fallback { $value:expr } { $fallback:expr }) => {
        $value
    };
    (@macro_fallback { $fallback:expr }) => {
        $fallback
    };
}