use super::{
    EvalResult, eval_assignment, eval_expr, eval_pipe_apply_expression, eval_pipe_expression,
};
use crate::runtime::env::Env;
use crate::runtime::range::{expand_range_inclusive_values, expand_range_values};
use crate::runtime::value::{FunctionValue, ParamSpec, RuntimeError, Value};
use std::rc::Rc;
use suji_ast::Span;
use suji_ast::ast::{BinaryOp, CompoundOp, Expr, Literal, Stmt};

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
        BinaryOp::Pipe => eval_pipe_expression(left, right, env.clone()),
        BinaryOp::PipeApplyFwd | BinaryOp::PipeApplyBwd => {
            eval_pipe_apply_expression(left, op, right, env.clone())
        }
        BinaryOp::ComposeRight | BinaryOp::ComposeLeft => {
            eval_composition_expression(left, op, right, env.clone())
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
    if matches!(left, Value::Module(_)) || matches!(right, Value::Module(_)) {
        return Err(RuntimeError::InvalidOperation {
            message: "Cannot use module in binary operation.".to_string(),
        });
    }

    match op {
        // Pipe handled at higher level in eval_binary_expr
        BinaryOp::Pipe => unreachable!("Pipe op is evaluated in eval_binary_expr"),
        BinaryOp::PipeApplyFwd | BinaryOp::PipeApplyBwd => {
            unreachable!("Pipe apply ops are evaluated in eval_binary_expr")
        }
        BinaryOp::ComposeRight | BinaryOp::ComposeLeft => {
            unreachable!("Composition ops are evaluated in eval_binary_expr")
        }
        // Arithmetic operations
        BinaryOp::Add => match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.add(b))),
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
        BinaryOp::RangeInclusive => expand_range_inclusive_values(&left, &right),

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

/// Create an identifier expression with a provided span
fn make_identifier_expr_with_span(name: &str, span: Span) -> Expr {
    Expr::Literal(Literal::Identifier(name.to_string(), span))
}

/// Evaluate function composition operators (>> and <<)
pub fn eval_composition_expression(
    left: &Expr,
    op: &BinaryOp,
    right: &Expr,
    env: Rc<Env>,
) -> EvalResult<Value> {
    // Evaluate both sides and ensure they are functions
    let left_val = eval_expr(left, env.clone())?;
    let right_val = eval_expr(right, env.clone())?;

    let (left_func, right_func) = match (left_val, right_val) {
        (Value::Function(f), Value::Function(g)) => (f, g),
        (l, r) => {
            return Err(RuntimeError::TypeError {
                message: format!("Cannot compose {} and {}", l.type_name(), r.type_name()),
            });
        }
    };

    // Create a closure environment capturing both functions under internal names
    let composed_env = Rc::new(Env::new_child(env.clone()));
    composed_env.define_or_set("__f", Value::Function(left_func.clone()));
    composed_env.define_or_set("__g", Value::Function(right_func.clone()));

    // Build AST for composed function body with a single parameter `x`
    let param_name = "x".to_string();
    let left_span = left.covering_span();
    let right_span = right.covering_span();
    let x_expr = make_identifier_expr_with_span(&param_name, left_span.clone());

    let (inner_callee, outer_callee) = match op {
        BinaryOp::ComposeRight => ("__f", "__g"),
        BinaryOp::ComposeLeft => ("__g", "__f"),
        _ => unreachable!(),
    };

    let inner_callee_span = if inner_callee == "__f" {
        left_span.clone()
    } else {
        right_span.clone()
    };
    let inner_call = Expr::Call {
        callee: Box::new(make_identifier_expr_with_span(
            inner_callee,
            inner_callee_span.clone(),
        )),
        args: vec![x_expr],
        span: inner_callee_span,
    };

    let outer_callee_span = if outer_callee == "__f" {
        left_span.clone()
    } else {
        right_span.clone()
    };
    let outer_call = Expr::Call {
        callee: Box::new(make_identifier_expr_with_span(
            outer_callee,
            outer_callee_span.clone(),
        )),
        args: vec![inner_call],
        span: outer_callee_span,
    };

    let params = vec![ParamSpec {
        name: param_name,
        default: None,
    }];

    let body = Stmt::Expr(outer_call);

    Ok(Value::Function(FunctionValue {
        params,
        body,
        env: composed_env,
    }))
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
    let value_span = value.covering_span();
    let temp_expr = match &result_value {
        Value::Number(n) => Expr::Literal(Literal::Number(n.to_string(), value_span.clone())),
        Value::Boolean(b) => Expr::Literal(Literal::Boolean(*b, value_span.clone())),
        Value::String(s) => Expr::Literal(Literal::StringTemplate(
            vec![suji_ast::ast::StringPart::Text(s.clone())],
            value_span.clone(),
        )),
        Value::List(items) => {
            let mut exprs = Vec::new();
            for item in items {
                exprs.push(value_to_expr_with_span(item, value_span.clone()));
            }
            Expr::Literal(Literal::List(exprs, value_span.clone()))
        }
        Value::Map(map) => {
            let mut pairs = Vec::new();
            for (key, value_item) in map {
                pairs.push((
                    map_key_to_expr_with_span(key, value_span.clone()),
                    value_to_expr_with_span(value_item, value_span.clone()),
                ));
            }
            Expr::Literal(Literal::Map(pairs, value_span.clone()))
        }
        Value::Tuple(items) => {
            let mut exprs = Vec::new();
            for item in items {
                exprs.push(value_to_expr_with_span(item, value_span.clone()));
            }
            Expr::Literal(Literal::Tuple(exprs, value_span.clone()))
        }
        Value::Nil => Expr::Literal(Literal::Nil(value_span.clone())),
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
            if !n.is_integer() {
                return Err(RuntimeError::TypeError {
                    message: "List index must be an integer".to_string(),
                });
            }
            let idx = n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
                message: "List index out of range".to_string(),
            })?;
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
fn value_to_expr_with_span(value: &Value, span: Span) -> Expr {
    match value {
        Value::Number(n) => Expr::Literal(Literal::Number(n.to_string(), span.clone())),
        Value::Boolean(b) => Expr::Literal(Literal::Boolean(*b, span.clone())),
        Value::String(s) => Expr::Literal(Literal::StringTemplate(
            vec![suji_ast::ast::StringPart::Text(s.clone())],
            span.clone(),
        )),
        Value::List(items) => {
            let mut exprs = Vec::new();
            for item in items {
                exprs.push(value_to_expr_with_span(item, span.clone()));
            }
            Expr::Literal(Literal::List(exprs, span.clone()))
        }
        Value::Map(map) => {
            let mut pairs = Vec::new();
            for (key, value) in map {
                pairs.push((
                    map_key_to_expr_with_span(key, span.clone()),
                    value_to_expr_with_span(value, span.clone()),
                ));
            }
            Expr::Literal(Literal::Map(pairs, span.clone()))
        }
        Value::Tuple(items) => {
            let mut exprs = Vec::new();
            for item in items {
                exprs.push(value_to_expr_with_span(item, span.clone()));
            }
            Expr::Literal(Literal::Tuple(exprs, span.clone()))
        }
        Value::Nil => Expr::Literal(Literal::Nil(span.clone())),
        _ => Expr::Literal(Literal::Nil(span.clone())), // Fallback for complex types
    }
}

