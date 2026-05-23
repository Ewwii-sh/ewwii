use crate::prop::Property;
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
