// No executor needed
use super::super::value::{DecimalNumber, RuntimeError, Value};
use super::common::ValueRef;

/// Tuple methods: length(), to_list(), to_string()
pub fn call_tuple_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::Tuple(tuple) => match method {
            "length" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "length() takes no arguments".to_string(),
                    });
                }
                Ok(Value::Number(DecimalNumber::from_usize(tuple.len())))
            }
            "to_list" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_list() takes no arguments".to_string(),
                    });
                }
                Ok(Value::List(tuple.clone()))
            }
            "to_string" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_string() takes no arguments".to_string(),
                    });
                }
                Ok(Value::String(format!("{}", Value::Tuple(tuple.clone()))))
            }
            _ => Err(RuntimeError::MethodError {
                message: format!("Tuple has no method '{}'", method),
            }),
        },
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::ValueRef;
    use super::*;

    #[test]
    fn test_tuple_length() {
        let tuple = Value::Tuple(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::String("test".to_string()),
        ]);
        let receiver = ValueRef::Immutable(&tuple);
        let result = call_tuple_method(receiver, "length", vec![]).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));
    }

    #[test]
    fn test_tuple_to_list() {
        let tuple = Value::Tuple(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::String("test".to_string()),
        ]);
        let receiver = ValueRef::Immutable(&tuple);
        let result = call_tuple_method(receiver, "to_list", vec![]).unwrap();
        if let Value::List(list) = result {
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], Value::Number(DecimalNumber::from_i64(1)));
            assert_eq!(list[1], Value::String("test".to_string()));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_tuple_to_string() {
        let tuple = Value::Tuple(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::String("test".to_string()),
        ]);
        let receiver = ValueRef::Immutable(&tuple);
        let result = call_tuple_method(receiver, "to_string", vec![]).unwrap();
        if let Value::String(s) = result {
            assert!(s.contains("1"));
            assert!(s.contains("test"));
        } else {
            panic!("Expected string");
        }
    }

    // Additional edge case tests
    #[test]
    fn test_tuple_length_empty() {
        let tuple = Value::Tuple(vec![]);
        let receiver = ValueRef::Immutable(&tuple);
        let result = call_tuple_method(receiver, "length", vec![]).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(0)));
    }

    #[test]
    fn test_tuple_length_single() {
        let tuple = Value::Tuple(vec![Value::Number(DecimalNumber::from_i64(42))]);
        let receiver = ValueRef::Immutable(&tuple);
        let result = call_tuple_method(receiver, "length", vec![]).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));
    }

    #[test]
    fn test_tuple_to_list_empty() {
        let tuple = Value::Tuple(vec![]);
        let receiver = ValueRef::Immutable(&tuple);
        let result = call_tuple_method(receiver, "to_list", vec![]).unwrap();
        if let Value::List(list) = result {
            assert_eq!(list.len(), 0);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_tuple_to_string_single_element() {
        let tuple = Value::Tuple(vec![Value::Number(DecimalNumber::from_i64(42))]);
        let receiver = ValueRef::Immutable(&tuple);
        let result = call_tuple_method(receiver, "to_string", vec![]).unwrap();
        if let Value::String(s) = result {
            assert!(s.contains("42"));
            assert!(s.contains(",")); // Single element tuple should have trailing comma
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_tuple_methods_arity_mismatch() {
        let tuple = Value::Tuple(vec![Value::Number(DecimalNumber::from_i64(1))]);

        // Test length with arguments
        let receiver1 = ValueRef::Immutable(&tuple);
        let result1 = call_tuple_method(
            receiver1,
            "length",
            vec![Value::Number(DecimalNumber::from_i64(1))],
        );
        assert!(matches!(result1, Err(RuntimeError::ArityMismatch { .. })));

        // Test to_list with arguments
        let receiver2 = ValueRef::Immutable(&tuple);
        let result2 = call_tuple_method(
            receiver2,
            "to_list",
            vec![Value::Number(DecimalNumber::from_i64(1))],
        );
        assert!(matches!(result2, Err(RuntimeError::ArityMismatch { .. })));

        // Test to_string with arguments
        let receiver3 = ValueRef::Immutable(&tuple);
        let result3 = call_tuple_method(
            receiver3,
            "to_string",
            vec![Value::Number(DecimalNumber::from_i64(1))],
        );
        assert!(matches!(result3, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_tuple_invalid_method() {
        let tuple = Value::Tuple(vec![Value::Number(DecimalNumber::from_i64(1))]);
        let receiver = ValueRef::Immutable(&tuple);

        let result = call_tuple_method(receiver, "invalid_method", vec![]);
        assert!(matches!(result, Err(RuntimeError::MethodError { .. })));
    }
}
