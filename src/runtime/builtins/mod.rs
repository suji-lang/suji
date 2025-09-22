//! Builtins: functions and modules used by the runtime.

pub mod functions;
pub mod json;
pub mod modules;
pub mod nn_loader;
pub mod toml;
pub mod yaml;

// Re-export the main public API to maintain backward compatibility
pub use functions::{
    Builtin, BuiltinFunction, builtin_json_generate, builtin_json_parse, builtin_print,
    builtin_println, builtin_toml_generate, builtin_toml_parse, builtin_yaml_generate,
    builtin_yaml_parse, call_builtin, get_builtins,
};
pub use modules::{create_json_module, create_std_module, create_toml_module, create_yaml_module};

use crate::runtime::env::Env;

/// Setup the global environment with built-in functions and std module
pub fn setup_global_env(env: &Env) {
    // Add std module to global environment
    env.define_or_set("std", create_std_module());

    // Optionally add other global builtins here
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_builtin() {
        let args = vec![crate::runtime::value::Value::String("test".to_string())];
        // print returns number of bytes written
        assert_eq!(
            call_builtin("print", &args).unwrap(),
            crate::runtime::value::Value::Number(4.0)
        );
        // println returns number of bytes including newline
        assert_eq!(
            call_builtin("println", &args).unwrap(),
            crate::runtime::value::Value::Number(5.0)
        );

        // Test println with no arguments (should print just newline)
        assert_eq!(
            call_builtin("println", &[]).unwrap(),
            crate::runtime::value::Value::Number(1.0)
        );

        assert!(call_builtin("invalid", &args).is_err());
    }

    #[test]
    fn test_json_call_builtin() {
        let args = vec![crate::runtime::value::Value::String(
            "{\"test\": true}".to_string(),
        )];
        let result = call_builtin("json_parse", &args).unwrap();
        if let crate::runtime::value::Value::Map(map) = result {
            assert_eq!(
                map.get(&crate::runtime::value::MapKey::String("test".to_string())),
                Some(&crate::runtime::value::Value::Boolean(true))
            );
        } else {
            panic!("Expected map");
        }

        let args = vec![crate::runtime::value::Value::Boolean(true)];
        let result = call_builtin("json_generate", &args).unwrap();
        assert_eq!(
            result,
            crate::runtime::value::Value::String("true".to_string())
        );
    }
}
