//! This module provides data structures and enumrates that are used
//! as a bridge between the communication of both the plugin and the host.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

/// This enumrate provides improved error support for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginError {
    /// Error at FFI or data serialization/deserialization
    BridgeError(String),
    /// Error during registration
    RegistrationError(String),
    /// Some unknown internal error
    Internal(String),
}

impl From<String> for PluginError {
    fn from(msg: String) -> Self {
        PluginError::Internal(msg)
    }
}

// === register_function implementation === //

/// Handler for handling Native Function registeration in rhai
pub type NativeFn = Arc<dyn Fn(Vec<PluginValue>) -> Result<PluginValue, String> + Send + Sync>;

pub trait NativeFnExt {
    fn new<F>(f: F) -> Self
    where
        F: Fn(Vec<PluginValue>) -> Result<PluginValue, String> + Send + Sync + 'static;
}

impl NativeFnExt for NativeFn {
    fn new<F>(f: F) -> Self
    where
        F: Fn(Vec<PluginValue>) -> Result<PluginValue, String> + Send + Sync + 'static,
    {
        Arc::new(f)
    }
}

// === register_config implementation === //
#[repr(C)]
pub struct ConfigInfo {
    pub extension: &'static str,
    pub main_file: &'static str,
}

pub type ConfigFn = Arc<dyn Fn() -> Result<ConfigInfo, String> + Send + Sync>;

pub trait ConfigFnExt {
    fn new<F>(f: F) -> Self
    where
        F: Fn() -> Result<ConfigInfo, String> + Send + Sync + 'static;
}

impl ConfigFnExt for ConfigFn {
    fn new<F>(f: F) -> Self
    where
        F: Fn() -> Result<ConfigInfo, String> + Send + Sync + 'static,
    {
        Arc::new(f)
    }
}

#[repr(C)]
pub struct CustomConfigEngine;
