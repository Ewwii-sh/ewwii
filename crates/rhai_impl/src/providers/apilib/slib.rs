//! Slib, A rhai library for interacting with loaded shared libraries.

use rhai::{plugin::*, Array, Dynamic};

#[export_module]
pub mod slib {
    /// Call a function registered by the currently loaded shared library
    ///
    /// # Arguments
    ///
    /// * `fn_name`: The name of the function to call
    /// * `args`: The arguments to pass to the function (in an array)
    ///
    /// # Returns
    ///
    /// The result from the shared library
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "api::slib" as slib;
    ///
    /// let eg_output = slib::call_fn("my_func", ["foo", 80, true]);
    /// ```
    pub fn call_fn(fn_name: String, args: Array) -> Dynamic {
        match shared_utils::slib_store::call_registered(&fn_name, args) {
            Ok(Some(d)) => d,
            Ok(None) => Dynamic::default(),
            Err(e) => {
                log::error!("Error calling function: {}", e);

                Dynamic::default()
            }
        }
    }
}
