use rhai::plugin::*;
use rhai::EvalAltResult;
use std::process::Command;

#[export_module]
pub mod command {
    /// Executes a shell command without capturing the output.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The shell command to execute as a string.
    ///
    /// # Returns
    ///
    /// This function returns nothing if the command executes successfully. If there is an error
    /// running the command, it returns the error.
    ///
    /// # Example
    ///
    /// ```js
    /// import "std::command" as cmd;
    ///
    /// // Run a shell command (e.g., list directory contents)
    /// cmd::run("ls -l");
    /// ```
    #[rhai_fn(return_raw)]
    pub fn run(cmd: &str) -> Result<(), Box<EvalAltResult>> {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .map_err(|e| format!("Failed to run command: {}", e))?;
        Ok(())
    }

    /// Executes a shell command and captures its output.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The shell command to execute as a string.
    ///
    /// # Returns
    ///
    /// This function returns the standard output of the command as a `string`. If the command fails,
    /// it returns the error.
    ///
    /// # Example
    ///
    /// ```js
    /// import "std::command" as cmd;
    ///
    /// // Run a shell command and capture its output
    /// let output = cmd::run_and_read("echo 'Hello, world!'");
    /// print(output); // output: Hello, world!
    /// ```
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
