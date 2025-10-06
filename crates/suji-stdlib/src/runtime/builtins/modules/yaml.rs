//! YAML module creation.

use indexmap::IndexMap;
use suji_runtime::value::{MapKey, Value};

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

/// Create the yaml module as a Value::Map
pub fn create_yaml_module() -> Value {
    let mut yaml_map = IndexMap::new();

    // Add yaml functions to yaml module
    yaml_map.insert(
        MapKey::String("parse".to_string()),
        create_builtin_function_value("yaml_parse"),
    );
    yaml_map.insert(
        MapKey::String("generate".to_string()),
        create_builtin_function_value("yaml_generate"),
    );

    Value::Map(yaml_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_module_creation() {
        let yaml_module = create_yaml_module();
        if let Value::Map(map) = yaml_module {
            assert!(map.contains_key(&MapKey::String("parse".to_string())));
            assert!(map.contains_key(&MapKey::String("generate".to_string())));
        } else {
            panic!("YAML module should be a map");
        }
    }
}
