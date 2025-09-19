use rhai::plugin::*;
use rhai::EvalAltResult;

#[export_module]
pub mod env {
    /// Get the value of an environment variable.
    ///
    /// # Arguments
    ///
    /// * `var` - The name of the environment variable to retrieve.
    ///
    /// # Returns
    ///
    /// This function returns the value of the environment variable as a `String`.
    /// If the variable is not found or there is an error, it returns a `Result::Err` with the error message.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::env" as env;
    ///
    /// // Get the value of the "HOME" environment variable
    /// let home_dir = env::get_env("HOME");
    /// print(home_dir); // output: /home/username
    /// ```
    #[rhai_fn(return_raw)]
    pub fn get_env(var: &str) -> Result<String, Box<EvalAltResult>> {
        std::env::var(var).map_err(|e| format!("Failed to get env: {e}").into())
    }

    /// Set the value of an environment variable.
    ///
    /// # Arguments
    ///
    /// * `var` - The name of the environment variable to set.
    /// * `value` - The value to assign to the environment variable.
    ///
    /// # Returns
    ///
    /// This function does not return a value.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::env" as env;
    ///
    /// // Set the value of the "MY_VAR" environment variable
    /// env::set_env("MY_VAR", "SomeValue");
    /// ```
    pub fn set_env(var: &str, value: &str) {
        std::env::set_var(var, value);
    }

    /// Get the path to the home directory.
    ///
    /// # Returns
    ///
    /// This function returns the value of the "HOME" environment variable as a `String`.
    /// If the variable is not found or there is an error, it returns a `Result::Err` with the error message.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::env" as env;
    ///
    /// // Get the home directory
    /// let home_dir = env::get_home_dir();
    /// print(home_dir); // output: /home/username
    /// ```
    #[rhai_fn(return_raw)]
    pub fn get_home_dir() -> Result<String, Box<EvalAltResult>> {
        std::env::var("HOME").map_err(|e| format!("Failed to get home directory: {e}").into())
    }

    /// Get the current working directory.
    ///
    /// # Returns
    ///
    /// This function returns the current working directory as a `String`. If there is an error
    /// (e.g., if the path cannot be retrieved), it returns a `Result::Err` with the error message.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::env" as env;
    ///
    /// // Get the current working directory
    /// let current_dir = env::get_current_dir();
    /// print(current_dir); // output: /home/username/project
    /// ```
    #[rhai_fn(return_raw)]
    pub fn get_current_dir() -> Result<String, Box<EvalAltResult>> {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {e}").into())
            .and_then(|p| {
                p.into_os_string().into_string().map_err(|_| "Invalid path encoding".into())
            })
    }

    /// Get the current username.
    ///
    /// # Returns
    ///
    /// This function returns the value of the "USER" environment variable as a `String`.
    /// If the variable is not found or there is an error, it returns a `Result::Err` with the error message.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::env" as env;
    ///
    /// // Get the username of the current user
    /// let username = env::get_username();
    /// print(username); // output: username
    /// ```
    #[rhai_fn(return_raw)]
    pub fn get_username() -> Result<String, Box<EvalAltResult>> {
        std::env::var("USER").map_err(|e| format!("Failed to get USER: {e}").into())
    }
}
