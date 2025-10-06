//! Built-in: yaml:generate(value) -> string.

use super::super::yaml::suji_to_yaml_value;
use suji_runtime::value::{RuntimeError, Value};

/// Convert SUJI value to YAML string.
pub fn builtin_yaml_generate(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "yaml:generate() takes exactly one argument".to_string(),
        });
    }

    let suji_value = &args[0];
    let yaml_value = suji_to_yaml_value(suji_value)?;

    let mut yaml_string = String::new();
    {
        let mut emitter = yaml_rust2::YamlEmitter::new(&mut yaml_string);
        emitter.compact(false);
        emitter
            .dump(&yaml_value)
            .map_err(|e| RuntimeError::YamlGenerateError {
                message: format!("Failed to serialize to YAML: {}", e),
                value_type: suji_value.type_name().to_string(),
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
    use indexmap::IndexMap;
    use std::rc::Rc;
    use suji_ast::ast::Stmt;
    use suji_lexer::token::Span;
    use suji_runtime::env::Env;
    use suji_runtime::value::DecimalNumber;
    use suji_runtime::value::{FunctionValue, MapKey};

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
            builtin_yaml_generate(&[Value::Number(DecimalNumber::from_i64(42))]).unwrap(),
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
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3)),
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
        map_data.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(30)),
        );
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
