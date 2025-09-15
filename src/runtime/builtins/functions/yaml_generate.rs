//! YAML generate builtin function implementation
//!
//! This module contains the implementation of the yaml:generate builtin function.

use super::super::yaml::nn_to_yaml_value;
use crate::runtime::value::{RuntimeError, Value};

/// YAML generate function - converts nn value to YAML string
pub fn builtin_yaml_generate(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "yaml:generate() takes exactly one argument".to_string(),
        });
    }

    let nn_value = &args[0];
    let yaml_value = nn_to_yaml_value(nn_value)?;

    let mut yaml_string = String::new();
    {
        let mut emitter = yaml_rust2::YamlEmitter::new(&mut yaml_string);
        emitter.compact(false);
        emitter
            .dump(&yaml_value)
            .map_err(|e| RuntimeError::YamlGenerateError {
                message: format!("Failed to serialize to YAML: {}", e),
                value_type: nn_value.type_name().to_string(),
            })?;
    }

    // Replace ~ with null for consistency with tests
    let mut yaml_string = yaml_string.replace("~", "null");

    // Remove document separator if present
    if yaml_string.starts_with("---\n") {
        yaml_string = yaml_string[4..].to_string();
    }

    // Remove trailing newline if present (println will add its own)
    let trimmed_yaml = yaml_string.trim_end();
    Ok(Value::String(trimmed_yaml.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Stmt;
    use crate::runtime::env::Env;
    use crate::runtime::value::{FunctionValue, MapKey};
    use crate::token::Span;
    use indexmap::IndexMap;
    use std::rc::Rc;

    #[test]
    fn test_yaml_generate_simple_values() {
        assert_eq!(
            builtin_yaml_generate(&[Value::Nil]).unwrap(),
            Value::String("null".to_string())
        );
        assert_eq!(
            builtin_yaml_generate(&[Value::Boolean(true)]).unwrap(),
            Value::String("true".to_string())
        );
        assert_eq!(
            builtin_yaml_generate(&[Value::Boolean(false)]).unwrap(),
            Value::String("false".to_string())
        );
        assert_eq!(
            builtin_yaml_generate(&[Value::Number(42.0)]).unwrap(),
            Value::String("42".to_string())
        );
        assert_eq!(
            builtin_yaml_generate(&[Value::String("hello".to_string())]).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_yaml_generate_arrays() {
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let result = builtin_yaml_generate(&[list]).unwrap();
        assert_eq!(result, Value::String("- 1\n- 2\n- 3".to_string()));
    }

    #[test]
    fn test_yaml_generate_objects() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map_data.insert(MapKey::String("age".to_string()), Value::Number(30.0));
        let map = Value::Map(map_data);

        let result = builtin_yaml_generate(&[map]).unwrap();
        // Note: Order may vary, so we check if it contains the expected keys
        let yaml_str = if let Value::String(s) = result {
            s
        } else {
            panic!("Expected string")
        };
        assert!(yaml_str.contains("name: Alice"));
        assert!(yaml_str.contains("age: 30"));
    }

    #[test]
    fn test_yaml_generate_unsupported_types() {
        // Test regex (should fail)
        let regex = Value::Regex(regex::Regex::new("test").unwrap());
        let result = builtin_yaml_generate(&[regex]);
        assert!(matches!(
            result,
            Err(RuntimeError::YamlGenerateError { .. })
        ));

        // Test function (should fail)
        let func = Value::Function(FunctionValue {
            params: vec![],
            body: Stmt::Block {
                statements: vec![],
                span: Span::default(),
            },
            env: Rc::new(Env::new()),
        });
        let result = builtin_yaml_generate(&[func]);
        assert!(matches!(
            result,
            Err(RuntimeError::YamlGenerateError { .. })
        ));
    }
}
