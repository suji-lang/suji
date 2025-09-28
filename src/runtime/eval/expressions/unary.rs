use super::EvalResult;
use crate::ast::UnaryOp;
use crate::runtime::value::{RuntimeError, Value};

/// Evaluate a unary operation
pub fn eval_unary_op(op: &UnaryOp, value: Value) -> EvalResult<Value> {
    match op {
        UnaryOp::Negate => match value {
            Value::Number(n) => Ok(Value::Number(-n)),
            _ => Err(RuntimeError::TypeError {
                message: format!("Cannot negate {}", value.type_name()),
            }),
        },
        UnaryOp::Not => match value {
            Value::Boolean(b) => Ok(Value::Boolean(!b)),
            _ => Err(RuntimeError::TypeError {
                message: format!("Cannot apply logical NOT to {}", value.type_name()),
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unary_operations() {
        let result = eval_unary_op(
            &UnaryOp::Negate,
            Value::Number(crate::runtime::value::DecimalNumber::from_i64(42)),
        )
        .unwrap();
        assert_eq!(
            result,
            Value::Number(crate::runtime::value::DecimalNumber::from_i64(-42))
        );

        let result = eval_unary_op(&UnaryOp::Not, Value::Boolean(true)).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }
}
