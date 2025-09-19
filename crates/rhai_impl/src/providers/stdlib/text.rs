use rhai::plugin::*;

/// A module providing utility functions for string manipulation.
#[export_module]
pub mod text {

    /// Converts a string to a slug (lowercase words joined by hyphens).
    ///
    /// # Arguments
    ///
    /// * `text` - A string to be converted to a slug.
    ///
    /// # Returns
    ///
    /// Returns the `text` as a slug.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::text" as text;
    ///
    /// let result = text::to_slug("Hello World!");
    /// print(result); // output: "hello-world"
    /// ```
    pub fn to_slug(text: &str) -> String {
        let lower = text.to_lowercase();

        let sanitized: String = lower
            .chars()
            .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
            .collect();

        let words = sanitized.split_whitespace();
        let slug = words.collect::<Vec<_>>().join("-");

        slug
    }

    /// Converts a string to camel case.
    ///
    /// # Arguments
    ///
    /// * `text` - A string to be converted to camel case.
    ///
    /// # Returns
    ///
    /// Returns the `text` in camel case format.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::text" as text;
    ///
    /// let result = text::to_camel_case("hello world example");
    /// print(result); // output: "helloWorldExample"
    /// ```
    pub fn to_camel_case(text: &str) -> String {
        let cleaned: String = text
            .chars()
            .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
            .collect();

        let words = cleaned.split_whitespace();

        let camel = words
            .enumerate()
            .map(|(i, word)| {
                if i == 0 {
                    word.to_lowercase()
                } else {
                    let mut chars = word.chars();
                    match chars.next() {
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                }
            })
            .collect::<String>();

        camel
    }

    /// Truncates a string to the specified number of characters.
    ///
    /// # Arguments
    ///
    /// * `text` - A string to be truncated.
    /// * `max_chars` - The maximum number of characters to keep in the string.
    ///
    /// # Returns
    ///
    /// Returns a truncated string.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::text" as text;
    ///
    /// let result = text::truncate_chars("Hello World!", 5);
    /// print(result); // output: "Hello"
    /// ```
    pub fn truncate_chars(text: String, max_chars: i64) -> String {
        match text.char_indices().nth(max_chars.try_into().unwrap()) {
            None => text,
            Some((idx, _)) => text[..idx].to_string(),
        }
    }

    /// Converts a string to uppercase.
    ///
    /// # Arguments
    ///
    /// * `s` - A string to be converted to uppercase.
    ///
    /// # Returns
    ///
    /// Returns the string in uppercase.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::text" as text;
    ///
    /// let result = text::to_upper("hello");
    /// print(result); // output: "HELLO"
    /// ```
    pub fn to_upper(s: &str) -> String {
        s.to_uppercase()
    }

    /// Converts a string to lowercase.
    ///
    /// # Arguments
    ///
    /// * `s` - A string to be converted to lowercase.
    ///
    /// # Returns
    ///
    /// Returns the string in lowercase.
    ///
    /// # Example
    ///
    /// ```javascript
    /// import "std::text" as text;
    ///
    /// let result = text::to_lower("HELLO");
    /// print(result); // output: "hello"
    /// ```
    pub fn to_lower(s: &str) -> String {
        s.to_lowercase()
    }
}
