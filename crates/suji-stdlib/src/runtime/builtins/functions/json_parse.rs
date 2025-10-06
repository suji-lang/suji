//! Built-in: json:parse(string) -> value.

use super::super::json::json_to_suji_value;
use suji_runtime::value::{RuntimeError, Value};

/// Parse JSON string to SUJI value.
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

    // Convert JSON value to SUJI value
    json_to_suji_value(json_value)
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
            Value::Number(suji_runtime::value::DecimalNumber::parse("42").unwrap())
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
            assert_eq!(
                items[0],
                Value::Number(suji_runtime::value::DecimalNumber::parse("1").unwrap())
            );
            assert_eq!(
                items[1],
                Value::Number(suji_runtime::value::DecimalNumber::parse("2").unwrap())
            );
            assert_eq!(
                items[2],
                Value::Number(suji_runtime::value::DecimalNumber::parse("3").unwrap())
            );
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
                map.get(&suji_runtime::value::MapKey::String("name".to_string())),
                Some(&Value::String("Alice".to_string()))
            );
            assert_eq!(
                map.get(&suji_runtime::value::MapKey::String("age".to_string())),
                Some(&Value::Number(
                    suji_runtime::value::DecimalNumber::parse("30").unwrap()
                ))
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
        let result = builtin_json_parse(&[Value::Number(
            suji_runtime::value::DecimalNumber::parse("42").unwrap(),
        )]);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }
}
