use super::{EvalResult, eval_assignment, eval_expr};
use crate::ast::{BinaryOp, CompoundOp, Expr, Literal};
use crate::runtime::env::Env;
use crate::runtime::module::ModuleRegistry;
use crate::runtime::range::expand_range_values;
use crate::runtime::value::{FunctionValue, RuntimeError, Value};
use crate::token::Span;
use std::rc::Rc;

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
        BinaryOp::Pipe => {
            // Evaluate and execute a pipeline of closures left-to-right
            let mut stages: Vec<FunctionValue> = Vec::new();

            // Helper to collect function stages from an expression
            fn collect_stage(
                expr: &Expr,
                env: Rc<Env>,
                out: &mut Vec<FunctionValue>,
            ) -> Result<(), RuntimeError> {
                match expr {
                    Expr::Binary {
                        left,
                        op: BinaryOp::Pipe,
                        right,
                        ..
                    } => {
                        collect_stage(left, env.clone(), out)?;
                        collect_stage(right, env, out)?;
                        Ok(())
                    }
                    _ => {
                        let v = eval_expr(expr, env)?;
                        match v {
                            Value::Function(f) => {
                                out.push(f);
                                Ok(())
                            }
                            _ => Err(RuntimeError::TypeError {
                                message: "Pipe operator requires function values on both sides"
                                    .to_string(),
                            }),
                        }
                    }
                }
            }

            collect_stage(left, env.clone(), &mut stages)?;
            collect_stage(right, env.clone(), &mut stages)?;

            if stages.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: "Empty pipe expression".to_string(),
                });
            }

            // Base std module
            let base_std = match stages.first() {
                Some(f) => f.env.get("std").unwrap_or(Value::Nil),
                None => Value::Nil,
            };

            // Helper to extract std map or create fresh
            fn get_std_map(
                value: Value,
            ) -> indexmap::IndexMap<crate::runtime::value::MapKey, Value> {
                if let Value::Map(m) = value {
                    m
                } else if let Value::Map(m) = crate::runtime::builtins::create_std_module() {
                    m
                } else {
                    indexmap::IndexMap::new()
                }
            }

            let mut current_input: Option<Vec<u8>> = None;
            let last_index = stages.len() - 1;
            let mut last_result: Option<Value> = None;

            for (i, func) in stages.into_iter().enumerate() {
                // Start from a std map snapshot
                let mut std_map = get_std_map(base_std.clone());

                // Get io submodule map
                let io_key = crate::runtime::value::MapKey::String("io".to_string());
                let mut io_map = match std_map.get(&io_key) {
                    Some(Value::Map(m)) => m.clone(),
                    _ => match crate::runtime::builtins::create_std_module() {
                        Value::Map(std_m) => match std_m.get(&io_key) {
                            Some(Value::Map(m)) => m.clone(),
                            _ => indexmap::IndexMap::new(),
                        },
                        _ => indexmap::IndexMap::new(),
                    },
                };

                // Replace stdin if there is input
                if let Some(bytes) = current_input.take() {
                    let stdin_key = crate::runtime::value::MapKey::String("stdin".to_string());
                    let stdin_stream = Rc::new(
                        crate::runtime::value::StreamHandle::new_memory_readable(bytes),
                    );
                    io_map.insert(stdin_key, Value::Stream(stdin_stream));
                }

                // Replace stdout for non-last stages to capture output
                let mut capture_stdout = None;
                if i != last_index {
                    let stdout_key = crate::runtime::value::MapKey::String("stdout".to_string());
                    let stdout_handle =
                        Rc::new(crate::runtime::value::StreamHandle::new_memory_writable());
                    capture_stdout = Some(stdout_handle.clone());
                    io_map.insert(stdout_key, Value::Stream(stdout_handle));
                }

                // Put io back into std (keep a clone for env override)
                let io_map_clone = io_map.clone();
                std_map.insert(io_key.clone(), Value::Map(io_map));

                // Build overridden std value and module registry
                let overridden_std = Value::Map(std_map.clone());
                let registry = ModuleRegistry::new().with_custom_std(overridden_std);

                // Also inject overridden std and io directly into the call environment to ensure
                // both module-path imports and pre-bound names resolve to these instances
                let env_overrides = Some(vec![
                    ("std".to_string(), Value::Map(std_map)),
                    ("io".to_string(), Value::Map(io_map_clone)),
                ]);

                // Call function with no args; allow default params
                match crate::runtime::eval::call_function_with_modules(
                    &func,
                    Vec::new(),
                    Some(env.clone()),
                    &registry,
                    env_overrides,
                ) {
                    Ok(val) => {
                        last_result = Some(val);
                        // If not last stage, capture stdout bytes for next stage
                        if let Some(stdout_handle) = capture_stdout {
                            current_input = stdout_handle.take_memory_output();
                        }
                    }
                    Err(e) => {
                        return Err(RuntimeError::InvalidOperation {
                            message: format!("Failed to execute closure in pipe: {}", e),
                        });
                    }
                }
            }

            Ok(last_result.unwrap_or(Value::Nil))
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
        // Pipe handled at higher level in eval_binary_expr
        BinaryOp::Pipe => unreachable!("Pipe op is evaluated in eval_binary_expr"),
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
    use crate::ast::BinaryOp;

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

        // Test pass-by-value semantics (original lists unchanged)
        assert_eq!(
            list1,
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
        );
        assert_eq!(
            list2,
            Value::List(vec![Value::Number(3.0), Value::Number(4.0)])
        );
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
