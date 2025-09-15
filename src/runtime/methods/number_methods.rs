use super::super::value::{RuntimeError, Value};
use super::common::ValueRef;

/// Number methods: to_string(), is_int()
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
                Ok(Value::Boolean(n.fract() == 0.0))
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
    use super::super::common::ValueRef;
    use super::*;

    #[test]
    fn test_number_to_string() {
        let n = Value::Number(42.0);
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(receiver, "to_string", vec![]).unwrap();
        assert_eq!(result, Value::String("42".to_string()));

        let n2 = Value::Number(2.5);
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_number_method(receiver2, "to_string", vec![]).unwrap();
        assert_eq!(result2, Value::String("2.5".to_string()));
    }

    #[test]
    fn test_number_is_int() {
        let n = Value::Number(42.0);
        let receiver = ValueRef::Immutable(&n);
        let result = call_number_method(receiver, "is_int", vec![]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        let n2 = Value::Number(2.5);
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_number_method(receiver2, "is_int", vec![]).unwrap();
        assert_eq!(result2, Value::Boolean(false));
    }
}
