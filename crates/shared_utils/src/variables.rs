use crate::prop::{Callback, Property};
use crate::template::TemplateExpr;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Global reactive variable.
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct GlobalVar {
    pub name: String,
    pub initial: Property,
    pub template: Option<TemplateExpr>,
}

/// Internal structure used by bound.
/// This is not meant to be used by plugins for dynamic variable interaction/mutation.
/// [`TemplateExpr`] can be used for that matter (property of GlobalVar struct).
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct GlobalCompare {
    pub name: String,
    pub vars: Vec<Property>,
    pub closure: Callback,
}

impl GlobalVar {
    pub fn from(name: String, initial: Property) -> Self {
        Self { name, initial, template: None }
    }

    pub fn set_template(&mut self, template: Option<TemplateExpr>) {
        self.template = template;
    }
}
