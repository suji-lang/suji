//! TOML module creation
//!
//! This module handles the creation of the toml builtin module.

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

/// Create the toml module as a Value::Map
pub fn create_toml_module() -> Value {
    let mut toml_map = IndexMap::new();

    // Add toml functions to toml module
    toml_map.insert(
        MapKey::String("parse".to_string()),
        create_builtin_function_value("toml_parse"),
    );
    toml_map.insert(
        MapKey::String("generate".to_string()),
        create_builtin_function_value("toml_generate"),
    );

    Value::Map(toml_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_module_creation() {
        let toml_module = create_toml_module();
        if let Value::Map(map) = toml_module {
            assert!(map.contains_key(&MapKey::String("parse".to_string())));
            assert!(map.contains_key(&MapKey::String("generate".to_string())));
        } else {
            panic!("TOML module should be a map");
        }
    }
}
