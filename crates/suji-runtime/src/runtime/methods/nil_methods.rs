use super::super::value::{RuntimeError, Value};
use super::common::ValueRef;

/// Nil methods: to_string()
pub fn call_nil_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::Nil => match method {
            "to_string" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_string() takes no arguments".to_string(),
                    });
                }
                Ok(Value::String("nil".to_string()))
            }
            _ => Err(RuntimeError::MethodError {
                message: format!("Nil has no method '{}'", method),
            }),
        },
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::ValueRef;
    use super::*;
    use crate::runtime::value::DecimalNumber;

    #[test]
    fn test_nil_to_string() {
        let nil_val = Value::Nil;
        let receiver = ValueRef::Immutable(&nil_val);
        let result = call_nil_method(receiver, "to_string", vec![]).unwrap();
        assert_eq!(result, Value::String("nil".to_string()));
    }

    #[test]
    fn test_nil_to_string_arity_mismatch() {
        let nil_val = Value::Nil;
        let receiver = ValueRef::Immutable(&nil_val);
        let result = call_nil_method(
            receiver,
            "to_string",
            vec![Value::Number(DecimalNumber::from_i64(1))],
        );
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_nil_invalid_method() {
        let nil_val = Value::Nil;
        let receiver = ValueRef::Immutable(&nil_val);
        let result = call_nil_method(receiver, "invalid_method", vec![]);
        assert!(matches!(result, Err(RuntimeError::MethodError { .. })));
    }
}
