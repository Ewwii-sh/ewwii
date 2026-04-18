use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static CALLBACK_REGISTRY: RefCell<HashMap<u64, rhai::FnPtr>> = RefCell::new(HashMap::new());
}

pub fn register_callback(handle: u64, ptr: rhai::FnPtr) {
    CALLBACK_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(handle, ptr);
        log::trace!(
            "Registered callback handle {} on thread {:?}",
            handle,
            std::thread::current().id()
        );
    });
}

pub fn get_callback(handle: u64) -> Option<rhai::FnPtr> {
    CALLBACK_REGISTRY.with(|registry| registry.borrow().get(&handle).cloned())
}
