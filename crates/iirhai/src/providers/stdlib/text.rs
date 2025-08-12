use rhai::plugin::*;

#[export_module]
pub mod text {
    pub fn to_slug(text: &str) -> String {
        let lower = text.to_lowercase();

        let sanitized: String = lower.chars().map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' }).collect();

        let words = sanitized.split_whitespace();
        let slug = words.collect::<Vec<_>>().join("-");

        slug
    }

    pub fn to_camel_case(text: &str) -> String {
        let cleaned: String = text.chars().map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' }).collect();

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

    pub fn truncate_chars(text: String, max_chars: i64) -> String {
        match text.char_indices().nth(max_chars.try_into().unwrap()) {
            None => text,
            Some((idx, _)) => text[..idx].to_string(),
        }
    }

    pub fn to_upper(s: &str) -> String {
        s.to_uppercase()
    }

    pub fn to_lower(s: &str) -> String {
        s.to_lowercase()
    }
}
