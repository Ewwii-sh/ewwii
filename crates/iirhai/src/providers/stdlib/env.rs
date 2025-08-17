use rhai::plugin::*;
use rhai::EvalAltResult;

#[export_module]
pub mod env {
    #[rhai_fn(return_raw)]
    pub fn get_env(var: &str) -> Result<String, Box<EvalAltResult>> {
        std::env::var(var).map_err(|e| format!("Failed to get env: {e}").into())
    }

    pub fn set_env(var: &str, value: &str) {
        std::env::set_var(var, value);
    }

    #[rhai_fn(return_raw)]
    pub fn get_home_dir() -> Result<String, Box<EvalAltResult>> {
        std::env::var("HOME").map_err(|e| format!("Failed to get home directory: {e}").into())
    }

    #[rhai_fn(return_raw)]
    pub fn get_current_dir() -> Result<String, Box<EvalAltResult>> {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get CURRENT DIRECTORY: {e}").into())
            .and_then(|p| p.into_os_string().into_string().map_err(|_| "Invalid path encoding".into()))
    }

    #[rhai_fn(return_raw)]
    pub fn get_username() -> Result<String, Box<EvalAltResult>> {
        std::env::var("USER").map_err(|e| format!("Failed to get USER: {e}").into())
    }
}
