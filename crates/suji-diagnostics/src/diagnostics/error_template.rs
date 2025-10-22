/// Error template for consistent error message formatting
#[derive(Debug, Clone)]
pub struct ErrorTemplate {
    /// Error code for identification
    pub code: u32,
    /// Short title for the error
    pub title: &'static str,
    /// Main error message
    pub message: String,
    /// Helpful suggestions for fixing the error
    pub suggestions: Vec<String>,
}

impl ErrorTemplate {
    /// Create a new error template
    pub fn new(code: u32, title: &'static str, message: &str) -> Self {
        Self {
            code,
            title,
            message: message.to_string(),
            suggestions: Vec::new(),
        }
    }

    /// Add a suggestion to the template
    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestions.push(suggestion.to_string());
        self
    }
}
