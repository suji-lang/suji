//! Standard library module creation
//!
//! This module handles the creation of the std builtin module.

use super::{json::create_json_module, toml::create_toml_module, yaml::create_yaml_module};
use crate::runtime::value::{MapKey, Value};
use indexmap::IndexMap;

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

/// Create the std module as a Value::Map
pub fn create_std_module() -> Value {
    let mut std_map = IndexMap::new();

    // Add print to std module
    std_map.insert(
        MapKey::String("print".to_string()),
        create_builtin_function_value("print"),
    );

    // Add println to std module
    std_map.insert(
        MapKey::String("println".to_string()),
        create_builtin_function_value("println"),
    );

    // Add json module to std module
    std_map.insert(MapKey::String("json".to_string()), create_json_module());

    // Add yaml module to std module
    std_map.insert(MapKey::String("yaml".to_string()), create_yaml_module());

    // Add toml module to std module
    std_map.insert(MapKey::String("toml".to_string()), create_toml_module());

    Value::Map(std_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_std_module_creation() {
        let std_module = create_std_module();

        if let Value::Map(map) = std_module {
            assert!(map.contains_key(&MapKey::String("print".to_string())));
            assert!(map.contains_key(&MapKey::String("println".to_string())));
            let print_val = map.get(&MapKey::String("print".to_string())).unwrap();
            let println_val = map.get(&MapKey::String("println".to_string())).unwrap();
            assert!(matches!(print_val, Value::Function(_)));
            assert!(matches!(println_val, Value::Function(_)));
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
}
