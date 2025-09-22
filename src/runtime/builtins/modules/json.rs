//! JSON module creation.

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

/// Create the json module as a Value::Map
pub fn create_json_module() -> Value {
    let mut json_map = IndexMap::new();

    // Add json functions to json module
    json_map.insert(
        MapKey::String("parse".to_string()),
        create_builtin_function_value("json_parse"),
    );
    json_map.insert(
        MapKey::String("generate".to_string()),
        create_builtin_function_value("json_generate"),
    );

    Value::Map(json_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_module_creation() {
        let json_module = create_json_module();
        if let Value::Map(map) = json_module {
            assert!(map.contains_key(&MapKey::String("parse".to_string())));
            assert!(map.contains_key(&MapKey::String("generate".to_string())));
        } else {
            panic!("JSON module should be a map");
        }
    }
}
