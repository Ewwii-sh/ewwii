use serde::{Deserialize, Serialize};

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

/// A type interface for Nbcl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NbclType {
    String,
    Int,
    Float,
    Bool,
    Array,
    Any,
    Null,
}

/// This enumrate provides improved error support for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginError {
    /// Error at FFI or data serialization/deserialization
    BridgeError(String),
    /// Error during registration
    RegistrationError(String),
    /// Error during parsing
    ParseError(String),
    /// Some unknown internal error
    Internal(String),
}

impl From<String> for PluginError {
    fn from(msg: String) -> Self {
        PluginError::Internal(msg)
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::BridgeError(msg) => write!(f, "Bridge error: {}", msg),
            PluginError::RegistrationError(msg) => write!(f, "Registration error: {}", msg),
            PluginError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            PluginError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}
