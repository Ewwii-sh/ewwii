use rhai::plugin::*;
use std::env as rust_env;
use std::env::VarError;

#[export_module]
pub mod env {
    pub fn get_env(var: &str) -> Result<String, String> {
        rust_env::var(var).map_err(|e| e.to_string())
    }

    pub fn set_env(var: &str, value: &str) {
        rust_env::set_var(var, value);
    }

    pub fn get_home_dir() -> Option<String> {
        rust_env::home_dir()
            .and_then(|p| p.into_os_string().into_string().ok())
    }

    pub fn get_current_dir() -> Result<String, String> {
        std::env::current_dir()
            .map_err(|e| e.to_string())
            .and_then(|p| p.into_os_string().into_string().map_err(|_| "Invalid path encoding".to_string()))
    }

    pub fn get_username() -> Result<String, String> {
        rust_env::var("USER").map_err(|e| e.to_string())
    }
}