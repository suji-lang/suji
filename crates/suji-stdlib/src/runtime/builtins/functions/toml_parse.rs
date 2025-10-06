//! Built-in: toml:parse(string) -> value.

use super::super::toml::toml_to_suji_value;
use suji_runtime::value::{RuntimeError, Value};

/// Parse TOML string to SUJI value.
pub fn builtin_toml_parse(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "toml:parse() takes exactly one argument".to_string(),
        });
    }

    let toml_string = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "toml:parse() argument must be a string".to_string(),
            });
        }
    };

    // Parse TOML string
    let toml_value: toml::Value =
        toml::from_str(toml_string).map_err(|e| RuntimeError::TomlParseError {
            message: format!("Invalid TOML: {}", e),
            toml_input: Some(toml_string.clone()),
        })?;

    // Convert TOML value to SUJI value
    toml_to_suji_value(toml_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_runtime::value::DecimalNumber;

    #[test]
    fn test_toml_parse_simple_values() {
        // Test parsing simple TOML values
        assert_eq!(
            builtin_toml_parse(&[Value::String("value = true".to_string())]).unwrap(),
            Value::Map({
                let mut map = indexmap::IndexMap::new();
                map.insert(
                    suji_runtime::value::MapKey::String("value".to_string()),
                    Value::Boolean(true),
                );
                map
            })
        );
        assert_eq!(
            builtin_toml_parse(&[Value::String("value = false".to_string())]).unwrap(),
            Value::Map({
                let mut map = indexmap::IndexMap::new();
                map.insert(
                    suji_runtime::value::MapKey::String("value".to_string()),
                    Value::Boolean(false),
                );
                map
            })
        );
        assert_eq!(
            builtin_toml_parse(&[Value::String("value = 42".to_string())]).unwrap(),
            Value::Map({
                let mut map = indexmap::IndexMap::new();
                map.insert(
                    suji_runtime::value::MapKey::String("value".to_string()),
                    Value::Number(DecimalNumber::from_i64(42)),
                );
                map
            })
        );
        assert_eq!(
            builtin_toml_parse(&[Value::String("value = \"hello\"".to_string())]).unwrap(),
            Value::Map({
                let mut map = indexmap::IndexMap::new();
                map.insert(
                    suji_runtime::value::MapKey::String("value".to_string()),
                    Value::String("hello".to_string()),
                );
                map
            })
        );
    }

    #[test]
    fn test_toml_parse_arrays() {
        let result =
            builtin_toml_parse(&[Value::String("values = [1, 2, 3]".to_string())]).unwrap();
        if let Value::Map(map) = result {
            if let Some(Value::List(items)) =
                map.get(&suji_runtime::value::MapKey::String("values".to_string()))
            {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(1)));
                assert_eq!(items[1], Value::Number(DecimalNumber::from_i64(2)));
                assert_eq!(items[2], Value::Number(DecimalNumber::from_i64(3)));
            } else {
                panic!("Expected list in values key");
            }
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_toml_parse_tables() {
        let result =
            builtin_toml_parse(&[Value::String("name = \"Alice\"\nage = 30".to_string())]).unwrap();
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
    fn test_toml_parse_invalid_toml() {
        let result = builtin_toml_parse(&[Value::String("invalid toml".to_string())]);
        assert!(matches!(result, Err(RuntimeError::TomlParseError { .. })));
    }

    #[test]
    fn test_toml_parse_wrong_argument_type() {
        let result = builtin_toml_parse(&[Value::Number(DecimalNumber::from_i64(42))]);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }
}
