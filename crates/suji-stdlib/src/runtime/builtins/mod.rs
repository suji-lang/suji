//! Builtins: functions and modules used by the runtime.

pub mod functions;
pub mod json;
pub mod modules;
pub mod suji_loader;
pub mod toml;
pub mod yaml;

// Re-export the main public API to maintain backward compatibility
pub use functions::{
    Builtin, BuiltinFunction, builtin_io_open, builtin_json_generate, builtin_json_parse,
    builtin_print, builtin_println, builtin_random_random, builtin_random_seed,
    builtin_toml_generate, builtin_toml_parse, builtin_yaml_generate, builtin_yaml_parse,
    call_builtin,
};
pub use modules::{create_json_module, create_std_module, create_toml_module, create_yaml_module};

use suji_runtime::env::Env;

/// Setup the global environment with built-in functions and std module
pub fn setup_global_env(env: &Env) {
    // Register all builtin functions with the runtime registry
    register_all_builtins();

    // Add std module to global environment
    env.define_or_set("std", create_std_module());

    // Optionally add other global builtins here
}

/// Register all builtin function implementations with the runtime
fn register_all_builtins() {
    use suji_runtime::builtins::register_builtin;

    // Register core functions
    register_builtin("print", builtin_print as suji_runtime::builtins::BuiltinFn);
    register_builtin(
        "println",
        builtin_println as suji_runtime::builtins::BuiltinFn,
    );

    // Register JSON functions
    register_builtin(
        "json_parse",
        builtin_json_parse as suji_runtime::builtins::BuiltinFn,
    );
    register_builtin(
        "json_generate",
        builtin_json_generate as suji_runtime::builtins::BuiltinFn,
    );

    // Register YAML functions
    register_builtin(
        "yaml_parse",
        builtin_yaml_parse as suji_runtime::builtins::BuiltinFn,
    );
    register_builtin(
        "yaml_generate",
        builtin_yaml_generate as suji_runtime::builtins::BuiltinFn,
    );

    // Register TOML functions
    register_builtin(
        "toml_parse",
        builtin_toml_parse as suji_runtime::builtins::BuiltinFn,
    );
    register_builtin(
        "toml_generate",
        builtin_toml_generate as suji_runtime::builtins::BuiltinFn,
    );

    // Register IO functions
    register_builtin(
        "io_open",
        builtin_io_open as suji_runtime::builtins::BuiltinFn,
    );

    // Register random functions
    register_builtin(
        "random_random",
        builtin_random_random as suji_runtime::builtins::BuiltinFn,
    );
    register_builtin(
        "random_seed",
        builtin_random_seed as suji_runtime::builtins::BuiltinFn,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_runtime::value::DecimalNumber;

    #[test]
    fn test_call_builtin() {
        // Initialize the builtin registry
        register_all_builtins();

        let args = vec![suji_runtime::value::Value::String("test".to_string())];
        // print returns number of bytes written
        assert_eq!(
            call_builtin("print", &args).unwrap(),
            suji_runtime::value::Value::Number(DecimalNumber::from_i64(4))
        );
        // println returns number of bytes including newline
        assert_eq!(
            call_builtin("println", &args).unwrap(),
            suji_runtime::value::Value::Number(DecimalNumber::from_i64(5))
        );

        // Test println with no arguments (should print just newline)
        assert_eq!(
            call_builtin("println", &[]).unwrap(),
            suji_runtime::value::Value::Number(DecimalNumber::from_i64(1))
        );

        assert!(call_builtin("invalid", &args).is_err());
    }

    #[test]
    fn test_json_call_builtin() {
        // Initialize the builtin registry
        register_all_builtins();

        let args = vec![suji_runtime::value::Value::String(
            "{\"test\": true}".to_string(),
        )];
        let result = call_builtin("json_parse", &args).unwrap();
        if let suji_runtime::value::Value::Map(map) = result {
            assert_eq!(
                map.get(&suji_runtime::value::MapKey::String("test".to_string())),
                Some(&suji_runtime::value::Value::Boolean(true))
            );
        } else {
            panic!("Expected map");
        }

        let args = vec![suji_runtime::value::Value::Boolean(true)];
        let result = call_builtin("json_generate", &args).unwrap();
        assert_eq!(
            result,
            suji_runtime::value::Value::String("true".to_string())
        );
    }
}
