//! TOML conversion and utilities
//!
//! This module handles conversion between NN values and TOML, including
//! error handling and type validation.

pub mod converter;
pub mod types;

// Re-export main conversion functions
pub use converter::{nn_to_toml_value, toml_to_nn_value};
pub use types::*;
