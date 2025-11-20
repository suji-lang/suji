// No executor needed
use super::super::value::{RuntimeError, Value};
use super::common::{ValueRef, call_type_checking_method};

/// Boolean methods: to_string()
pub fn call_boolean_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::Boolean(b) => match method {
            "to_string" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_string() takes no arguments".to_string(),
                    });
                }
                Ok(Value::String(b.to_string()))
            }
            "is_number" | "is_bool" | "is_string" | "is_list" | "is_map" | "is_stream"
            | "is_function" | "is_tuple" | "is_regex" => {
                call_type_checking_method(method, receiver.get(), args)
            }
            _ => Err(RuntimeError::MethodError {
                message: format!("Boolean has no method '{}'", method),
            }),
        },
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::ValueRef;
    use super::*;
    use crate::value::DecimalNumber;

    #[test]
    fn test_boolean_to_string() {
        let bool_true = Value::Boolean(true);
        let receiver = ValueRef::Immutable(&bool_true);
        let result = call_boolean_method(receiver, "to_string", vec![]).unwrap();
        assert_eq!(result, Value::String("true".to_string()));

        let bool_false = Value::Boolean(false);
        let receiver2 = ValueRef::Immutable(&bool_false);
        let result2 = call_boolean_method(receiver2, "to_string", vec![]).unwrap();
        assert_eq!(result2, Value::String("false".to_string()));
    }

    #[test]
    fn test_boolean_to_string_arity_mismatch() {
        let bool_val = Value::Boolean(true);
        let receiver = ValueRef::Immutable(&bool_val);
        let result = call_boolean_method(
            receiver,
            "to_string",
            vec![Value::Number(DecimalNumber::from_i64(1))],
        );
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_boolean_invalid_method() {
        let bool_val = Value::Boolean(true);
        let receiver = ValueRef::Immutable(&bool_val);
        let result = call_boolean_method(receiver, "invalid_method", vec![]);
        assert!(matches!(result, Err(RuntimeError::MethodError { .. })));
    }
}
