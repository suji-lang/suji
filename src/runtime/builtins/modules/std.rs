//! Standard library module creation
//!
//! This module handles the creation of the std builtin module.

use super::{json::create_json_module, toml::create_toml_module, yaml::create_yaml_module};
use crate::runtime::builtins::nn_loader::{load_print, load_println};
use crate::runtime::value::{EnvProxy, MapKey, StreamHandle, Value};
use indexmap::IndexMap;
use std::rc::Rc;

/// Create a special built-in function value that can be called normally
fn create_builtin_function_value(name: &str) -> Value {
    use crate::ast::Stmt;
    use crate::runtime::env::Env;
    use crate::runtime::value::{FunctionValue, ParamSpec};
    use crate::token::Span;
    use std::rc::Rc;

    // Create a function value with a special marker that identifies it as built-in
    Value::Function(FunctionValue {
        params: vec![ParamSpec {
            name: format!("__builtin_{}__", name), // Special marker with function name
            default: None,
        }],
        body: Stmt::Block {
            statements: vec![],
            span: Span::default(),
        },
        env: Rc::new(Env::new()),
    })
}

/// Create the FD module with standard I/O streams
///
/// The FD module provides access to the process standard streams:
/// - `stdin`: readable stream for standard input
/// - `stdout`: writable stream for standard output
/// - `stderr`: writable stream for standard error
///
/// These streams are shared process-wide. Closing them is discouraged
/// as it affects subsequent I/O operations.
///
/// # Usage
/// ```nn
/// import std:FD
///
/// # Write to stdout
/// FD:stdout::write("Hello, world!\n")
///
/// # Read from stdin (may block)
/// input = FD:stdin::read()
/// ```
pub fn create_fd_module() -> Value {
    let mut fd_map = IndexMap::new();

    // Create shared stream handles - these will be the same instances
    // referenced by all imports of FD
    let stdin_stream = Rc::new(StreamHandle::new_stdin());
    let stdout_stream = Rc::new(StreamHandle::new_stdout());
    let stderr_stream = Rc::new(StreamHandle::new_stderr());

    // Insert into FD map
    fd_map.insert(
        MapKey::String("stdin".to_string()),
        Value::Stream(stdin_stream),
    );
    fd_map.insert(
        MapKey::String("stdout".to_string()),
        Value::Stream(stdout_stream),
    );
    fd_map.insert(
        MapKey::String("stderr".to_string()),
        Value::Stream(stderr_stream),
    );

    Value::Map(fd_map)
}

/// Create the ENV module for environment variable access
///
/// The ENV module provides access to process environment variables as a map-like interface:
/// - All operations delegate to std::env functions
/// - Changes affect the actual process environment
/// - Subprocesses inherit environment changes
/// - Keys and values are always strings
///
/// # Usage
/// ```nn
/// import std:ENV
///
/// # Read environment variable
/// path = ENV:PATH
///
/// # Set environment variable
/// ENV:MY_VAR = "value"
///
/// # Check if variable exists
/// has_home = ENV::contains("HOME")
/// ```
pub fn create_env_module() -> Value {
    // Create a shared EnvProxy instance
    let env_proxy = Rc::new(EnvProxy::new());
    Value::EnvMap(env_proxy)
}

