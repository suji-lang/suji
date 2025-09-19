use super::super::value::{RuntimeError, Value};
use super::common::ValueRef;

/// String methods: length(), split(separator=" "), to_number(), to_list(), index_of(), to_string()
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
            "contains" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "contains() takes exactly one argument".to_string(),
                    });
                }
                match &args[0] {
                    Value::String(substring) => Ok(Value::Boolean(s.contains(substring))),
                    _ => Err(RuntimeError::TypeError {
                        message: "contains() argument must be a string".to_string(),
                    }),
                }
            }
            "starts_with" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "starts_with() takes exactly one argument".to_string(),
                    });
                }
                match &args[0] {
                    Value::String(prefix) => Ok(Value::Boolean(s.starts_with(prefix))),
                    _ => Err(RuntimeError::TypeError {
                        message: "starts_with() argument must be a string".to_string(),
                    }),
                }
            }
            "ends_with" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "ends_with() takes exactly one argument".to_string(),
                    });
                }
                match &args[0] {
                    Value::String(suffix) => Ok(Value::Boolean(s.ends_with(suffix))),
                    _ => Err(RuntimeError::TypeError {
                        message: "ends_with() argument must be a string".to_string(),
                    }),
                }
            }
            "replace" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "replace() takes exactly two arguments".to_string(),
                    });
                }
                match (&args[0], &args[1]) {
                    (Value::String(old), Value::String(new)) => {
                        Ok(Value::String(s.replace(old, new)))
                    }
                    _ => Err(RuntimeError::TypeError {
                        message: "replace() arguments must be strings".to_string(),
                    }),
                }
            }
            "trim" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "trim() takes no arguments".to_string(),
                    });
                }
                Ok(Value::String(s.trim().to_string()))
            }
            "upper" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "upper() takes no arguments".to_string(),
                    });
                }
                Ok(Value::String(s.to_uppercase()))
            }
            "lower" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "lower() takes no arguments".to_string(),
                    });
                }
                Ok(Value::String(s.to_lowercase()))
            }
            "reverse" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "reverse() takes no arguments".to_string(),
                    });
                }
                Ok(Value::String(s.chars().rev().collect()))
            }
            "repeat" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "repeat() takes exactly one argument".to_string(),
                    });
                }
                match &args[0] {
                    Value::Number(count) => {
                        if *count < 0.0 || !count.is_finite() {
                            return Err(RuntimeError::TypeError {
                                message: "repeat() count must be a non-negative finite number"
                                    .to_string(),
                            });
                        }
                        let count = *count as usize;
                        Ok(Value::String(s.repeat(count)))
                    }
                    _ => Err(RuntimeError::TypeError {
                        message: "repeat() argument must be a number".to_string(),
                    }),
                }
            }
            "to_string" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_string() takes no arguments".to_string(),
                    });
                }
                Ok(Value::String(s.clone()))
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

    #[test]
    fn test_string_contains() {
        let s = Value::String("hello world".to_string());
        let receiver = ValueRef::Immutable(&s);

        // Test contains with existing substring
        let result = call_string_method(
            receiver,
            "contains",
            vec![Value::String("world".to_string())],
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test contains with non-existing substring
        let receiver2 = ValueRef::Immutable(&s);
        let result2 = call_string_method(
            receiver2,
            "contains",
            vec![Value::String("xyz".to_string())],
        )
        .unwrap();
        assert_eq!(result2, Value::Boolean(false));

        // Test error case - wrong argument type
        let receiver3 = ValueRef::Immutable(&s);
        let result3 = call_string_method(receiver3, "contains", vec![Value::Number(42.0)]);
        assert!(matches!(result3, Err(RuntimeError::TypeError { .. })));

        // Test error case - wrong number of arguments
        let receiver4 = ValueRef::Immutable(&s);
        let result4 = call_string_method(receiver4, "contains", vec![]);
        assert!(matches!(result4, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_string_starts_with() {
        let s = Value::String("hello world".to_string());
        let receiver = ValueRef::Immutable(&s);

        // Test starts_with with matching prefix
        let result = call_string_method(
            receiver,
            "starts_with",
            vec![Value::String("hello".to_string())],
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test starts_with with non-matching prefix
        let receiver2 = ValueRef::Immutable(&s);
        let result2 = call_string_method(
            receiver2,
            "starts_with",
            vec![Value::String("world".to_string())],
        )
        .unwrap();
        assert_eq!(result2, Value::Boolean(false));

        // Test error case - wrong argument type
        let receiver3 = ValueRef::Immutable(&s);
        let result3 = call_string_method(receiver3, "starts_with", vec![Value::Number(42.0)]);
        assert!(matches!(result3, Err(RuntimeError::TypeError { .. })));
    }

    #[test]
    fn test_string_ends_with() {
        let s = Value::String("hello world".to_string());
        let receiver = ValueRef::Immutable(&s);

        // Test ends_with with matching suffix
        let result = call_string_method(
            receiver,
            "ends_with",
            vec![Value::String("world".to_string())],
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test ends_with with non-matching suffix
        let receiver2 = ValueRef::Immutable(&s);
        let result2 = call_string_method(
            receiver2,
            "ends_with",
            vec![Value::String("hello".to_string())],
        )
        .unwrap();
        assert_eq!(result2, Value::Boolean(false));

        // Test error case - wrong argument type
        let receiver3 = ValueRef::Immutable(&s);
        let result3 = call_string_method(receiver3, "ends_with", vec![Value::Number(42.0)]);
        assert!(matches!(result3, Err(RuntimeError::TypeError { .. })));
    }

    #[test]
    fn test_string_replace() {
        let s = Value::String("hello world".to_string());
        let receiver = ValueRef::Immutable(&s);

        // Test replace with single occurrence
        let result = call_string_method(
            receiver,
            "replace",
            vec![
                Value::String("world".to_string()),
                Value::String("universe".to_string()),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::String("hello universe".to_string()));

        // Test replace with multiple occurrences
        let s2 = Value::String("hello hello world".to_string());
        let receiver2 = ValueRef::Immutable(&s2);
        let result2 = call_string_method(
            receiver2,
            "replace",
            vec![
                Value::String("hello".to_string()),
                Value::String("hi".to_string()),
            ],
        )
        .unwrap();
        assert_eq!(result2, Value::String("hi hi world".to_string()));

        // Test error case - wrong argument types
        let receiver3 = ValueRef::Immutable(&s);
        let result3 = call_string_method(
            receiver3,
            "replace",
            vec![Value::Number(42.0), Value::String("test".to_string())],
        );
        assert!(matches!(result3, Err(RuntimeError::TypeError { .. })));

        // Test error case - wrong number of arguments
        let receiver4 = ValueRef::Immutable(&s);
        let result4 = call_string_method(
            receiver4,
            "replace",
            vec![Value::String("test".to_string())],
        );
        assert!(matches!(result4, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_string_trim() {
        let s = Value::String("  hello world  ".to_string());
        let receiver = ValueRef::Immutable(&s);

        let result = call_string_method(receiver, "trim", vec![]).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));

        // Test with only leading whitespace
        let s2 = Value::String("  hello".to_string());
        let receiver2 = ValueRef::Immutable(&s2);
        let result2 = call_string_method(receiver2, "trim", vec![]).unwrap();
        assert_eq!(result2, Value::String("hello".to_string()));

        // Test with only trailing whitespace
        let s3 = Value::String("hello  ".to_string());
        let receiver3 = ValueRef::Immutable(&s3);
        let result3 = call_string_method(receiver3, "trim", vec![]).unwrap();
        assert_eq!(result3, Value::String("hello".to_string()));

        // Test with no whitespace
        let s4 = Value::String("hello".to_string());
        let receiver4 = ValueRef::Immutable(&s4);
        let result4 = call_string_method(receiver4, "trim", vec![]).unwrap();
        assert_eq!(result4, Value::String("hello".to_string()));

        // Test error case - with arguments
        let receiver5 = ValueRef::Immutable(&s);
        let result5 =
            call_string_method(receiver5, "trim", vec![Value::String("test".to_string())]);
        assert!(matches!(result5, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_string_upper() {
        let s = Value::String("hello world".to_string());
        let receiver = ValueRef::Immutable(&s);

        let result = call_string_method(receiver, "upper", vec![]).unwrap();
        assert_eq!(result, Value::String("HELLO WORLD".to_string()));

        // Test with mixed case
        let s2 = Value::String("Hello World".to_string());
        let receiver2 = ValueRef::Immutable(&s2);
        let result2 = call_string_method(receiver2, "upper", vec![]).unwrap();
        assert_eq!(result2, Value::String("HELLO WORLD".to_string()));

        // Test error case - with arguments
        let receiver3 = ValueRef::Immutable(&s);
        let result3 =
            call_string_method(receiver3, "upper", vec![Value::String("test".to_string())]);
        assert!(matches!(result3, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_string_lower() {
        let s = Value::String("HELLO WORLD".to_string());
        let receiver = ValueRef::Immutable(&s);

        let result = call_string_method(receiver, "lower", vec![]).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));

        // Test with mixed case
        let s2 = Value::String("Hello World".to_string());
        let receiver2 = ValueRef::Immutable(&s2);
        let result2 = call_string_method(receiver2, "lower", vec![]).unwrap();
        assert_eq!(result2, Value::String("hello world".to_string()));

        // Test error case - with arguments
        let receiver3 = ValueRef::Immutable(&s);
        let result3 =
            call_string_method(receiver3, "lower", vec![Value::String("test".to_string())]);
        assert!(matches!(result3, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_string_reverse() {
        let s = Value::String("hello".to_string());
        let receiver = ValueRef::Immutable(&s);

        let result = call_string_method(receiver, "reverse", vec![]).unwrap();
        assert_eq!(result, Value::String("olleh".to_string()));

        // Test with empty string
        let s2 = Value::String("".to_string());
        let receiver2 = ValueRef::Immutable(&s2);
        let result2 = call_string_method(receiver2, "reverse", vec![]).unwrap();
        assert_eq!(result2, Value::String("".to_string()));

        // Test with single character
        let s3 = Value::String("a".to_string());
        let receiver3 = ValueRef::Immutable(&s3);
        let result3 = call_string_method(receiver3, "reverse", vec![]).unwrap();
        assert_eq!(result3, Value::String("a".to_string()));

        // Test error case - with arguments
        let receiver4 = ValueRef::Immutable(&s);
        let result4 = call_string_method(
            receiver4,
            "reverse",
            vec![Value::String("test".to_string())],
        );
        assert!(matches!(result4, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_string_repeat() {
        let s = Value::String("ha".to_string());
        let receiver = ValueRef::Immutable(&s);

        // Test repeat with positive count
        let result = call_string_method(receiver, "repeat", vec![Value::Number(3.0)]).unwrap();
        assert_eq!(result, Value::String("hahaha".to_string()));

        // Test repeat with zero count
        let receiver2 = ValueRef::Immutable(&s);
        let result2 = call_string_method(receiver2, "repeat", vec![Value::Number(0.0)]).unwrap();
        assert_eq!(result2, Value::String("".to_string()));

        // Test repeat with one count
        let receiver3 = ValueRef::Immutable(&s);
        let result3 = call_string_method(receiver3, "repeat", vec![Value::Number(1.0)]).unwrap();
        assert_eq!(result3, Value::String("ha".to_string()));

        // Test error case - negative count
        let receiver4 = ValueRef::Immutable(&s);
        let result4 = call_string_method(receiver4, "repeat", vec![Value::Number(-1.0)]);
        assert!(matches!(result4, Err(RuntimeError::TypeError { .. })));

        // Test error case - non-finite count
        let receiver5 = ValueRef::Immutable(&s);
        let result5 = call_string_method(receiver5, "repeat", vec![Value::Number(f64::INFINITY)]);
        assert!(matches!(result5, Err(RuntimeError::TypeError { .. })));

        // Test error case - wrong argument type
        let receiver6 = ValueRef::Immutable(&s);
        let result6 = call_string_method(receiver6, "repeat", vec![Value::String("3".to_string())]);
        assert!(matches!(result6, Err(RuntimeError::TypeError { .. })));

        // Test error case - wrong number of arguments
        let receiver7 = ValueRef::Immutable(&s);
        let result7 = call_string_method(receiver7, "repeat", vec![]);
        assert!(matches!(result7, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_string_to_string() {
        let s = Value::String("hello world".to_string());
        let receiver = ValueRef::Immutable(&s);
        let result = call_string_method(receiver, "to_string", vec![]).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));

        // Test with empty string
        let empty = Value::String("".to_string());
        let receiver2 = ValueRef::Immutable(&empty);
        let result2 = call_string_method(receiver2, "to_string", vec![]).unwrap();
        assert_eq!(result2, Value::String("".to_string()));

        // Test with special characters
        let special = Value::String("Hello, 世界! 🌍".to_string());
        let receiver3 = ValueRef::Immutable(&special);
        let result3 = call_string_method(receiver3, "to_string", vec![]).unwrap();
        assert_eq!(result3, Value::String("Hello, 世界! 🌍".to_string()));
    }

    #[test]
    fn test_string_to_string_arity_mismatch() {
        let s = Value::String("test".to_string());
        let receiver = ValueRef::Immutable(&s);
        let result = call_string_method(receiver, "to_string", vec![Value::Number(1.0)]);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }
}
