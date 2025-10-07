use rhai::Engine;

/// The shared trait defining the Ewwii plugin API
pub trait EwwiiAPI: Send + Sync {
    // General stuff
    fn log(&self, msg: &str);

    // Rhai Manipulation Stuff
    fn get_rhai_engine(&self) -> &mut Engine;
    fn set_rhai_engine(&self, engine: &Engine);
}

/// The API format that the plugin should follow
pub trait Plugin: Send + Sync {
    fn init(&self, host: &dyn EwwiiAPI);
}
