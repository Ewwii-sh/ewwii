use codespan_reporting::diagnostic;
use ewwii_shared_util::{Span, Spanned};
use crate::dynval;
use thiserror::Error;

pub type DiagResult<T> = Result<T, DiagError>;

#[derive(Debug, Error)]
#[error("{}", .0.to_message())]
pub struct DiagError(pub diagnostic::Diagnostic<usize>);

static_assertions::assert_impl_all!(DiagError: Send, Sync);
static_assertions::assert_impl_all!(dynval::ConversionError: Send, Sync);
static_assertions::assert_impl_all!(lalrpop_util::ParseError < usize, lexer::Token, parse_error::ParseError>: Send, Sync);

impl<T: ToDiagnostic> From<T> for DiagError {
    fn from(x: T) -> Self {
        Self(x.to_diagnostic())
    }
}

impl DiagError {
    pub fn note(self, note: &str) -> Self {
        DiagError(self.0.with_note(note.to_string()))
    }
}

pub trait DiagResultExt<T> {
    fn note(self, note: &str) -> DiagResult<T>;
}

impl<T> DiagResultExt<T> for DiagResult<T> {
    fn note(self, note: &str) -> DiagResult<T> {
        self.map_err(|e| e.note(note))
    }
}
