//! JSON-specific types and utilities
//!
//! This module contains types and utilities specific to JSON conversion.

use crate::runtime::value::RuntimeError;

/// JSON-specific error types
#[derive(Debug, Clone)]
pub enum JsonError {
    /// Error parsing JSON string
    ParseError {
        message: String,
        json_input: Option<String>,
    },
    /// Error generating JSON string
    GenerateError { message: String, value_type: String },
}

impl From<JsonError> for RuntimeError {
    fn from(err: JsonError) -> Self {
        match err {
            JsonError::ParseError {
                message,
                json_input,
            } => RuntimeError::JsonParseError {
                message,
                json_input,
            },
            JsonError::GenerateError {
                message,
                value_type,
            } => RuntimeError::JsonGenerateError {
                message,
                value_type,
            },
        }
    }
}
