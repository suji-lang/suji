use crate::eval::EvalResult;
use suji_ast::BinaryOp;
use suji_values::{RuntimeError, Value};

/// Evaluate comparison operations (<, <=, >, >=).
///
/// Supports Number and String comparisons.
pub fn eval_comparison_op(op: BinaryOp, left: Value, right: Value) -> EvalResult<Value> {
    match op {
        BinaryOp::Less => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a < b)),
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot compare {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        BinaryOp::LessEqual => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a <= b)),
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot compare {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        BinaryOp::Greater => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a > b)),
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot compare {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        BinaryOp::GreaterEqual => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a >= b)),
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot compare {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        _ => Err(RuntimeError::TypeError {
            message: "Invalid comparison operator".to_string(),
        }),
    }
}

/// Evaluate arithmetic operations (-, *, /, %, ^).
///
/// Note: Add (+) is handled separately in eval_binary_op due to special cases
/// for String concatenation and List concatenation.
pub fn eval_arithmetic_op(op: BinaryOp, left: Value, right: Value) -> EvalResult<Value> {
    match op {
        BinaryOp::Subtract => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.sub(b))),
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot subtract {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        BinaryOp::Multiply => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.mul(b))),
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot multiply {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        BinaryOp::Divide => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => match a.div(b) {
                Ok(result) => Ok(Value::Number(result)),
                Err(err) => Err(RuntimeError::InvalidOperation {
                    message: err.to_string(),
                }),
            },
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot divide {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        BinaryOp::Modulo => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => match a.rem(b) {
                Ok(result) => Ok(Value::Number(result)),
                Err(err) => Err(RuntimeError::InvalidOperation {
                    message: err.to_string(),
                }),
            },
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot modulo {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        BinaryOp::Power => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => match a.pow(b) {
                Ok(result) => Ok(Value::Number(result)),
                Err(err) => Err(RuntimeError::InvalidOperation {
                    message: err.to_string(),
                }),
            },
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot exponentiate {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        _ => Err(RuntimeError::TypeError {
            message: "Invalid arithmetic operator".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_values::DecimalNumber;

    #[test]
    fn test_comparison_less() {
        let result = eval_comparison_op(
            BinaryOp::Less,
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3)),
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_comparison_op(
            BinaryOp::Less,
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_comparison_less_equal() {
        let result = eval_comparison_op(
            BinaryOp::LessEqual,
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(2)),
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_comparison_op(
            BinaryOp::LessEqual,
            Value::String("a".to_string()),
            Value::String("a".to_string()),
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_comparison_greater() {
        let result = eval_comparison_op(
            BinaryOp::Greater,
            Value::Number(DecimalNumber::from_i64(5)),
            Value::Number(DecimalNumber::from_i64(3)),
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_comparison_op(
            BinaryOp::Greater,
            Value::String("z".to_string()),
            Value::String("a".to_string()),
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_comparison_greater_equal() {
        let result = eval_comparison_op(
            BinaryOp::GreaterEqual,
            Value::Number(DecimalNumber::from_i64(5)),
            Value::Number(DecimalNumber::from_i64(5)),
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_comparison_op(
            BinaryOp::GreaterEqual,
            Value::String("z".to_string()),
            Value::String("z".to_string()),
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_comparison_type_error() {
        let result = eval_comparison_op(
            BinaryOp::Less,
            Value::Number(DecimalNumber::from_i64(2)),
            Value::String("3".to_string()),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().without_span(),
            RuntimeError::TypeError { .. }
        ));
    }

    #[test]
    fn test_arithmetic_subtract() {
        let result = eval_arithmetic_op(
            BinaryOp::Subtract,
            Value::Number(DecimalNumber::from_i64(10)),
            Value::Number(DecimalNumber::from_i64(3)),
        )
        .unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(7)));
    }

    #[test]
    fn test_arithmetic_multiply() {
        let result = eval_arithmetic_op(
            BinaryOp::Multiply,
            Value::Number(DecimalNumber::from_i64(4)),
            Value::Number(DecimalNumber::from_i64(5)),
        )
        .unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(20)));
    }

    #[test]
    fn test_arithmetic_divide() {
        let result = eval_arithmetic_op(
            BinaryOp::Divide,
            Value::Number(DecimalNumber::from_i64(20)),
            Value::Number(DecimalNumber::from_i64(4)),
        )
        .unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));
    }

    #[test]
    fn test_arithmetic_modulo() {
        let result = eval_arithmetic_op(
            BinaryOp::Modulo,
            Value::Number(DecimalNumber::from_i64(10)),
            Value::Number(DecimalNumber::from_i64(3)),
        )
        .unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));
    }

    #[test]
    fn test_arithmetic_power() {
        let result = eval_arithmetic_op(
            BinaryOp::Power,
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3)),
        )
        .unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(8)));
    }

    #[test]
    fn test_arithmetic_type_error() {
        let result = eval_arithmetic_op(
            BinaryOp::Subtract,
            Value::Number(DecimalNumber::from_i64(2)),
            Value::String("3".to_string()),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().without_span(),
            RuntimeError::TypeError { .. }
        ));
    }
}
