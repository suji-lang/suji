//! YAML-specific error types.

use suji_values::value::RuntimeError;

/// YAML-specific error types
#[derive(Debug, thiserror::Error)]
pub enum YamlError {
    #[error("YAML parse error: {message}")]
    ParseError {
        message: String,
        yaml_input: Option<String>,
    },

    #[error("YAML generation error: {message}")]
    GenerateError { message: String, value_type: String },
}

impl From<YamlError> for RuntimeError {
    fn from(err: YamlError) -> Self {
        match err {
            YamlError::ParseError {
                message,
                yaml_input,
            } => RuntimeError::YamlParseError {
                message,
                yaml_input,
            },
            YamlError::GenerateError {
                message,
                value_type,
            } => RuntimeError::YamlGenerateError {
                message,
                value_type,
            },
        }
    }
}
