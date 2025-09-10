use super::{EvalResult, eval_assignment, eval_expr};
use crate::ast::{BinaryOp, CompoundOp, Expr, Literal, UnaryOp};
use crate::runtime::env::Env;
use crate::runtime::range::expand_range_values;
use crate::runtime::value::{RuntimeError, Value};
use crate::token::Span;
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
            (Value::List(a), Value::List(b)) => {
                // List concatenation: pass-by-value semantics
                let mut result = a.clone();
                result.extend(b.clone());
                Ok(Value::List(result))
            }
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

/// Evaluate a compound assignment expression (target += value, target -= value, etc.)
pub fn eval_compound_assignment(
    target: &Expr,
    op: CompoundOp,
    value: &Expr,
    env: Rc<Env>,
) -> EvalResult<Value> {
    // Evaluate the right-hand side
    let rhs_value = eval_expr(value, env.clone())?;

    // Get the current value of the target
    let current_value = match target {
        Expr::Literal(Literal::Identifier(name, _)) => env.get(name)?,
        Expr::Index { target, index, .. } => {
            // For indexed targets, we need to evaluate the index first
            let target_value = eval_expr(target, env.clone())?;
            let index_value = eval_expr(index, env.clone())?;
            eval_index_value(&target_value, &index_value)?
        }
        Expr::MapAccessByName { target, key, .. } => {
            // For map access targets, we need to evaluate the target first
            let target_value = eval_expr(target, env.clone())?;
            eval_map_access_value(&target_value, key)?
        }
        _ => {
            return Err(RuntimeError::InvalidOperation {
                message: "Invalid compound assignment target".to_string(),
            });
        }
    };

    // Apply the compound operation
    let result_value = match op {
        CompoundOp::PlusAssign => eval_binary_op(&BinaryOp::Add, current_value, rhs_value)?,
        CompoundOp::MinusAssign => eval_binary_op(&BinaryOp::Subtract, current_value, rhs_value)?,
        CompoundOp::MultiplyAssign => {
            eval_binary_op(&BinaryOp::Multiply, current_value, rhs_value)?
        }
        CompoundOp::DivideAssign => eval_binary_op(&BinaryOp::Divide, current_value, rhs_value)?,
        CompoundOp::ModuloAssign => eval_binary_op(&BinaryOp::Modulo, current_value, rhs_value)?,
    };

    // Assign the result back to the target
    // We need to create a temporary expression with the result value
    let temp_expr = match &result_value {
        Value::Number(n) => Expr::Literal(Literal::Number(*n, Span::default())),
        Value::Boolean(b) => Expr::Literal(Literal::Boolean(*b, Span::default())),
        Value::String(s) => Expr::Literal(Literal::StringTemplate(
            vec![crate::ast::StringPart::Text(s.clone())],
            Span::default(),
        )),
        Value::List(items) => {
            let mut exprs = Vec::new();
            for item in items {
                exprs.push(value_to_expr(item));
            }
            Expr::Literal(Literal::List(exprs, Span::default()))
        }
        Value::Map(map) => {
            let mut pairs = Vec::new();
            for (key, value) in map {
                pairs.push((map_key_to_expr(key), value_to_expr(value)));
            }
            Expr::Literal(Literal::Map(pairs, Span::default()))
        }
        Value::Tuple(items) => {
            let mut exprs = Vec::new();
            for item in items {
                exprs.push(value_to_expr(item));
            }
            Expr::Literal(Literal::Tuple(exprs, Span::default()))
        }
        Value::Nil => Expr::Literal(Literal::Nil(Span::default())),
        _ => {
            return Err(RuntimeError::InvalidOperation {
                message: "Cannot assign complex value type".to_string(),
            });
        }
    };

    eval_assignment(target, &temp_expr, env)?;

    Ok(result_value)
}

