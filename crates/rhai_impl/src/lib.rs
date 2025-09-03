//! IIRhai is a simple crate which configures rhai for the `ewwii` widget system.
//!
//! This crate supports parsing, error handling, and has a custom module_resolver.

pub mod ast;
pub mod builtins;
mod dyn_id;
pub mod error;
pub mod helper;
pub mod module_resolver;
pub mod parser;
pub mod providers;
pub mod updates;
