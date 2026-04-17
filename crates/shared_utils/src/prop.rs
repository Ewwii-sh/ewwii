use crate::variables::{GlobalVar, GlobalCompare};
use serde::{Serialize, Deserialize};
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
    GlobalVar(GlobalVar),
    GlobalCompare(GlobalCompare),
}

/// Alternative to [`rhai::FnPtr`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Callback {
    pub name: String,
    pub curried_args: Vec<Property>,
    pub handle: Option<u64>, 
}

impl Property {
    pub fn from_dynamic(d: rhai::Dynamic) -> Self {
        // Handle Basic Types
        if d.is_unit() { return Self::None; }
        if d.is_bool() { return Self::Bool(d.as_bool().unwrap()); }
        if d.is_int() { return Self::Int(d.as_int().unwrap()); }
        if d.is_float() { return Self::Float(d.as_float().unwrap()); }
        
        // Handle Specialized Strings
        if d.is_char() { return Self::String(d.as_char().unwrap().to_string()); }
        if d.is_string() { return Self::String(d.into_string().unwrap_or_default()); }

        // Handle Recursive Arrays
        if let Ok(arr) = d.clone().into_array() {
            return Self::Array(arr.into_iter().map(Self::from_dynamic).collect());
        }

        // Handle Property Maps
        if let Ok(map) = d.clone().into_map() {
            return Self::Map(PropertyMap::from_rhai(map));
        }

        // Handle Function Pointers
        if let Ok(fn_ptr) = d.clone().cast::<rhai::FnPtr>() {
            return Self::Callback(Callback {
                name: fn_ptr.name().to_string(),
                curried_args: fn_ptr.curry().iter().cloned()
                    .map(Self::from_dynamic)
                    .collect(),
                handle: None, 
            });
        }

        // Handle Variants
        if let Some(var) = d.clone().try_cast::<GlobalVar>() {
            return Self::GlobalVar(var);
        }
        if let Some(comp) = d.clone().try_cast::<GlobalCompare>() {
            return Self::GlobalCompare(comp);
        }

        Self::None
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