use eww_shared_util::Span;

use crate::{error::DiagError, parser::ast::Ast};

#[derive(thiserror::Error, Debug)]
pub enum FilesError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    DiagError(#[from] DiagError),
}

pub trait DuckFileProvider {
    fn load_duck_file(&mut self, path: std::path::PathBuf) -> Result<(Span, Vec<Ast>), FilesError>;
    fn load_duck_str(&mut self, name: String, content: String) -> Result<(Span, Vec<Ast>), DiagError>;
    fn unload(&mut self, id: usize);
}
