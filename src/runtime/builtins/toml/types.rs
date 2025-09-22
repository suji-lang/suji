//! TOML-specific error types.

use crate::runtime::value::RuntimeError;

/// TOML-specific error types
#[derive(Debug, thiserror::Error)]
pub enum TomlError {
    #[error("TOML parse error: {message}")]
    ParseError {
        message: String,
        toml_input: Option<String>,
    },

    #[error("TOML generation error: {message}")]
    GenerateError { message: String, value_type: String },

    #[error("TOML conversion error: {message}")]
    ConversionError { message: String },
}

impl From<TomlError> for RuntimeError {
    fn from(err: TomlError) -> Self {
        match err {
            TomlError::ParseError {
                message,
                toml_input,
            } => RuntimeError::TomlParseError {
                message,
                toml_input,
            },
            TomlError::GenerateError {
                message,
                value_type,
            } => RuntimeError::TomlGenerateError {
                message,
                value_type,
            },
            TomlError::ConversionError { message } => RuntimeError::TomlConversionError { message },
        }
    }
}
