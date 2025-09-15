//! JSON conversion and utilities
//!
//! This module handles conversion between NN values and JSON, including
//! error handling and type validation.

pub mod converter;
pub mod types;

// Re-export main conversion functions
pub use converter::{json_to_nn_value, nn_to_json_value};
pub use types::*;
