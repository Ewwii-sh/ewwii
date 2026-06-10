//! This module provides data structures and enumrates that are used
//! as a bridge between the communication of both the plugin and the host.

use std::sync::Arc;

mod ipc;
mod registration;
mod shared;
mod library;

pub use ipc::*;
pub use registration::*;
pub use shared::*;
pub use library::*;

// === handlers === //

pub type ConfigCallbackFn = Arc<dyn Fn(&str, &str) + Send + Sync>;

pub trait ConfigCallbackFnExt {
    fn new<F>(f: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static;
}

impl ConfigCallbackFnExt for ConfigCallbackFn {
    fn new<F>(f: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        Arc::new(f)
    }
}
