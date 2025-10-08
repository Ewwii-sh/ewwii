use rhai::Engine;

/// The shared trait defining the Ewwii plugin API
pub trait EwwiiAPI: Send + Sync {
    // == General stuff == //
    /// Print a message from the host
    fn log(&self, msg: &str);

    // == Rhai Manipulation Stuff == //
    /// Perform an action on the current real-time rhai engine
    fn rhai_engine_action(&self, f: Box<dyn FnOnce(&mut Engine) + Send>) -> Result<(), String>;
}

/// The API format that the plugin should follow
pub trait Plugin: Send + Sync {
    /// Function ran by host to startup plugin (and its a must-have for plugin loading)
    fn init(&self, host: &dyn EwwiiAPI);
}
