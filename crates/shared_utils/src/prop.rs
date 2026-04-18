use crate::variables::{GlobalCompare, GlobalVar};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

/// A deterministic, serializable collection of widget properties.
/// Replaces rhai::Map in your WidgetNode.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Hash)]
pub struct PropertyMap(pub BTreeMap<String, Property>);

impl PropertyMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Converts a Rhai Map into stable PropertyMap.
    pub fn from_rhai(rhai_map: rhai::Map) -> Self {
        let mut map = BTreeMap::new();
        for (k, v) in rhai_map {
            map.insert(k.to_string(), Property::from_dynamic(v));
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
    pub fn from_dynamic(d: rhai::Dynamic) -> Self {
        // Handle Basic Types
        if d.is_unit() {
            return Self::None;
        }
        if d.is_bool() {
            return Self::Bool(d.as_bool().unwrap());
        }
        if d.is_int() {
            return Self::Int(d.as_int().unwrap());
        }
        if d.is_float() {
            return Self::Float(d.as_float().unwrap());
        }

        // Handle Specialized Strings
        if d.is_char() {
            return Self::String(d.as_char().unwrap().to_string());
        }
        if d.is_string() {
            return Self::String(d.into_string().unwrap_or_default());
        }

        // Handle Recursive Arrays
        if let Ok(arr) = d.clone().into_array() {
            return Self::Array(arr.into_iter().map(Self::from_dynamic).collect());
        }

        // Handle Property Maps
        if d.is_map() {
            let map = d.clone().cast::<rhai::Map>();
            return Self::Map(PropertyMap::from_rhai(map));
        }

        // Handle Function Pointers
        if let Some(fn_ptr) = d.clone().try_cast::<rhai::FnPtr>() {
            return Self::Callback(Callback { name: fn_ptr.fn_name().to_string(), handle: None });
        }

        // Handle Variants
        if let Some(var) = d.clone().try_cast::<GlobalVar>() {
            return Self::GlobalVar(Box::new(var));
        }

        if let Some(comp) = d.clone().try_cast::<GlobalCompare>() {
            return Self::GlobalCompare(Box::new(comp));
        }

        Self::None
    }

    pub fn into_dynamic(self) -> rhai::Dynamic {
        match self {
            Self::String(s) => s.into(),
            Self::Int(i) => i.into(),
            Self::Float(f) => f.into(),
            Self::Bool(b) => b.into(),
            Self::Array(arr) => {
                let vec: Vec<rhai::Dynamic> = arr.into_iter().map(|p| p.into_dynamic()).collect();
                vec.into()
            }
            Self::Map(map) => {
                let mut rhai_map = rhai::Map::new();
                for (k, v) in map.0 {
                    rhai_map.insert(k.into(), v.into_dynamic());
                }
                rhai_map.into()
            }
            _ => rhai::Dynamic::UNIT,
        }
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
