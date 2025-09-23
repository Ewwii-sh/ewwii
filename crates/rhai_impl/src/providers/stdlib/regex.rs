use regex::Regex;
use rhai::plugin::*;
use rhai::{Array, Dynamic};

/// A module providing regular expression support.
#[export_module]
pub mod regex_lib {
    /// Checks if a regex pattern matches the given text.
    ///
    /// # Arguments
    ///
    /// * `text` - A string to be matched with the pattern.
    /// * `pattern` - The pattern to match on the string.
    ///
    /// # Returns
    ///
    /// Returns a boolean (true/false) based on if the pattern is matched on the text provided.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::regex" as regex;
    ///
    /// let result = regex::is_match("This is an example!", "example");
    /// if result == true {
    ///     print("The pattern is matched!");
    /// }
    /// ```
    #[rhai_fn(return_raw)]
    pub fn is_match(text: &str, pattern: &str) -> Result<bool, Box<EvalAltResult>> {
        let re = Regex::new(pattern).map_err(|e| format!("Failed to read regex pattern: {}", e))?;
        Ok(re.is_match(text))
    }

    /// Returns the first match of a regex pattern in the text.
    ///
    /// # Arguments
    ///
    /// * `text` - A string to be matched with the pattern.
    /// * `pattern` - The pattern to match on the string.
    ///
    /// # Returns
    ///
    /// Returns a string which is the first match of a regex pattern.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::regex" as regex;
    ///
    /// let result = regex::find("This is an example!", `\be\w*\b`);
    /// print(result); // output: "example"
    /// ```
    #[rhai_fn(return_raw)]
    pub fn find(text: &str, pattern: &str) -> Result<String, Box<EvalAltResult>> {
        let re = Regex::new(pattern).map_err(|e| format!("Failed to read regex pattern: {}", e))?;
        match re.find(text).map(|m| m.as_str().to_string()) {
            Some(s) => Ok(s),
            None => Ok(String::new()),
        }
    }

    /// Returns all matches of a regex pattern in the text.
    ///
    /// # Arguments
    ///
    /// * `text` - A string to be matched with the pattern.
    /// * `pattern` - The pattern to match on the string.
    ///
    /// # Returns
    ///
    /// Returns aan array of strings containing the all the things that match the regex pattern in the provided text.
    ///
    /// # Example
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::regex" as regex;
    ///
    /// let result = regex::find("This is an example!", `\be\w*\b`);
    /// print(result); // output: ["example"]
    /// ```
    #[rhai_fn(return_raw)]
    pub fn find_all(text: &str, pattern: &str) -> Result<Array, Box<EvalAltResult>> {
        let re = Regex::new(pattern).map_err(|e| format!("Failed to read regex pattern: {}", e))?;
        let results: Array = re
            .find_iter(text)
            .map(|m| Dynamic::from(m.as_str().to_string()))
            .collect();

        Ok(results)
    }

    /// Replaces all matches of a regex pattern with a replacement string.
    ///
    /// # Arguments
    ///
    /// * `text` - A string to be matched with the pattern.
    /// * `pattern` - The pattern to match on the string.
    /// * `replacement` - A string that the matched pattern will get replaced with.
    ///
    /// # Returns
    ///
    /// Returns the provided text with the matches of the regex pattern replaced with the provided replacement argument.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::regex" as regex;
    ///
    /// let result = regex::replace("This is an example!", "an example", "a test");
    /// print(result); // output: "This is a test!"
    /// ```
    #[rhai_fn(return_raw)]
    pub fn replace(
        text: &str,
        pattern: &str,
        replacement: &str,
    ) -> Result<String, Box<EvalAltResult>> {
        let re = Regex::new(pattern).map_err(|e| format!("Failed to read regex pattern: {}", e))?;
        Ok(re.replace_all(text, replacement).to_string())
    }
}
