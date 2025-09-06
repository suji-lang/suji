use super::{EvalResult, eval_expr};
use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::runtime::env::Env;
use crate::runtime::range::expand_range_values;
use crate::runtime::value::{RuntimeError, Value};
use std::rc::Rc;

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

/// Evaluate a binary expression with short-circuiting for logical ops
pub fn eval_binary_expr(
    left: &Expr,
    op: &BinaryOp,
    right: &Expr,
    env: Rc<Env>,
) -> EvalResult<Value> {
    // Short-circuit evaluation for logical operators
    match op {
        BinaryOp::And => {
            let left_val = eval_expr(left, env.clone())?;
            if !left_val.is_truthy() {
                Ok(Value::Boolean(false))
            } else {
                let right_val = eval_expr(right, env)?;
                match right_val {
                    Value::Boolean(b) => Ok(Value::Boolean(b)),
                    _ => Err(RuntimeError::TypeError {
                        message: "Logical AND requires boolean operands".to_string(),
                    }),
                }
            }
        }
        BinaryOp::Or => {
            let left_val = eval_expr(left, env.clone())?;
            if left_val.is_truthy() {
                Ok(Value::Boolean(true))
            } else {
                let right_val = eval_expr(right, env)?;
                match right_val {
                    Value::Boolean(b) => Ok(Value::Boolean(b)),
                    _ => Err(RuntimeError::TypeError {
                        message: "Logical OR requires boolean operands".to_string(),
                    }),
                }
            }
        }
        _ => {
            // Evaluate both sides for other operations
            let left_val = eval_expr(left, env.clone())?;
            let right_val = eval_expr(right, env)?;
            eval_binary_op(op, left_val, right_val)
        }
    }
}

/// Evaluate a binary operation on two values
pub fn eval_binary_op(op: &BinaryOp, left: Value, right: Value) -> EvalResult<Value> {
    match op {
        // Arithmetic operations
        BinaryOp::Add => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(RuntimeError::TypeError {
                message: format!("Cannot add {} and {}", left.type_name(), right.type_name()),
            }),
        },
        BinaryOp::Subtract => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot subtract {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        BinaryOp::Multiply => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot multiply {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        BinaryOp::Divide => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::InvalidOperation {
                        message: "Division by zero".to_string(),
                    })
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot divide {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        BinaryOp::Modulo => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::InvalidOperation {
                        message: "Modulo by zero".to_string(),
                    })
                } else {
                    Ok(Value::Number(a % b))
                }
            }
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot modulo {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },
        BinaryOp::Power => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.powf(*b))),
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot exponentiate {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        },

        // Comparison operations
        BinaryOp::Equal => Ok(Value::Boolean(left == right)),
        BinaryOp::NotEqual => Ok(Value::Boolean(left != right)),
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

        // Range operation
        BinaryOp::Range => expand_range_values(&left, &right),

        // Regex matching
        BinaryOp::RegexMatch => match (&left, &right) {
            (Value::String(s), Value::Regex(regex)) => Ok(Value::Boolean(regex.is_match(s))),
            _ => Err(RuntimeError::TypeError {
                message: "Regex match requires string ~ regex".to_string(),
            }),
        },
        BinaryOp::RegexNotMatch => match (&left, &right) {
            (Value::String(s), Value::Regex(regex)) => Ok(Value::Boolean(!regex.is_match(s))),
            _ => Err(RuntimeError::TypeError {
                message: "Regex not-match requires string !~ regex".to_string(),
            }),
        },

        // Logical operations (should be handled by eval_binary_expr)
        BinaryOp::And | BinaryOp::Or => unreachable!("Logical ops should be short-circuited"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, UnaryOp};

    #[test]
    fn test_arithmetic() {
        let result =
            eval_binary_op(&BinaryOp::Add, Value::Number(2.0), Value::Number(3.0)).unwrap();
        assert_eq!(result, Value::Number(5.0));

        let result =
            eval_binary_op(&BinaryOp::Multiply, Value::Number(4.0), Value::Number(5.0)).unwrap();
        assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_string_concatenation() {
        let result = eval_binary_op(
            &BinaryOp::Add,
            Value::String("hello".to_string()),
            Value::String(" world".to_string()),
        )
        .unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_unary_operations() {
        let result = eval_unary_op(&UnaryOp::Negate, Value::Number(42.0)).unwrap();
        assert_eq!(result, Value::Number(-42.0));

        let result = eval_unary_op(&UnaryOp::Not, Value::Boolean(true)).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_comparisons() {
        let result =
            eval_binary_op(&BinaryOp::Less, Value::Number(2.0), Value::Number(3.0)).unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result =
            eval_binary_op(&BinaryOp::Equal, Value::Number(5.0), Value::Number(5.0)).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }
}
