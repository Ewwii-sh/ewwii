//! This module provides data structures and enumrates that are used
//! as a bridge between the communication of both the plugin and the host.

use std::sync::Arc;
use serde::{Serialize, Deserialize};

// === Shared Implementation === //

#[repr(C)]
pub struct PluginInfo {
    pub id: &'static str,
    pub version: &'static str,
}

impl PluginInfo {
    pub fn new(id: &'static str, version: &'static str) -> Self {
        Self { id, version }
    }

    pub fn builder() -> PluginInfoBuilder {
        PluginInfoBuilder::default()
    }
}

#[derive(Default)]
pub struct PluginInfoBuilder {
    id: &'static str,
    version: &'static str,
}

impl PluginInfoBuilder {
    pub fn id(mut self, id: &'static str) -> Self {
        self.id = id;
        self
    }

    pub fn version(mut self, version: &'static str) -> Self {
        self.version = version;
        self
    }

    pub fn build(self) -> PluginInfo {
        PluginInfo::new(self.id, self.version)
    }
}

// === register_function implementation === //

/// An enumrate providing options for
/// function registaration namespaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FnNamespace {
    Custom(String),
    Global,
}

/// Used for Plugin and host communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<PluginValue>),
    Null,
}

/// Handler for handling Native Function registeration in rhai
pub type NativeFn = 
    Arc<dyn Fn(Vec<PluginValue>) -> Result<PluginValue, String> + Send + Sync>;

