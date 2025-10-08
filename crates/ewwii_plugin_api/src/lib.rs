use rhai::Engine;

/// The shared trait defining the Ewwii plugin API
pub trait EwwiiAPI: Send + Sync {
    // General stuff
    fn log(&self, msg: &str);

    // Rhai Manipulation Stuff
    fn rhai_engine_action(&self, f: Box<dyn FnOnce(&mut Engine) + Send>) -> Result<(), String>;
}

/// The API format that the plugin should follow
pub trait Plugin: Send + Sync {
    fn init(&self, host: &dyn EwwiiAPI);
}
