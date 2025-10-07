/// The shared trait defining the Ewwii plugin API
pub trait EwwiiAPI: Send + Sync {
    // General stuff
    fn log(&self, msg: &str);
}

/// The API format that the plugin should follow
pub trait Plugin: Send + Sync {
    fn init(&self, host: &dyn EwwiiAPI);
}