/// Helper function to convert a MapKey to an Expr
fn map_key_to_expr_with_span(key: &crate::runtime::value::MapKey, span: Span) -> Expr {
    match key {
        crate::runtime::value::MapKey::String(s) => Expr::Literal(Literal::StringTemplate(
            vec![suji_ast::ast::StringPart::Text(s.clone())],
            span.clone(),
        )),
        crate::runtime::value::MapKey::Number(n) => {
            Expr::Literal(Literal::Number(n.0.to_string(), span.clone()))
        }
        crate::runtime::value::MapKey::Boolean(b) => {
            Expr::Literal(Literal::Boolean(*b, span.clone()))
        }
        crate::runtime::value::MapKey::Tuple(items) => {
            let mut exprs = Vec::new();
            for item in items {
                exprs.push(value_to_expr_with_span(&item.to_value(), span.clone()));
            }
            Expr::Literal(Literal::Tuple(exprs, span.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::value::DecimalNumber;
    use suji_ast::ast::BinaryOp;

    #[test]
    fn test_arithmetic() {
        let result = eval_binary_op(
            &BinaryOp::Add,
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3)),
        )
        .unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));

        let result = eval_binary_op(
            &BinaryOp::Multiply,
            Value::Number(DecimalNumber::from_i64(4)),
            Value::Number(DecimalNumber::from_i64(5)),
        )
        .unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(20)));
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
    fn test_list_concatenation() {
        // Test basic list concatenation
        let list1 = Value::List(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
        ]);
        let list2 = Value::List(vec![
            Value::Number(DecimalNumber::from_i64(3)),
            Value::Number(DecimalNumber::from_i64(4)),
        ]);
        let result = eval_binary_op(&BinaryOp::Add, list1.clone(), list2.clone()).unwrap();

        let expected = Value::List(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3)),
            Value::Number(DecimalNumber::from_i64(4)),
        ]);
        assert_eq!(result, expected);

        // Test pass-by-value semantics (original lists unchanged)
        assert_eq!(
            list1,
            Value::List(vec![
                Value::Number(DecimalNumber::from_i64(1)),
                Value::Number(DecimalNumber::from_i64(2))
            ])
        );
        assert_eq!(
            list2,
            Value::List(vec![
                Value::Number(DecimalNumber::from_i64(3)),
                Value::Number(DecimalNumber::from_i64(4))
            ])
        );
    }

    #[test]
    fn test_comparisons() {
        let result = eval_binary_op(
            &BinaryOp::Less,
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3)),
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_binary_op(
            &BinaryOp::Equal,
            Value::Number(DecimalNumber::from_i64(5)),
            Value::Number(DecimalNumber::from_i64(5)),
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));
    }
}
