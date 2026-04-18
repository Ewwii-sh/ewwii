use crate::prop::{Callback, Property};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct GlobalVar {
    pub name: String,
    pub initial: Property,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct GlobalCompare {
    pub name: String,
    pub vars: Vec<Property>,
    pub closure: Callback,
}

impl GlobalVar {
    pub fn from(name: String, initial: Property) -> Self {
        Self { name, initial }
    }
}
