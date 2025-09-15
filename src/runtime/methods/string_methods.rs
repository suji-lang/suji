use super::super::value::{RuntimeError, Value};
use super::common::ValueRef;

/// String methods: length(), split(separator=" "), to_number(), to_list(), index_of()
pub fn call_string_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::String(s) => match method {
            "length" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "length() takes no arguments".to_string(),
                    });
                }
                Ok(Value::Number(s.chars().count() as f64))
            }
            "split" => {
                let separator = if args.is_empty() {
                    " ".to_string()
                } else if args.len() == 1 {
                    match &args[0] {
                        Value::String(sep) => sep.clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "split() separator must be a string".to_string(),
                            });
                        }
                    }
                } else {
                    return Err(RuntimeError::ArityMismatch {
                        message: "split() takes at most one argument".to_string(),
                    });
                };

                let parts: Vec<Value> = s
                    .split(&separator)
                    .map(|part| Value::String(part.to_string()))
                    .collect();
                Ok(Value::List(parts))
            }
            "to_number" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_number() takes no arguments".to_string(),
                    });
                }
                s.parse::<f64>().map(Value::Number).map_err(|_| {
                    RuntimeError::InvalidNumberConversion {
                        message: format!("Cannot convert '{}' to number", s),
                    }
                })
            }
            "to_list" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_list() takes no arguments".to_string(),
                    });
                }
                let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
                Ok(Value::List(chars))
            }
            "index_of" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "index_of() takes exactly one argument".to_string(),
                    });
                }
                match &args[0] {
                    Value::String(substring) => {
                        let index = s.find(substring).map(|i| i as f64).unwrap_or(-1.0);
                        Ok(Value::Number(index))
                    }
                    _ => Err(RuntimeError::TypeError {
                        message: "index_of() argument must be a string".to_string(),
                    }),
                }
            }
            _ => Err(RuntimeError::MethodError {
                message: format!("String has no method '{}'", method),
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
    fn test_string_length() {
        let s = Value::String("hello".to_string());
        let receiver = ValueRef::Immutable(&s);
        let result = call_string_method(receiver, "length", vec![]).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_string_split() {
        let s = Value::String("a,b,c".to_string());
        let receiver = ValueRef::Immutable(&s);

        // Split with custom separator
        let result =
            call_string_method(receiver, "split", vec![Value::String(",".to_string())]).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::String("a".to_string()));
            assert_eq!(items[1], Value::String("b".to_string()));
            assert_eq!(items[2], Value::String("c".to_string()));
        } else {
            panic!("Expected list");
        }

        // Split with default separator
        let s2 = Value::String("hello world".to_string());
        let receiver2 = ValueRef::Immutable(&s2);
        let result2 = call_string_method(receiver2, "split", vec![]).unwrap();
        if let Value::List(items) = result2 {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], Value::String("hello".to_string()));
            assert_eq!(items[1], Value::String("world".to_string()));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_string_to_number() {
        let s = Value::String("123".to_string());
        let receiver = ValueRef::Immutable(&s);
        let result = call_string_method(receiver, "to_number", vec![]).unwrap();
        assert_eq!(result, Value::Number(123.0));

        let s2 = Value::String("123.45".to_string());
        let receiver2 = ValueRef::Immutable(&s2);
        let result2 = call_string_method(receiver2, "to_number", vec![]).unwrap();
        assert_eq!(result2, Value::Number(123.45));

        let s3 = Value::String("invalid".to_string());
        let receiver3 = ValueRef::Immutable(&s3);
        let result3 = call_string_method(receiver3, "to_number", vec![]);
        assert!(matches!(
            result3,
            Err(RuntimeError::InvalidNumberConversion { .. })
        ));
    }

    #[test]
    fn test_string_to_list() {
        let s = Value::String("hello".to_string());
        let receiver = ValueRef::Immutable(&s);
        let result = call_string_method(receiver, "to_list", vec![]).unwrap();

        if let Value::List(chars) = result {
            assert_eq!(chars.len(), 5);
            assert_eq!(chars[0], Value::String("h".to_string()));
            assert_eq!(chars[1], Value::String("e".to_string()));
            assert_eq!(chars[2], Value::String("l".to_string()));
            assert_eq!(chars[3], Value::String("l".to_string()));
            assert_eq!(chars[4], Value::String("o".to_string()));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_string_index_of() {
        let s = Value::String("hello world".to_string());
        let receiver = ValueRef::Immutable(&s);

        let result = call_string_method(
            receiver,
            "index_of",
            vec![Value::String("world".to_string())],
        )
        .unwrap();
        assert_eq!(result, Value::Number(6.0));

        let receiver2 = ValueRef::Immutable(&s);
        let result2 = call_string_method(
            receiver2,
            "index_of",
            vec![Value::String("xyz".to_string())],
        )
        .unwrap();
        assert_eq!(result2, Value::Number(-1.0));
    }
}
