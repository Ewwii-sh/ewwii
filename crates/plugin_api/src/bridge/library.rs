use super::NativeFn;
use super::NbclType;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub struct LibraryItem {
    pub(crate) name: String,
    pub(crate) functions: HashMap<String, LibraryFn>
}

pub struct LibraryFn {
    pub(crate) params: Vec<NbclType>,
    pub(crate) ret: NbclType,
    pub(crate) handler: NativeFn,
}

impl LibraryItem {
    /// Internal create new library method
    fn new(name: String) -> Self {
        Self { name, functions: HashMap::new() }
    }

    /// Start a new item definition
    pub fn define(name: impl Into<String>) -> Self {
        Self::new(name.into())
    }

    /// Chainable function registration
    pub fn with_fn(mut self, name: &str, params: Vec<NbclType>, ret: NbclType, f: NativeFn) -> Self {
        let lib_fn = LibraryFn {
            params,
            ret,
            handler: f,
        };
        self.functions.insert(name.to_string(), lib_fn);

        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryItemFFI {
    pub name: String,
    pub functions: HashMap<String, LibraryFnFFI>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryFnFFI {
    pub params: Vec<NbclType>,
    pub ret: NbclType,
    pub callback_id: u64,
}
