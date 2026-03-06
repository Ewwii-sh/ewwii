use rhai::Dynamic;

#[derive(Clone)]
pub struct GlobalVar {
    name: String,
    value: Dynamic,
}

impl GlobalVar {
    pub fn from(name: String, value: Dynamic) -> Self {
        Self {
            name,
            value
        }
    }
}