use super::value::{RuntimeError, Value};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Mutex;

/// Global cache for compiled regex patterns
static REGEX_CACHE: Lazy<Mutex<HashMap<String, Result<Regex, String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Compile a regex pattern, using cache if available
pub fn compile_regex(pattern: &str) -> Result<Regex, RuntimeError> {
    // Check cache first
    {
        let cache = REGEX_CACHE.lock().unwrap();
        if let Some(cached_result) = cache.get(pattern) {
            return match cached_result {
                Ok(regex) => Ok(regex.clone()),
                Err(error_msg) => Err(RuntimeError::RegexError {
                    message: error_msg.clone(),
                }),
            };
        }
    }

    // Compile the regex
    let compile_result = match Regex::new(pattern) {
        Ok(regex) => Ok(regex),
        Err(err) => Err(RuntimeError::RegexError {
            message: format!("Invalid regex pattern '{}': {}", pattern, err),
        }),
    };

    // Cache the result (both success and failure)
    {
        let mut cache = REGEX_CACHE.lock().unwrap();
        match &compile_result {
            Ok(regex) => {
                cache.insert(pattern.to_string(), Ok(regex.clone()));
            }
            Err(RuntimeError::RegexError { message }) => {
                cache.insert(pattern.to_string(), Err(message.clone()));
            }
            _ => unreachable!(),
        }
    }

    compile_result
}

/// Create a Value::Regex from a pattern string
pub fn create_regex_value(pattern: &str) -> Result<Value, RuntimeError> {
    let regex = compile_regex(pattern)?;
    Ok(Value::Regex(regex))
}

/// Test if a string matches a regex pattern
pub fn regex_match(text: &str, pattern: &str) -> Result<bool, RuntimeError> {
    let regex = compile_regex(pattern)?;
    Ok(regex.is_match(text))
}

/// Test if a string matches a compiled regex
pub fn regex_match_compiled(text: &str, regex: &Regex) -> bool {
    regex.is_match(text)
}

/// Clear the regex cache (useful for testing)
#[cfg(test)]
pub fn clear_regex_cache() {
    let mut cache = REGEX_CACHE.lock().unwrap();
    cache.clear();
}

/// Get cache statistics (for debugging)
#[cfg(debug_assertions)]
pub fn regex_cache_stats() -> (usize, usize) {
    let cache = REGEX_CACHE.lock().unwrap();
    let total = cache.len();
    let successful = cache.values().filter(|r| r.is_ok()).count();
    (total, successful)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_regex_compilation() {
        clear_regex_cache();

        let result = compile_regex(r"^hello.*world$");
        assert!(result.is_ok());

        let regex = result.unwrap();
        assert!(regex.is_match("hello beautiful world"));
        assert!(!regex.is_match("goodbye world"));
    }

    #[test]
    fn test_invalid_regex_pattern() {
        clear_regex_cache();

        let result = compile_regex(r"[invalid");
        assert!(matches!(result, Err(RuntimeError::RegexError { .. })));
    }

    #[test]
    fn test_regex_caching() {
        clear_regex_cache();

        // First compilation
        let result1 = compile_regex(r"test\d+");
        assert!(result1.is_ok());

        // Second compilation should use cache
        let result2 = compile_regex(r"test\d+");
        assert!(result2.is_ok());

        // Both should work the same
        let regex1 = result1.unwrap();
        let regex2 = result2.unwrap();

        assert!(regex1.is_match("test123"));
        assert!(regex2.is_match("test123"));
        assert!(!regex1.is_match("test"));
        assert!(!regex2.is_match("test"));
    }

    #[test]
    fn test_error_caching() {
        clear_regex_cache();

        // First attempt with invalid pattern
        let result1 = compile_regex(r"[invalid");
        assert!(matches!(result1, Err(RuntimeError::RegexError { .. })));

        // Second attempt should return cached error
        let result2 = compile_regex(r"[invalid");
        assert!(matches!(result2, Err(RuntimeError::RegexError { .. })));
    }

    #[test]
    fn test_create_regex_value() {
        clear_regex_cache();

        let result = create_regex_value(r"^\d{3}-\d{2}-\d{4}$");
        assert!(result.is_ok());

        if let Value::Regex(regex) = result.unwrap() {
            assert!(regex.is_match("123-45-6789"));
            assert!(!regex.is_match("not-a-ssn"));
        } else {
            panic!("Expected regex value");
        }
    }

    #[test]
    fn test_regex_match() {
        clear_regex_cache();

        // Test email pattern
        let email_pattern = r"^[^@\s]+@[^@\s]+\.[^@\s]+$";

        assert!(regex_match("user@example.com", email_pattern).unwrap());
        assert!(!regex_match("not-an-email", email_pattern).unwrap());
        assert!(!regex_match("user@", email_pattern).unwrap());
    }

    #[test]
    fn test_regex_match_compiled() {
        clear_regex_cache();

        let regex = compile_regex(r"^\w+$").unwrap();

        assert!(regex_match_compiled("hello", &regex));
        assert!(regex_match_compiled("test123", &regex));
        assert!(!regex_match_compiled("hello world", &regex));
        assert!(!regex_match_compiled("hello!", &regex));
    }

    #[test]
    fn test_regex_from_spec_examples() {
        clear_regex_cache();

        // Example from spec: email validation
        let email_pattern = r"^[^@\s]+@[^@\s]+\.[^@\s]+$";
        assert!(regex_match("ada@example.com", email_pattern).unwrap());
        assert!(!regex_match("not_an_email", email_pattern).unwrap());

        // Example from spec: user ID pattern
        let user_pattern = r"^user_[0-9]+$";
        assert!(regex_match("user_123", user_pattern).unwrap());
        assert!(regex_match("user_0", user_pattern).unwrap());
        assert!(!regex_match("user_", user_pattern).unwrap());
        assert!(!regex_match("admin_123", user_pattern).unwrap());
    }

    #[test]
    fn test_complex_patterns() {
        clear_regex_cache();

        // Test Unicode support
        let unicode_pattern = r"^\p{L}+$";
        assert!(regex_match("hello", unicode_pattern).unwrap());
        assert!(regex_match("café", unicode_pattern).unwrap());

        // Test case insensitive (using flags)
        let case_pattern = r"(?i)^HELLO$";
        assert!(regex_match("hello", case_pattern).unwrap());
        assert!(regex_match("HELLO", case_pattern).unwrap());
        assert!(regex_match("HeLLo", case_pattern).unwrap());
    }

    #[test]
    fn test_anchors_and_escapes() {
        clear_regex_cache();

        // Test anchors
        let anchored = r"^test$";
        assert!(regex_match("test", anchored).unwrap());
        assert!(!regex_match("testing", anchored).unwrap());
        assert!(!regex_match("pretest", anchored).unwrap());

        // Test escaping special characters
        let literal_dot = r"test\.txt$";
        assert!(regex_match("test.txt", literal_dot).unwrap());
        assert!(!regex_match("testXtxt", literal_dot).unwrap());
    }
}
