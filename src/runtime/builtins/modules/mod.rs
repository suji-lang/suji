//! Creation of built-in modules (std, json, yaml, toml).

pub mod json;
pub mod std;
pub mod toml;
pub mod yaml;

// Re-export module creation functions
pub use json::create_json_module;
pub use std::{create_env_module, create_fd_module, create_std_module};
pub use toml::create_toml_module;
pub use yaml::create_yaml_module;
