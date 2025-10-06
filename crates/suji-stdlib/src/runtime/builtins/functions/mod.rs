//! Built-in function implementations.

pub mod io_open;
pub mod json_generate;
pub mod json_parse;
pub mod print;
pub mod println;
pub mod random_core;
pub mod random_random;
pub mod random_seed;
pub mod toml_generate;
pub mod toml_parse;
pub mod yaml_generate;
pub mod yaml_parse;

// Re-export all function implementations
pub use io_open::builtin_io_open;
pub use json_generate::builtin_json_generate;
pub use json_parse::builtin_json_parse;
pub use print::builtin_print;
pub use println::builtin_println;
pub use random_random::builtin_random_random;
pub use random_seed::builtin_random_seed;
pub use toml_generate::builtin_toml_generate;
pub use toml_parse::builtin_toml_parse;
pub use yaml_generate::builtin_yaml_generate;
pub use yaml_parse::builtin_yaml_parse;

use suji_runtime::value::{RuntimeError, Value};

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

/// Call a built-in function by name
pub fn call_builtin(name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
    match name {
        "print" => builtin_print(args),
        "println" => builtin_println(args),
        "random_random" => builtin_random_random(args),
        "random_seed" => builtin_random_seed(args),
        "json_parse" => builtin_json_parse(args),
        "json_generate" => builtin_json_generate(args),
        "io_open" => builtin_io_open(args),
        "toml_parse" => builtin_toml_parse(args),
        "toml_generate" => builtin_toml_generate(args),
        "yaml_parse" => builtin_yaml_parse(args),
        "yaml_generate" => builtin_yaml_generate(args),
        _ => Err(RuntimeError::InvalidOperation {
            message: format!("Unknown built-in function: {}", name),
        }),
    }
}
