//! Built-in: json:parse(string) -> value.

use super::super::json::json_to_nn_value;
use crate::runtime::value::{RuntimeError, Value};

/// Parse JSON string to nn value.
pub fn builtin_json_parse(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "json:parse() takes exactly one argument".to_string(),
        });
    }

    let json_string = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "json:parse() argument must be a string".to_string(),
            });
        }
    };

    // Parse JSON string
    let json_value: serde_json::Value =
        serde_json::from_str(json_string).map_err(|e| RuntimeError::JsonParseError {
            message: format!("Invalid JSON: {}", e),
            json_input: Some(json_string.clone()),
        })?;

    // Convert JSON value to nn value
    json_to_nn_value(json_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parse_simple_values() {
        // Test parsing simple JSON values
        assert_eq!(
            builtin_json_parse(&[Value::String("null".to_string())]).unwrap(),
            Value::Nil
        );
        assert_eq!(
            builtin_json_parse(&[Value::String("true".to_string())]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            builtin_json_parse(&[Value::String("false".to_string())]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            builtin_json_parse(&[Value::String("42".to_string())]).unwrap(),
            Value::Number(42.0)
        );
        assert_eq!(
            builtin_json_parse(&[Value::String("\"hello\"".to_string())]).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_json_parse_arrays() {
        let result = builtin_json_parse(&[Value::String("[1, 2, 3]".to_string())]).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Number(1.0));
            assert_eq!(items[1], Value::Number(2.0));
            assert_eq!(items[2], Value::Number(3.0));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_json_parse_objects() {
        let result = builtin_json_parse(&[Value::String(
            "{\"name\": \"Alice\", \"age\": 30}".to_string(),
        )])
        .unwrap();
        if let Value::Map(map) = result {
            assert_eq!(map.len(), 2);
            assert_eq!(
                map.get(&crate::runtime::value::MapKey::String("name".to_string())),
                Some(&Value::String("Alice".to_string()))
            );
            assert_eq!(
                map.get(&crate::runtime::value::MapKey::String("age".to_string())),
                Some(&Value::Number(30.0))
            );
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_json_parse_invalid_json() {
        let result = builtin_json_parse(&[Value::String("invalid json".to_string())]);
        assert!(matches!(result, Err(RuntimeError::JsonParseError { .. })));
    }

    #[test]
    fn test_json_parse_wrong_argument_type() {
        let result = builtin_json_parse(&[Value::Number(42.0)]);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }
}
