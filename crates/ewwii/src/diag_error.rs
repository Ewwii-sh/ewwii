use codespan_reporting::diagnostic;
// use shared_utils::{Span, Spanned};
use crate::dynval;
use thiserror::Error;

// pub type DiagResult<T> = Result<T, DiagError>;

#[derive(Debug, Error)]
// #[error("{}", .0.to_message())] // old one
#[error("{:?}", .0)]
pub struct DiagError(pub diagnostic::Diagnostic<usize>);

static_assertions::assert_impl_all!(DiagError: Send, Sync);
static_assertions::assert_impl_all!(dynval::ConversionError: Send, Sync);

// /// Code used by yuck I suppose.
// impl<T: ToDiagnostic> From<T> for DiagError {
//     fn from(x: T) -> Self {
//         Self(x.to_diagnostic())
//     }
// }

// impl DiagError {
//     pub fn note(self, note: &str) -> Self {
//         DiagError(self.0.with_notes(vec![note.to_string()]))
//     }
// }

// pub trait DiagResultExt<T> {
//     fn note(self, note: &str) -> DiagResult<T>;
// }

// impl<T> DiagResultExt<T> for DiagResult<T> {
//     fn note(self, note: &str) -> DiagResult<T> {
//         self.map_err(|e| e.note(note))
//     }
// }
