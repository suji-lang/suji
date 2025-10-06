//! Built-in: yaml:parse(string) -> value.

use super::super::yaml::yaml_to_suji_value;
use suji_runtime::value::{RuntimeError, Value};

/// Parse YAML string to SUJI value.
pub fn builtin_yaml_parse(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "yaml:parse() takes exactly one argument".to_string(),
        });
    }

    let yaml_string = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "yaml:parse() argument must be a string".to_string(),
            });
        }
    };

    // Parse YAML string
    let yaml_value: yaml_rust2::Yaml = yaml_rust2::YamlLoader::load_from_str(yaml_string)
        .map_err(|e| RuntimeError::YamlParseError {
            message: format!("Invalid YAML: {}", e),
            yaml_input: Some(yaml_string.clone()),
        })?
        .into_iter()
        .next()
        .ok_or_else(|| RuntimeError::YamlParseError {
            message: "Empty YAML document".to_string(),
            yaml_input: Some(yaml_string.clone()),
        })?;

    // Convert YAML value to SUJI value
    yaml_to_suji_value(yaml_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_runtime::value::DecimalNumber;

    #[test]
    fn test_yaml_parse_simple_values() {
        // Test parsing simple YAML values
        assert_eq!(
            builtin_yaml_parse(&[Value::String("null".to_string())]).unwrap(),
            Value::Nil
        );
        assert_eq!(
            builtin_yaml_parse(&[Value::String("true".to_string())]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            builtin_yaml_parse(&[Value::String("false".to_string())]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            builtin_yaml_parse(&[Value::String("42".to_string())]).unwrap(),
            Value::Number(DecimalNumber::from_i64(42))
        );
        assert_eq!(
            builtin_yaml_parse(&[Value::String("hello".to_string())]).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_yaml_parse_arrays() {
        let result = builtin_yaml_parse(&[Value::String("- 1\n- 2\n- 3".to_string())]).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(1)));
            assert_eq!(items[1], Value::Number(DecimalNumber::from_i64(2)));
            assert_eq!(items[2], Value::Number(DecimalNumber::from_i64(3)));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_yaml_parse_objects() {
        let result =
            builtin_yaml_parse(&[Value::String("name: Alice\nage: 30".to_string())]).unwrap();
        if let Value::Map(map) = result {
            assert_eq!(map.len(), 2);
            assert_eq!(
                map.get(&suji_runtime::value::MapKey::String("name".to_string())),
                Some(&Value::String("Alice".to_string()))
            );
            assert_eq!(
                map.get(&suji_runtime::value::MapKey::String("age".to_string())),
                Some(&Value::Number(DecimalNumber::from_i64(30)))
            );
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_yaml_parse_invalid_yaml() {
        // Use a more obviously invalid YAML that should cause a parse error
        let result = builtin_yaml_parse(&[Value::String("key: [unclosed array".to_string())]);
        match result {
            Err(RuntimeError::YamlParseError { .. }) => {
                // Expected error type
            }
            Err(e) => {
                panic!("Expected YamlParseError, got: {:?}", e);
            }
            Ok(_) => {
                panic!("Expected error, got success");
            }
        }
    }

    #[test]
    fn test_yaml_parse_wrong_argument_type() {
        let result = builtin_yaml_parse(&[Value::Number(DecimalNumber::from_i64(42))]);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }
}
