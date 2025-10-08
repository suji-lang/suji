use std::path::Path;

/// Context for diagnostic reporting, containing source code and file information
#[derive(Debug, Clone)]
pub struct DiagnosticContext {
    /// The source code content
    pub source: String,
    /// Optional filename for the source
    pub filename: Option<String>,
    /// Optional file ID for ariadne (defaults to filename or "input")
    pub file_id: String,
}

impl DiagnosticContext {
    /// Create a new diagnostic context from source code
    pub fn new(source: String) -> Self {
        Self {
            file_id: "input".to_string(),
            filename: None,
            source,
        }
    }

    /// Create a new diagnostic context from a file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let source = std::fs::read_to_string(path)?;
        let filename = path.to_string_lossy().to_string();

        Ok(Self {
            file_id: filename.clone(),
            filename: Some(filename),
            source,
        })
    }

    /// Create a diagnostic context with a custom file ID
    pub fn with_file_id(source: String, file_id: String) -> Self {
        Self {
            file_id,
            filename: None,
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_context_creation() {
        let source = "hello world";
        let context = DiagnosticContext::new(source.to_string());
        assert_eq!(context.source, "hello world");
        assert_eq!(context.file_id, "input");
        assert!(context.filename.is_none());
    }
}