/// Helper function to evaluate index access on a value
fn eval_index_value(target: &Value, index: &Value) -> EvalResult<Value> {
    match (target, index) {
        (Value::List(items), Value::Number(n)) => {
            if n.fract() != 0.0 {
                return Err(RuntimeError::TypeError {
                    message: "List index must be an integer".to_string(),
                });
            }
            let idx = *n as i64;
            let normalized_idx = if idx < 0 {
                items.len() as i64 + idx
            } else {
                idx
            };

            if normalized_idx < 0 || normalized_idx >= items.len() as i64 {
                return Err(RuntimeError::IndexOutOfBounds {
                    message: format!(
                        "Index {} out of bounds for list of length {}",
                        idx,
                        items.len()
                    ),
                });
            }

            Ok(items[normalized_idx as usize].clone())
        }
        (Value::Map(map), key) => {
            let map_key = key.clone().try_into_map_key()?;
            map.get(&map_key)
                .cloned()
                .ok_or_else(|| RuntimeError::IndexOutOfBounds {
                    message: "Key not found in map".to_string(),
                })
        }
        _ => Err(RuntimeError::TypeError {
            message: format!(
                "Cannot index {} with {}",
                target.type_name(),
                index.type_name()
            ),
        }),
    }
}

/// Helper function to evaluate map access on a value
fn eval_map_access_value(target: &Value, key: &str) -> EvalResult<Value> {
    match target {
        Value::Map(map) => {
            let map_key = crate::runtime::value::MapKey::String(key.to_string());
            map.get(&map_key)
                .cloned()
                .ok_or_else(|| RuntimeError::IndexOutOfBounds {
                    message: format!("Key '{}' not found in map", key),
                })
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot access key '{}' on {}", key, target.type_name()),
        }),
    }
}

/// Helper function to convert a Value to an Expr
fn value_to_expr(value: &Value) -> Expr {
    match value {
        Value::Number(n) => Expr::Literal(Literal::Number(*n, Span::default())),
        Value::Boolean(b) => Expr::Literal(Literal::Boolean(*b, Span::default())),
        Value::String(s) => Expr::Literal(Literal::StringTemplate(
            vec![crate::ast::StringPart::Text(s.clone())],
            Span::default(),
        )),
        Value::List(items) => {
            let mut exprs = Vec::new();
            for item in items {
                exprs.push(value_to_expr(item));
            }
            Expr::Literal(Literal::List(exprs, Span::default()))
        }
        Value::Map(map) => {
            let mut pairs = Vec::new();
            for (key, value) in map {
                pairs.push((map_key_to_expr(key), value_to_expr(value)));
            }
            Expr::Literal(Literal::Map(pairs, Span::default()))
        }
        Value::Tuple(items) => {
            let mut exprs = Vec::new();
            for item in items {
                exprs.push(value_to_expr(item));
            }
            Expr::Literal(Literal::Tuple(exprs, Span::default()))
        }
        Value::Nil => Expr::Literal(Literal::Nil(Span::default())),
        _ => Expr::Literal(Literal::Nil(Span::default())), // Fallback for complex types
    }
}

