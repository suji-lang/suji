// No executor needed
use super::super::value::{RuntimeError, Value};
use super::common::ValueRef;

/// Number methods: to_string(), is_int(), abs(), ceil(), floor(), round(), sqrt(), pow(), min(), max()
pub fn call_number_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::Number(n) => match method {
            "to_string" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_string() takes no arguments".to_string(),
                    });
                }
                Ok(Value::String(n.to_string()))
            }
            "is_int" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "is_int() takes no arguments".to_string(),
                    });
                }
                Ok(Value::Boolean(n.is_integer()))
            }
            "abs" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "abs() takes no arguments".to_string(),
                    });
                }
                Ok(Value::Number(n.abs()))
            }
            "ceil" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "ceil() takes no arguments".to_string(),
                    });
                }
                Ok(Value::Number(n.ceil()))
            }
            "floor" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "floor() takes no arguments".to_string(),
                    });
                }
                Ok(Value::Number(n.floor()))
            }
            "round" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "round() takes no arguments".to_string(),
                    });
                }
                Ok(Value::Number(n.round()))
            }
            "sqrt" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "sqrt() takes no arguments".to_string(),
                    });
                }
                match n.sqrt() {
                    Ok(result) => Ok(Value::Number(result)),
                    Err(err) => Err(RuntimeError::InvalidOperation {
                        message: err.to_string(),
                    }),
                }
            }
            "pow" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "pow() takes exactly one argument".to_string(),
                    });
                }
                match &args[0] {
                    Value::Number(exponent) => match n.pow(exponent) {
                        Ok(result) => Ok(Value::Number(result)),
                        Err(err) => Err(RuntimeError::InvalidOperation {
                            message: err.to_string(),
                        }),
                    },
                    _ => Err(RuntimeError::TypeError {
                        message: "pow() argument must be a number".to_string(),
                    }),
                }
            }
            "min" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "min() takes exactly one argument".to_string(),
                    });
                }
                match &args[0] {
                    Value::Number(other) => Ok(Value::Number(n.min(other))),
                    _ => Err(RuntimeError::TypeError {
                        message: "min() argument must be a number".to_string(),
                    }),
                }
            }
            "max" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "max() takes exactly one argument".to_string(),
                    });
                }
                match &args[0] {
                    Value::Number(other) => Ok(Value::Number(n.max(other))),
                    _ => Err(RuntimeError::TypeError {
                        message: "max() argument must be a number".to_string(),
                    }),
                }
            }
            _ => Err(RuntimeError::MethodError {
                message: format!("Number has no method '{}'", method),
            }),
        },
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::value::DecimalNumber;
    use super::super::common::ValueRef;
    use super::*;

    #[test]
    fn test_number_to_string() {
        let n = Value::Number(DecimalNumber::from_i64(42));
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(receiver, "to_string", vec![]).unwrap();
        assert_eq!(result, Value::String("42".to_string()));

        let n2 = Value::Number(DecimalNumber::parse("2.5").unwrap());
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_number_method(receiver2, "to_string", vec![]).unwrap();
        assert_eq!(result2, Value::String("2.5".to_string()));
    }

    #[test]
    fn test_number_is_int() {
        let n = Value::Number(DecimalNumber::from_i64(42));
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(receiver, "is_int", vec![]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        let n2 = Value::Number(DecimalNumber::parse("2.5").unwrap());
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_number_method(receiver2, "is_int", vec![]).unwrap();
        assert_eq!(result2, Value::Boolean(false));
    }

    #[test]
    fn test_number_abs() {
        let n = Value::Number(DecimalNumber::from_i64(-5));
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(receiver, "abs", vec![]).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));

        let n2 = Value::Number(DecimalNumber::parse("2.5").unwrap());
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_number_method(receiver2, "abs", vec![]).unwrap();
        assert_eq!(result2, Value::Number(DecimalNumber::parse("2.5").unwrap()));
    }

    #[test]
    fn test_number_ceil() {
        let n = Value::Number(DecimalNumber::parse("2.5").unwrap());
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(receiver, "ceil", vec![]).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(3)));

        let n2 = Value::Number(DecimalNumber::parse("-2.1").unwrap());
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_number_method(receiver2, "ceil", vec![]).unwrap();
        assert_eq!(result2, Value::Number(DecimalNumber::from_i64(-2)));
    }

    #[test]
    fn test_number_floor() {
        let n = Value::Number(DecimalNumber::parse("2.5").unwrap());
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(receiver, "floor", vec![]).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));

        let n2 = Value::Number(DecimalNumber::parse("-2.1").unwrap());
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_number_method(receiver2, "floor", vec![]).unwrap();
        assert_eq!(result2, Value::Number(DecimalNumber::from_i64(-3)));
    }

    #[test]
    fn test_number_round() {
        let n = Value::Number(DecimalNumber::parse("2.5").unwrap());
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(receiver, "round", vec![]).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(3)));

        let n2 = Value::Number(DecimalNumber::parse("3.6").unwrap());
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_number_method(receiver2, "round", vec![]).unwrap();
        assert_eq!(result2, Value::Number(DecimalNumber::from_i64(4)));
    }

    #[test]
    fn test_number_sqrt() {
        let n = Value::Number(DecimalNumber::from_i64(16));
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(receiver, "sqrt", vec![]).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(4)));

        let n2 = Value::Number(DecimalNumber::from_i64(9));
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_number_method(receiver2, "sqrt", vec![]).unwrap();
        assert_eq!(result2, Value::Number(DecimalNumber::from_i64(3)));
    }

    #[test]
    fn test_number_sqrt_negative() {
        let n = Value::Number(-DecimalNumber::from_i64(4));
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(receiver, "sqrt", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_number_pow() {
        let n = Value::Number(DecimalNumber::from_i64(2));
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(
            receiver,
            "pow",
            vec![Value::Number(DecimalNumber::from_i64(3))],
        )
        .unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(8)));

        let n2 = Value::Number(DecimalNumber::from_i64(5));
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_number_method(
            receiver2,
            "pow",
            vec![Value::Number(DecimalNumber::from_i64(2))],
        )
        .unwrap();
        assert_eq!(result2, Value::Number(DecimalNumber::from_i64(25)));
    }

    #[test]
    fn test_number_min() {
        let n = Value::Number(DecimalNumber::from_i64(10));
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(
            receiver,
            "min",
            vec![Value::Number(DecimalNumber::from_i64(20))],
        )
        .unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(10)));

        let n2 = Value::Number(DecimalNumber::from_i64(15));
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_number_method(
            receiver2,
            "min",
            vec![Value::Number(DecimalNumber::from_i64(5))],
        )
        .unwrap();
        assert_eq!(result2, Value::Number(DecimalNumber::from_i64(5)));
    }

    #[test]
    fn test_number_max() {
        let n = Value::Number(DecimalNumber::from_i64(10));
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(
            receiver,
            "max",
            vec![Value::Number(DecimalNumber::from_i64(20))],
        )
        .unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(20)));

        let n2 = Value::Number(DecimalNumber::from_i64(15));
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_number_method(
            receiver2,
            "max",
            vec![Value::Number(DecimalNumber::from_i64(5))],
        )
        .unwrap();
        assert_eq!(result2, Value::Number(DecimalNumber::from_i64(15)));
    }

    // Error case tests
    #[test]
    fn test_number_method_arity_errors() {
        let n = Value::Number(DecimalNumber::from_i64(42));

        // Test abs with arguments
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(
            receiver,
            "abs",
            vec![Value::Number(DecimalNumber::from_i64(1))],
        );
        assert!(result.is_err());

        // Test pow with wrong number of arguments
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(receiver, "pow", vec![]);
        assert!(result.is_err());

        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(
            receiver,
            "pow",
            vec![
                Value::Number(DecimalNumber::from_i64(1)),
                Value::Number(DecimalNumber::from_i64(2)),
            ],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_number_method_type_errors() {
        let n = Value::Number(DecimalNumber::from_i64(42));

        // Test pow with non-number argument
        let receiver = ValueRef::Immutable(&n);
        let result =
            call_number_method(receiver, "pow", vec![Value::String("invalid".to_string())]);
        assert!(result.is_err());

        // Test min with non-number argument
        let receiver = ValueRef::Immutable(&n);
        let result =
            call_number_method(receiver, "min", vec![Value::String("invalid".to_string())]);
        assert!(result.is_err());
    }
}
