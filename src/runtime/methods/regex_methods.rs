use super::super::value::{RuntimeError, Value};
use super::common::ValueRef;

/// Regex methods: to_string()
pub fn call_regex_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::Regex(regex) => match method {
            "to_string" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_string() takes no arguments".to_string(),
                    });
                }
                Ok(Value::String(format!("/{}/", regex.as_str())))
            }
            _ => Err(RuntimeError::MethodError {
                message: format!("Regex has no method '{}'", method),
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
    use regex::Regex;

    #[test]
    fn test_regex_to_string() {
        let regex = Regex::new("test").unwrap();
        let regex_val = Value::Regex(regex);
        let receiver = ValueRef::Immutable(&regex_val);
        let result = call_regex_method(receiver, "to_string", vec![]).unwrap();
        assert_eq!(result, Value::String("/test/".to_string()));
    }

    #[test]
    fn test_regex_to_string_complex_pattern() {
        let regex = Regex::new(r"\d+\.\d+").unwrap();
        let regex_val = Value::Regex(regex);
        let receiver = ValueRef::Immutable(&regex_val);
        let result = call_regex_method(receiver, "to_string", vec![]).unwrap();
        assert_eq!(result, Value::String(r"/\d+\.\d+/".to_string()));
    }

    #[test]
    fn test_regex_to_string_arity_mismatch() {
        let regex = Regex::new("test").unwrap();
        let regex_val = Value::Regex(regex);
        let receiver = ValueRef::Immutable(&regex_val);
        let result = call_regex_method(
            receiver,
            "to_string",
            vec![Value::Number(DecimalNumber::from_i64(1))],
        );
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_regex_invalid_method() {
        let regex = Regex::new("test").unwrap();
        let regex_val = Value::Regex(regex);
        let receiver = ValueRef::Immutable(&regex_val);
        let result = call_regex_method(receiver, "invalid_method", vec![]);
        assert!(matches!(result, Err(RuntimeError::MethodError { .. })));
    }
}
