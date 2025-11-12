use super::pipe::eval_pipe_apply_expression;
use super::{EvalResult, eval_expr};
use std::rc::Rc;
use suji_ast::Span;
use suji_ast::ast::{BinaryOp, Expr, Literal, Stmt};
use suji_runtime::ModuleRegistry;
use suji_values::Env;
use suji_values::{FunctionBody, FunctionValue, ParamSpec, RuntimeError, Value};
use suji_values::{expand_range_inclusive_values, expand_range_values};

/// Evaluate a binary expression with short-circuiting for logical ops
pub fn eval_binary_expr(
    left: &Expr,
    op: &BinaryOp,
    right: &Expr,
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Value> {
    // Short-circuit evaluation for logical operators
    match op {
        BinaryOp::And => {
            let left_val = eval_expr(left, env.clone(), registry)?;
            if !left_val.is_truthy() {
                Ok(Value::Boolean(false))
            } else {
                let right_val = eval_expr(right, env, registry)?;
                match right_val {
                    Value::Boolean(b) => Ok(Value::Boolean(b)),
                    _ => Err(RuntimeError::TypeError {
                        message: "Logical AND requires boolean operands".to_string(),
                    }),
                }
            }
        }
        BinaryOp::Or => {
            let left_val = eval_expr(left, env.clone(), registry)?;
            if left_val.is_truthy() {
                Ok(Value::Boolean(true))
            } else {
                let right_val = eval_expr(right, env, registry)?;
                match right_val {
                    Value::Boolean(b) => Ok(Value::Boolean(b)),
                    _ => Err(RuntimeError::TypeError {
                        message: "Logical OR requires boolean operands".to_string(),
                    }),
                }
            }
        }
        BinaryOp::Pipe => super::pipe::eval_pipe_expression(left, right, env.clone(), registry),
        BinaryOp::PipeApplyFwd | BinaryOp::PipeApplyBwd => {
            eval_pipe_apply_expression(left, op, right, env.clone(), registry)
        }
        BinaryOp::ComposeRight | BinaryOp::ComposeLeft => {
            eval_composition_expression(left, op, right, env.clone(), registry)
        }
        _ => {
            // Evaluate both sides for other operations
            let left_val = eval_expr(left, env.clone(), registry)?;
            let right_val = eval_expr(right, env, registry)?;
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
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Value> {
    // Evaluate both sides and ensure they are functions
    let left_val = eval_expr(left, env.clone(), registry)?;
    let right_val = eval_expr(right, env.clone(), registry)?;

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
        body: FunctionBody::Ast(body),
        env: composed_env,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_ast::ast::BinaryOp;
    use suji_values::DecimalNumber;

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
