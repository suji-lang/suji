//! Builtin function implementations
//!
//! This module contains all builtin function implementations, organized by functionality.

pub mod json_generate;
pub mod json_parse;
pub mod print;
pub mod println;
pub mod toml_generate;
pub mod toml_parse;
pub mod yaml_generate;
pub mod yaml_parse;

// Re-export all function implementations
pub use json_generate::builtin_json_generate;
pub use json_parse::builtin_json_parse;
pub use print::builtin_print;
pub use println::builtin_println;
pub use toml_generate::builtin_toml_generate;
pub use toml_parse::builtin_toml_parse;
pub use yaml_generate::builtin_yaml_generate;
pub use yaml_parse::builtin_yaml_parse;

use crate::runtime::value::{RuntimeError, Value};

/// Built-in function type
pub type BuiltinFunction = fn(&[Value]) -> Result<Value, RuntimeError>;

/// Built-in function wrapper for the runtime
#[derive(Debug, Clone)]
pub struct Builtin {
    pub name: &'static str,
    pub func: BuiltinFunction,
    pub min_args: usize,
    pub max_args: Option<usize>, // None for variadic
}

/// Get all built-in functions
pub fn get_builtins() -> Vec<Builtin> {
    vec![
        Builtin {
            name: "print",
            func: builtin_print,
            min_args: 0,
            max_args: None, // variadic
        },
        Builtin {
            name: "println",
            func: builtin_println,
            min_args: 0,
            max_args: None, // variadic
        },
        Builtin {
            name: "json_parse",
            func: builtin_json_parse,
            min_args: 1,
            max_args: Some(1),
        },
        Builtin {
            name: "json_generate",
            func: builtin_json_generate,
            min_args: 1,
            max_args: Some(1),
        },
        Builtin {
            name: "toml_parse",
            func: builtin_toml_parse,
            min_args: 1,
            max_args: Some(1),
        },
        Builtin {
            name: "toml_generate",
            func: builtin_toml_generate,
            min_args: 1,
            max_args: Some(1),
        },
        Builtin {
            name: "yaml_parse",
            func: builtin_yaml_parse,
            min_args: 1,
            max_args: Some(1),
        },
        Builtin {
            name: "yaml_generate",
            func: builtin_yaml_generate,
            min_args: 1,
            max_args: Some(1),
        },
    ]
}


/// Call a built-in function by name
pub fn call_builtin(name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
    match name {
        "print" => builtin_print(args),
        "println" => builtin_println(args),
        "json_parse" => builtin_json_parse(args),
        "json_generate" => builtin_json_generate(args),
        "toml_parse" => builtin_toml_parse(args),
        "toml_generate" => builtin_toml_generate(args),
        "yaml_parse" => builtin_yaml_parse(args),
        "yaml_generate" => builtin_yaml_generate(args),
        _ => Err(RuntimeError::InvalidOperation {
            message: format!("Unknown built-in function: {}", name),
        }),
    }
}
