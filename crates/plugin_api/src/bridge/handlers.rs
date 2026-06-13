use std::sync::Arc;

// cfg callback
pub type ConfigCallbackFn = Arc<dyn Fn(&str, &str) + Send + Sync>;

pub trait ConfigCallbackFnExt {
    fn new<F>(f: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static;
}

impl ConfigCallbackFnExt for ConfigCallbackFn {
    fn new<F>(f: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        Arc::new(f)
    }
}

// listen handle
pub type ListenHandleFn = Arc<dyn Fn() + Send + Sync>;

pub trait ListenHandleFnExt {
    fn new<F>(f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static;
}

impl ListenHandleFnExt for ListenHandleFn {
    fn new<F>(f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Arc::new(f)
    }
}

// signal update
pub type SignalUpdateFn = Arc<dyn Fn(&str) + Send + Sync>;

pub trait SignalUpdateFnExt {
    fn new<F>(f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static;
}

impl SignalUpdateFnExt for SignalUpdateFn {
    fn new<F>(f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        Arc::new(f)
    }
}
