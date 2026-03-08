use rhai::Dynamic;

#[derive(Clone)]
pub struct GlobalVar {
    pub name: String,
    pub initial: Dynamic,
}

impl GlobalVar {
    pub fn from(name: String, initial: Dynamic) -> Self {
        Self {
            name,
            initial
        }
    }
}