/// Create the std module as a Value::Map
pub fn create_std_module() -> Value {
    let mut std_map = IndexMap::new();

    // Load NN-language print and println (pure NN definitions)
    let print_fn = load_print().unwrap_or_else(|_| create_builtin_function_value("print"));
    let println_fn = load_println().unwrap_or_else(|_| create_builtin_function_value("println"));

    // Insert print/println
    std_map.insert(MapKey::String("print".to_string()), print_fn);
    std_map.insert(MapKey::String("println".to_string()), println_fn);

    // Add json module to std module
    std_map.insert(MapKey::String("json".to_string()), create_json_module());

    // Add yaml module to std module
    std_map.insert(MapKey::String("yaml".to_string()), create_yaml_module());

    // Add toml module to std module
    std_map.insert(MapKey::String("toml".to_string()), create_toml_module());

    // Build FD and ENV modules and attach
    std_map.insert(MapKey::String("FD".to_string()), create_fd_module());
    std_map.insert(MapKey::String("ENV".to_string()), create_env_module());

    Value::Map(std_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fd_module_creation() {
        let fd_module = create_fd_module();

        if let Value::Map(map) = fd_module {
            // Check all three streams exist
            assert!(map.contains_key(&MapKey::String("stdin".to_string())));
            assert!(map.contains_key(&MapKey::String("stdout".to_string())));
            assert!(map.contains_key(&MapKey::String("stderr".to_string())));

            // Verify they are stream values
            let stdin = map.get(&MapKey::String("stdin".to_string())).unwrap();
            let stdout = map.get(&MapKey::String("stdout".to_string())).unwrap();
            let stderr = map.get(&MapKey::String("stderr".to_string())).unwrap();

            assert!(matches!(stdin, Value::Stream(_)));
            assert!(matches!(stdout, Value::Stream(_)));
            assert!(matches!(stderr, Value::Stream(_)));
        } else {
            panic!("FD module should be a map");
        }
    }

    #[test]
    fn test_std_module_creation() {
        let std_module = create_std_module();

        if let Value::Map(map) = std_module {
            assert!(map.contains_key(&MapKey::String("print".to_string())));
            assert!(map.contains_key(&MapKey::String("println".to_string())));
            assert!(map.contains_key(&MapKey::String("FD".to_string())));
            let print_val = map.get(&MapKey::String("print".to_string())).unwrap();
            let println_val = map.get(&MapKey::String("println".to_string())).unwrap();
            let fd_val = map.get(&MapKey::String("FD".to_string())).unwrap();
            assert!(matches!(print_val, Value::Function(_)));
            assert!(matches!(println_val, Value::Function(_)));
            assert!(matches!(fd_val, Value::Map(_)));
        } else {
            panic!("std module should be a map");
        }
    }

    #[test]
    fn test_std_module_includes_json() {
        let std_module = create_std_module();
        if let Value::Map(map) = std_module {
            assert!(map.contains_key(&MapKey::String("println".to_string())));
            assert!(map.contains_key(&MapKey::String("json".to_string())));

            // Check that json is a module (map)
            if let Some(Value::Map(json_map)) = map.get(&MapKey::String("json".to_string())) {
                assert!(json_map.contains_key(&MapKey::String("parse".to_string())));
                assert!(json_map.contains_key(&MapKey::String("generate".to_string())));
            } else {
                panic!("JSON should be a module (map)");
            }
        } else {
            panic!("std module should be a map");
        }
    }

    #[test]
    fn test_std_module_includes_yaml() {
        let std_module = create_std_module();
        if let Value::Map(map) = std_module {
            assert!(map.contains_key(&MapKey::String("yaml".to_string())));

            // Check that yaml is a module (map)
            if let Some(Value::Map(yaml_map)) = map.get(&MapKey::String("yaml".to_string())) {
                assert!(yaml_map.contains_key(&MapKey::String("parse".to_string())));
                assert!(yaml_map.contains_key(&MapKey::String("generate".to_string())));
            } else {
                panic!("YAML should be a module (map)");
            }
        } else {
            panic!("std module should be a map");
        }
    }

    #[test]
    fn test_std_module_includes_toml() {
        let std_module = create_std_module();
        if let Value::Map(map) = std_module {
            assert!(map.contains_key(&MapKey::String("toml".to_string())));

            // Check that toml is a module (map)
            if let Some(Value::Map(toml_map)) = map.get(&MapKey::String("toml".to_string())) {
                assert!(toml_map.contains_key(&MapKey::String("parse".to_string())));
                assert!(toml_map.contains_key(&MapKey::String("generate".to_string())));
            } else {
                panic!("TOML should be a module (map)");
            }
        } else {
            panic!("std module should be a map");
        }
    }

    #[test]
    fn test_fd_in_std_module() {
        let std_module = create_std_module();

        if let Value::Map(std_map) = std_module {
            assert!(std_map.contains_key(&MapKey::String("FD".to_string())));

            let fd_module = std_map.get(&MapKey::String("FD".to_string())).unwrap();
            if let Value::Map(fd_map) = fd_module {
                assert!(fd_map.contains_key(&MapKey::String("stdin".to_string())));
                assert!(fd_map.contains_key(&MapKey::String("stdout".to_string())));
                assert!(fd_map.contains_key(&MapKey::String("stderr".to_string())));

                // Verify stream types
                let stdin = fd_map.get(&MapKey::String("stdin".to_string())).unwrap();
                let stdout = fd_map.get(&MapKey::String("stdout".to_string())).unwrap();
                let stderr = fd_map.get(&MapKey::String("stderr".to_string())).unwrap();

                assert!(matches!(stdin, Value::Stream(_)));
                assert!(matches!(stdout, Value::Stream(_)));
                assert!(matches!(stderr, Value::Stream(_)));
            } else {
                panic!("FD should be a map");
            }
        } else {
            panic!("std module should be a map");
        }
    }

    #[test]
    fn test_fd_stream_properties() {
        let fd_module = create_fd_module();

        if let Value::Map(fd_map) = fd_module {
            let stdin = fd_map.get(&MapKey::String("stdin".to_string())).unwrap();
            let stdout = fd_map.get(&MapKey::String("stdout".to_string())).unwrap();
            let stderr = fd_map.get(&MapKey::String("stderr".to_string())).unwrap();

            // Verify stream properties
            if let Value::Stream(stdin_handle) = stdin {
                assert!(stdin_handle.is_readable());
                assert!(!stdin_handle.is_writable());
                assert_eq!(stdin_handle.name, "stdin");
                assert!(!stdin_handle.is_closed.get());
            } else {
                panic!("stdin should be a stream");
            }

            if let Value::Stream(stdout_handle) = stdout {
                assert!(!stdout_handle.is_readable());
                assert!(stdout_handle.is_writable());
                assert_eq!(stdout_handle.name, "stdout");
                assert!(!stdout_handle.is_closed.get());
            } else {
                panic!("stdout should be a stream");
            }

            if let Value::Stream(stderr_handle) = stderr {
                assert!(!stderr_handle.is_readable());
                assert!(stderr_handle.is_writable());
                assert_eq!(stderr_handle.name, "stderr");
                assert!(!stderr_handle.is_closed.get());
            } else {
                panic!("stderr should be a stream");
            }
        } else {
            panic!("FD module should be a map");
        }
    }
}
