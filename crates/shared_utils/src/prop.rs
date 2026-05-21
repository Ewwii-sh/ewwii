use crate::variables::{GlobalCompare, GlobalVar};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use nbcl::Value;

/// A deterministic, serializable collection of widget properties.
/// Replaces rhai::Map in your WidgetNode.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Hash)]
pub struct PropertyMap(pub BTreeMap<String, Property>);

impl PropertyMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Converts an Nbcl property map into stable PropertyMap.
    pub fn from_nbcl(nbcl_map: HashMap<String, Value>) -> Self {
        let mut map = BTreeMap::new();
        for (k, v) in nbcl_map {
            map.insert(k.to_string(), Property::from_value(v));
        }
        Self(map)
    }

    /// Converts an actual Nbcl map into stable PropertyMap.
    pub fn from_nbcl_map(nbcl_map: Vec<(String, Value)>) -> Self {
        let mut map = BTreeMap::new();
        for (k, v) in nbcl_map {
            map.insert(k.to_string(), Property::from_value(v));
        }
        Self(map)
    }

    pub fn insert(&mut self, key: impl Into<String>, value: Property) {
        self.0.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Property> {
        self.0.get(key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Alternative to [`rhai::Dynamic`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Property {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Property>),
    Map(PropertyMap),
    Callback(Callback),

    // Custom Variants
    GlobalVar(Box<GlobalVar>),
    GlobalCompare(Box<GlobalCompare>),
}

/// Alternative to [`rhai::FnPtr`]
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Callback {
    pub name: String,
    pub handle: Option<u64>,
}

impl Property {
    pub fn from_value(value: Value) -> Self {
        return match value {
            Value::Int(v) => Self::Int(v),
            Value::Bool(v) => Self::Bool(v),
            Value::Float(v) => Self::Float(v),
            Value::Str(v) => Self::String(v),
            Value::List(v) => Self::Array(v.into_iter().map(|inner| Self::from_value(inner)).collect()),
            Value::Map(v) => Self::Map(PropertyMap::from_nbcl_map(v)),
            Value::Lambda(v) => Self::Callback(Callback { name: v, handle: None }),
            Value::Null => Self::None,
            _ => Self::None,
        }

        // Probably commented out temporarily (21-05-2026)
        // // Handle Variants
        // if let Some(var) = d.clone().try_cast::<GlobalVar>() {
        //     return Self::GlobalVar(Box::new(var));
        // }

        // if let Some(comp) = d.clone().try_cast::<GlobalCompare>() {
        //     return Self::GlobalCompare(Box::new(comp));
        // }
    }

    /// Returns the bool value if the property is a Bool
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    /// Returns the i64 value if the property is an Int
    pub fn as_int(&self) -> Option<i64> {
        if let Self::Int(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    /// Returns the f64 value if the property is a Float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Int(i) => Some(*i as f64), // Helpful auto-conversion
            _ => None,
        }
    }

    /// Returns a reference to the String if the property is a String
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }

    /// Returns a reference to the Vec if the property is an Array
    pub fn as_array(&self) -> Option<&[Property]> {
        if let Self::Array(a) = self {
            Some(a.as_slice())
        } else {
            None
        }
    }

    /// Returns a reference to the PropertyMap if the property is a Map
    pub fn as_map(&self) -> Option<&PropertyMap> {
        if let Self::Map(m) = self {
            Some(m)
        } else {
            None
        }
    }

    /// Returns a reference to the Callback if the property is a Callback
    pub fn as_callback(&self) -> Option<&Callback> {
        if let Self::Callback(c) = self {
            Some(c)
        } else {
            None
        }
    }

    /// Returns a reference to the GlobalVar (unboxes automatically)
    pub fn as_global_var(&self) -> Option<&GlobalVar> {
        if let Self::GlobalVar(v) = self {
            Some(v.as_ref())
        } else {
            None
        }
    }

    /// Returns a reference to the GlobalCompare (unboxes automatically)
    pub fn as_global_compare(&self) -> Option<&GlobalCompare> {
        if let Self::GlobalCompare(c) = self {
            Some(c.as_ref())
        } else {
            None
        }
    }
}

impl From<bool> for Property {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<i64> for Property {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<f64> for Property {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<String> for Property {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for Property {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<Vec<Property>> for Property {
    fn from(v: Vec<Property>) -> Self {
        Self::Array(v)
    }
}

impl From<PropertyMap> for Property {
    fn from(m: PropertyMap) -> Self {
        Self::Map(m)
    }
}

impl From<GlobalVar> for Property {
    fn from(v: GlobalVar) -> Self {
        Self::GlobalVar(Box::new(v))
    }
}

impl From<GlobalCompare> for Property {
    fn from(c: GlobalCompare) -> Self {
        Self::GlobalCompare(Box::new(c))
    }
}

impl Hash for Property {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Self::None => {}.hash(state),
            Self::Bool(b) => b.hash(state),
            Self::Int(i) => i.hash(state),
            // Convert float to bits for hashing as it doesnt supports it
            Self::Float(f) => f.to_bits().hash(state),
            Self::String(s) => s.hash(state),
            Self::Array(a) => a.hash(state),
            Self::Map(m) => m.hash(state),
            Self::Callback(c) => c.hash(state),
            Self::GlobalVar(v) => v.hash(state),
            Self::GlobalCompare(c) => c.hash(state),
        }
    }
}
