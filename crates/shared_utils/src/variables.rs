use rhai::{Dynamic, Array, FnPtr};

#[derive(Clone)]
pub struct GlobalVar {
    pub name: String,
    pub initial: Dynamic,
}

#[derive(Clone)]
pub struct GlobalCompare {
    pub name: String,
    pub vars: Array,
    pub closure: FnPtr,
}

impl GlobalVar {
    pub fn from(name: String, initial: Dynamic) -> Self {
        Self { name, initial }
    }
}