/// Helper function to convert a MapKey to an Expr
fn map_key_to_expr(key: &crate::runtime::value::MapKey) -> Expr {
    match key {
        crate::runtime::value::MapKey::String(s) => Expr::Literal(Literal::StringTemplate(
            vec![crate::ast::StringPart::Text(s.clone())],
            Span::default(),
        )),
        crate::runtime::value::MapKey::Number(n) => {
            Expr::Literal(Literal::Number(n.0, Span::default()))
        }
        crate::runtime::value::MapKey::Boolean(b) => {
            Expr::Literal(Literal::Boolean(*b, Span::default()))
        }
        crate::runtime::value::MapKey::Tuple(items) => {
            let mut exprs = Vec::new();
            for item in items {
                exprs.push(value_to_expr(&item.to_value()));
            }
            Expr::Literal(Literal::Tuple(exprs, Span::default()))
        }
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

    #[test]
    fn test_compound_assignment() {
        use crate::ast::{Expr, Literal};
        use crate::runtime::env::Env;
        use crate::token::Span;
        use std::rc::Rc;

        let env = Rc::new(Env::new());
        env.define_or_set("x", Value::Number(10.0));

        // Test x += 5
        let target = Expr::Literal(Literal::Identifier("x".to_string(), Span::default()));
        let value = Expr::Literal(Literal::Number(5.0, Span::default()));
        let result =
            eval_compound_assignment(&target, CompoundOp::PlusAssign, &value, env.clone()).unwrap();

        assert_eq!(result, Value::Number(15.0));
        assert_eq!(env.get("x").unwrap(), Value::Number(15.0));

        // Test x -= 3
        let result =
            eval_compound_assignment(&target, CompoundOp::MinusAssign, &value, env.clone())
                .unwrap();

        assert_eq!(result, Value::Number(10.0));
        assert_eq!(env.get("x").unwrap(), Value::Number(10.0));

        // Test x *= 2
        let multiply_value = Expr::Literal(Literal::Number(2.0, Span::default()));
        let result = eval_compound_assignment(
            &target,
            CompoundOp::MultiplyAssign,
            &multiply_value,
            env.clone(),
        )
        .unwrap();

        assert_eq!(result, Value::Number(20.0));
        assert_eq!(env.get("x").unwrap(), Value::Number(20.0));

        // Test x /= 4
        let divide_value = Expr::Literal(Literal::Number(4.0, Span::default()));
        let result = eval_compound_assignment(
            &target,
            CompoundOp::DivideAssign,
            &divide_value,
            env.clone(),
        )
        .unwrap();

        assert_eq!(result, Value::Number(5.0));
        assert_eq!(env.get("x").unwrap(), Value::Number(5.0));

        // Test x %= 3
        let modulo_value = Expr::Literal(Literal::Number(3.0, Span::default()));
        let result = eval_compound_assignment(
            &target,
            CompoundOp::ModuloAssign,
            &modulo_value,
            env.clone(),
        )
        .unwrap();

        assert_eq!(result, Value::Number(2.0));
        assert_eq!(env.get("x").unwrap(), Value::Number(2.0));
    }

    #[test]
    fn test_list_concatenation() {
        // Test basic list concatenation
        let list1 = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let list2 = Value::List(vec![Value::Number(3.0), Value::Number(4.0)]);
        let result = eval_binary_op(&BinaryOp::Add, list1.clone(), list2.clone()).unwrap();

        let expected = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
        assert_eq!(result, expected);

        // Test empty list concatenation
        let empty = Value::List(vec![]);
        let numbers = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);

        let result1 = eval_binary_op(&BinaryOp::Add, empty.clone(), numbers.clone()).unwrap();
        assert_eq!(result1, numbers);

        let result2 = eval_binary_op(&BinaryOp::Add, numbers.clone(), empty.clone()).unwrap();
        assert_eq!(result2, numbers);

        // Test both empty lists
        let result3 = eval_binary_op(&BinaryOp::Add, empty.clone(), empty.clone()).unwrap();
        assert_eq!(result3, empty);

        // Test mixed types in lists
        let mixed1 = Value::List(vec![
            Value::Number(1.0),
            Value::String("hello".to_string()),
            Value::Boolean(true),
        ]);
        let mixed2 = Value::List(vec![
            Value::Number(2.5),
            Value::String("world".to_string()),
            Value::Boolean(false),
        ]);
        let result = eval_binary_op(&BinaryOp::Add, mixed1.clone(), mixed2.clone()).unwrap();

        let expected = Value::List(vec![
            Value::Number(1.0),
            Value::String("hello".to_string()),
            Value::Boolean(true),
            Value::Number(2.5),
            Value::String("world".to_string()),
            Value::Boolean(false),
        ]);
        assert_eq!(result, expected);

        // Test pass-by-value semantics (original lists unchanged)
        assert_eq!(
            list1,
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
        );
        assert_eq!(
            list2,
            Value::List(vec![Value::Number(3.0), Value::Number(4.0)])
        );
        assert_eq!(
            mixed1,
            Value::List(vec![
                Value::Number(1.0),
                Value::String("hello".to_string()),
                Value::Boolean(true)
            ])
        );
        assert_eq!(
            mixed2,
            Value::List(vec![
                Value::Number(2.5),
                Value::String("world".to_string()),
                Value::Boolean(false)
            ])
        );
    }

    #[test]
    fn test_list_concatenation_errors() {
        // Test list + number (should fail)
        let list = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let number = Value::Number(3.0);
        let result = eval_binary_op(&BinaryOp::Add, list, number);
        assert!(result.is_err());
        if let Err(RuntimeError::TypeError { message }) = result {
            assert!(message.contains("Cannot add list and number"));
        }

        // Test list + string (should fail)
        let list = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let string = Value::String("hello".to_string());
        let result = eval_binary_op(&BinaryOp::Add, list, string);
        assert!(result.is_err());
        if let Err(RuntimeError::TypeError { message }) = result {
            assert!(message.contains("Cannot add list and string"));
        }

        // Test list + map (should fail)
        let list = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let map = Value::Map(indexmap::IndexMap::new());
        let result = eval_binary_op(&BinaryOp::Add, list, map);
        assert!(result.is_err());
        if let Err(RuntimeError::TypeError { message }) = result {
            assert!(message.contains("Cannot add list and map"));
        }
    }
}
