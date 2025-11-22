//! Module exposing extra utilities for rhai.

#[cfg(feature = "include-rhai")]
mod rhai_included {
    /// _(include-rhai)_ An enumrate providing options for
    /// function registaration namespaces.
    pub enum RhaiFnNamespace {
        Custom(String),
        Global,
    }
}

#[cfg(feature = "include-rhai")]
pub use rhai_included::*;
