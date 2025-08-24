use rhai::plugin::*;
use rhai::EvalAltResult;
use std::process::Command;

#[export_module]
pub mod command {
    #[rhai_fn(return_raw)]
    pub fn run(cmd: &str) -> Result<(), Box<EvalAltResult>> {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .map_err(|e| format!("Failed to run command: {}", e))?;
        Ok(())
    }

    #[rhai_fn(return_raw)]
    pub fn run_and_read(cmd: &str) -> Result<String, Box<EvalAltResult>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| format!("Failed to run command: {}", e))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
