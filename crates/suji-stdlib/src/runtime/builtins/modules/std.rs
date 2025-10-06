//! Standard library module creation.

use super::{
    json::create_json_module, random::create_random_module, toml::create_toml_module,
    yaml::create_yaml_module,
};
use crate::runtime::builtins::suji_loader::{load_print, load_println};
use indexmap::IndexMap;
use std::rc::Rc;
use suji_runtime::env_overlay::EnvProxy;
use suji_runtime::value::{MapKey, StreamHandle, Value};

/// Create a special built-in function value that can be called normally
fn create_builtin_function_value(name: &str) -> Value {
    use std::rc::Rc;
    use suji_ast::ast::Stmt;
    use suji_lexer::token::Span;
    use suji_runtime::env::Env;
    use suji_runtime::value::{FunctionValue, ParamSpec};

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

/// Create the io module with stdio streams + open function
pub fn create_io_module() -> Value {
    let mut io_map = IndexMap::new();

    // Create shared stream handles - these will be the same instances
    // referenced by all imports of the module
    let stdin_stream = Rc::new(StreamHandle::new_stdin());
    let stdout_stream = Rc::new(StreamHandle::new_stdout());
    let stderr_stream = Rc::new(StreamHandle::new_stderr());

    // Insert into FD map
    io_map.insert(
        MapKey::String("stdin".to_string()),
        Value::Stream(stdin_stream),
    );
    io_map.insert(
        MapKey::String("stdout".to_string()),
        Value::Stream(stdout_stream),
    );
    io_map.insert(
        MapKey::String("stderr".to_string()),
        Value::Stream(stderr_stream),
    );

    // File opening function
    io_map.insert(
        MapKey::String("open".to_string()),
        create_builtin_function_value("io_open"),
    );

    Value::Map(io_map)
}

/// Create the ENV module exposing process environment via overlay proxy.
pub fn create_env_module() -> Value {
    // Create a shared EnvProxy instance
    let env_proxy = Rc::new(EnvProxy::new());
    Value::EnvMap(env_proxy)
}

/// Create the std module as a Value::Map
pub fn create_std_module() -> Value {
    let mut std_map = IndexMap::new();

    // Load SUJI-language print and println (pure SUJI definitions)
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

    // Build io module (renamed from FD) and attach
    std_map.insert(MapKey::String("io".to_string()), create_io_module());

    // Add random module to std module
    std_map.insert(MapKey::String("random".to_string()), create_random_module());

    // Build env module with `var` (renamed from ENV) and attach
    let mut env_map = IndexMap::new();
    env_map.insert(MapKey::String("var".to_string()), create_env_module());
    // Snapshot command-line arguments into maps at startup
    // Exclude interpreter name (argv[0]) and any interpreter options (leading '-' args)
    let mut args_map: IndexMap<MapKey, Value> = IndexMap::new();
    let mut iter = std::env::args();
    // Skip interpreter name
    let _ = iter.next();
    // Skip interpreter options (starting with '-') until first non-option (script path)
    let mut script_seen = false;
    for a in iter {
        if !script_seen {
            if a.starts_with('-') {
                continue;
            } else {
                // First non-option is script path; include as "0" and mark seen
                args_map.insert(MapKey::String("0".to_string()), Value::String(a));
                script_seen = true;
                continue;
            }
        }
        // Subsequent args are positional starting from "1"
        let idx = args_map.len().saturating_sub(1); // exclude "0"
        args_map.insert(MapKey::String(idx.to_string()), Value::String(a));
    }

    // Attach env submodules
    env_map.insert(
        MapKey::String("args".to_string()),
        Value::Map(args_map.clone()),
    );
    env_map.insert(MapKey::String("argv".to_string()), Value::Map(args_map));

    std_map.insert(MapKey::String("env".to_string()), Value::Map(env_map));

    Value::Map(std_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_module_creation() {
        let io_module = create_io_module();

        if let Value::Map(map) = io_module {
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
            panic!("io module should be a map");
        }
    }

    #[test]
    fn test_std_module_creation() {
        let std_module = create_std_module();

        if let Value::Map(map) = std_module {
            assert!(map.contains_key(&MapKey::String("print".to_string())));
            assert!(map.contains_key(&MapKey::String("println".to_string())));
            assert!(map.contains_key(&MapKey::String("io".to_string())));
            assert!(map.contains_key(&MapKey::String("env".to_string())));
            let print_val = map.get(&MapKey::String("print".to_string())).unwrap();
            let println_val = map.get(&MapKey::String("println".to_string())).unwrap();
            let io_val = map.get(&MapKey::String("io".to_string())).unwrap();
            let env_val = map.get(&MapKey::String("env".to_string())).unwrap();
            assert!(matches!(print_val, Value::Function(_)));
            assert!(matches!(println_val, Value::Function(_)));
            assert!(matches!(io_val, Value::Map(_)));
            assert!(matches!(env_val, Value::Map(_)));
        } else {
            panic!("std module should be a map");
        }
    }

    #[test]
    fn test_std_module_includes_serialization_modules() {
        let std_module = create_std_module();
        if let Value::Map(map) = std_module {
            // Test JSON module
            assert!(map.contains_key(&MapKey::String("json".to_string())));
            if let Some(Value::Map(json_map)) = map.get(&MapKey::String("json".to_string())) {
                assert!(json_map.contains_key(&MapKey::String("parse".to_string())));
                assert!(json_map.contains_key(&MapKey::String("generate".to_string())));
            } else {
                panic!("JSON should be a module (map)");
            }

            // Test YAML module
            assert!(map.contains_key(&MapKey::String("yaml".to_string())));
            if let Some(Value::Map(yaml_map)) = map.get(&MapKey::String("yaml".to_string())) {
                assert!(yaml_map.contains_key(&MapKey::String("parse".to_string())));
                assert!(yaml_map.contains_key(&MapKey::String("generate".to_string())));
            } else {
                panic!("YAML should be a module (map)");
            }

            // Test TOML module
            assert!(map.contains_key(&MapKey::String("toml".to_string())));
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
    fn test_io_in_std_module() {
        let std_module = create_std_module();

        if let Value::Map(std_map) = std_module {
            assert!(std_map.contains_key(&MapKey::String("io".to_string())));

            let io_module = std_map.get(&MapKey::String("io".to_string())).unwrap();
            if let Value::Map(io_map) = io_module {
                assert!(io_map.contains_key(&MapKey::String("stdin".to_string())));
                assert!(io_map.contains_key(&MapKey::String("stdout".to_string())));
                assert!(io_map.contains_key(&MapKey::String("stderr".to_string())));

                // Verify stream types
                let stdin = io_map.get(&MapKey::String("stdin".to_string())).unwrap();
                let stdout = io_map.get(&MapKey::String("stdout".to_string())).unwrap();
                let stderr = io_map.get(&MapKey::String("stderr".to_string())).unwrap();

                assert!(matches!(stdin, Value::Stream(_)));
                assert!(matches!(stdout, Value::Stream(_)));
                assert!(matches!(stderr, Value::Stream(_)));
            } else {
                panic!("io should be a map");
            }
        } else {
            panic!("std module should be a map");
        }
    }

    #[test]
    fn test_io_stream_properties() {
        let io_module = create_io_module();

        if let Value::Map(io_map) = io_module {
            let stdin = io_map.get(&MapKey::String("stdin".to_string())).unwrap();
            let stdout = io_map.get(&MapKey::String("stdout".to_string())).unwrap();
            let stderr = io_map.get(&MapKey::String("stderr".to_string())).unwrap();

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
            panic!("io module should be a map");
        }
    }
}
