use super::{EvalResult, eval_expr, eval_stmt, should_break_loop, should_continue_loop};
use crate::ast::{Expr, LoopBindings, MatchArm, Stmt};
use crate::runtime::env::Env;
use crate::runtime::value::{ControlFlow, RuntimeError, Value};
use std::rc::Rc;

/// Evaluate a block statement
pub fn eval_block(
    statements: &[Stmt],
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
) -> EvalResult<Option<Value>> {
    // Create new scope for block
    let block_env = Rc::new(Env::new_child(env));

    let mut last_value = None;
    for stmt in statements {
        match eval_stmt(stmt, block_env.clone(), loop_stack) {
            Ok(value) => last_value = value,
            Err(control_flow_error) => return Err(control_flow_error), // Propagate control flow
        }
    }

    Ok(last_value)
}

/// Evaluate an infinite loop
pub fn eval_infinite_loop(
    label: Option<&str>,
    body: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
) -> EvalResult<Option<Value>> {
    // Push label onto loop stack
    if let Some(label_name) = label {
        loop_stack.push(label_name.to_string());
    }

    let result = loop {
        match eval_stmt(body, env.clone(), loop_stack) {
            Ok(_) => {
                // Continue looping
            }
            Err(RuntimeError::ControlFlow {
                flow: ControlFlow::Break(break_label),
            }) => {
                match &break_label {
                    Some(target_label) => {
                        if label.map(|l| l == target_label).unwrap_or(false) {
                            // This break targets our loop
                            break Ok(Some(Value::Null));
                        } else {
                            // Break targets outer loop, propagate
                            break Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Break(break_label),
                            });
                        }
                    }
                    None => {
                        // Unlabeled break targets innermost loop (us)
                        break Ok(Some(Value::Null));
                    }
                }
            }
            Err(RuntimeError::ControlFlow {
                flow: ControlFlow::Continue(continue_label),
            }) => {
                match &continue_label {
                    Some(target_label) => {
                        if label.map(|l| l == target_label).unwrap_or(false) {
                            // This continue targets our loop - restart
                            continue;
                        } else {
                            // Continue targets outer loop, propagate
                            break Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Continue(continue_label),
                            });
                        }
                    }
                    None => {
                        // Unlabeled continue targets innermost loop (us)
                        continue;
                    }
                }
            }
            Err(other_error) => {
                break Err(other_error);
            }
        }
    };

    // Pop label from loop stack
    if label.is_some() {
        loop_stack.pop();
    }

    result
}

/// Evaluate a loop-through statement
pub fn eval_loop_through(
    label: Option<&str>,
    iterable: &Expr,
    bindings: &LoopBindings,
    body: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
) -> EvalResult<Option<Value>> {
    let iterable_value = eval_expr(iterable, env.clone())?;

    // Push label onto loop stack
    if let Some(label_name) = label {
        loop_stack.push(label_name.to_string());
    }

    let result = match iterable_value {
        Value::List(items) => {
            match bindings {
                LoopBindings::None => {
                    // loop through list { ... } - no bindings
                    let mut iter_result = Ok(Some(Value::Null));
                    for _ in items {
                        match eval_stmt(body, env.clone(), loop_stack) {
                            Ok(_) => continue,
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Break(break_label),
                            }) => {
                                if should_break_loop(label, &break_label) {
                                    iter_result = Ok(Some(Value::Null));
                                    break;
                                } else {
                                    iter_result = Err(RuntimeError::ControlFlow {
                                        flow: ControlFlow::Break(break_label),
                                    });
                                    break;
                                }
                            }
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Continue(continue_label),
                            }) => {
                                if should_continue_loop(label, &continue_label) {
                                    continue;
                                } else {
                                    iter_result = Err(RuntimeError::ControlFlow {
                                        flow: ControlFlow::Continue(continue_label),
                                    });
                                    break;
                                }
                            }
                            Err(other_error) => {
                                iter_result = Err(other_error);
                                break;
                            }
                        }
                    }
                    iter_result
                }
                LoopBindings::One(var_name) => {
                    // loop through list with x { ... }
                    let mut iter_result = Ok(Some(Value::Null));
                    for item in items {
                        let loop_env = Rc::new(Env::new_child(env.clone()));
                        loop_env.define_or_set(var_name, item);

                        match eval_stmt(body, loop_env, loop_stack) {
                            Ok(_) => continue,
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Break(break_label),
                            }) => {
                                if should_break_loop(label, &break_label) {
                                    iter_result = Ok(Some(Value::Null));
                                    break;
                                } else {
                                    iter_result = Err(RuntimeError::ControlFlow {
                                        flow: ControlFlow::Break(break_label),
                                    });
                                    break;
                                }
                            }
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Continue(continue_label),
                            }) => {
                                if should_continue_loop(label, &continue_label) {
                                    continue;
                                } else {
                                    iter_result = Err(RuntimeError::ControlFlow {
                                        flow: ControlFlow::Continue(continue_label),
                                    });
                                    break;
                                }
                            }
                            Err(other_error) => {
                                iter_result = Err(other_error);
                                break;
                            }
                        }
                    }
                    iter_result
                }
                LoopBindings::Two(_, _) => Err(RuntimeError::TypeError {
                    message: "List iteration requires at most one binding variable".to_string(),
                }),
            }
        }
        Value::Map(map) => {
            match bindings {
                LoopBindings::None => {
                    // loop through map { ... } - no bindings
                    let mut iter_result = Ok(Some(Value::Null));
                    for _ in map.iter() {
                        match eval_stmt(body, env.clone(), loop_stack) {
                            Ok(_) => continue,
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Break(break_label),
                            }) => {
                                if should_break_loop(label, &break_label) {
                                    iter_result = Ok(Some(Value::Null));
                                    break;
                                } else {
                                    iter_result = Err(RuntimeError::ControlFlow {
                                        flow: ControlFlow::Break(break_label),
                                    });
                                    break;
                                }
                            }
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Continue(continue_label),
                            }) => {
                                if should_continue_loop(label, &continue_label) {
                                    continue;
                                } else {
                                    iter_result = Err(RuntimeError::ControlFlow {
                                        flow: ControlFlow::Continue(continue_label),
                                    });
                                    break;
                                }
                            }
                            Err(other_error) => {
                                iter_result = Err(other_error);
                                break;
                            }
                        }
                    }
                    iter_result
                }
                LoopBindings::One(var_name) => {
                    // loop through map with k { ... } - bind keys only
                    let mut iter_result = Ok(Some(Value::Null));
                    for (key, _) in map.iter() {
                        let loop_env = Rc::new(Env::new_child(env.clone()));
                        loop_env.define_or_set(var_name, key.to_value());

                        match eval_stmt(body, loop_env, loop_stack) {
                            Ok(_) => continue,
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Break(break_label),
                            }) => {
                                if should_break_loop(label, &break_label) {
                                    iter_result = Ok(Some(Value::Null));
                                    break;
                                } else {
                                    iter_result = Err(RuntimeError::ControlFlow {
                                        flow: ControlFlow::Break(break_label),
                                    });
                                    break;
                                }
                            }
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Continue(continue_label),
                            }) => {
                                if should_continue_loop(label, &continue_label) {
                                    continue;
                                } else {
                                    iter_result = Err(RuntimeError::ControlFlow {
                                        flow: ControlFlow::Continue(continue_label),
                                    });
                                    break;
                                }
                            }
                            Err(other_error) => {
                                iter_result = Err(other_error);
                                break;
                            }
                        }
                    }
                    iter_result
                }
                LoopBindings::Two(key_name, value_name) => {
                    // loop through map with k, v { ... }
                    let mut iter_result = Ok(Some(Value::Null));
                    for (key, value) in map.iter() {
                        let loop_env = Rc::new(Env::new_child(env.clone()));
                        loop_env.define_or_set(key_name, key.to_value());
                        loop_env.define_or_set(value_name, value.clone());

                        match eval_stmt(body, loop_env, loop_stack) {
                            Ok(_) => continue,
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Break(break_label),
                            }) => {
                                if should_break_loop(label, &break_label) {
                                    iter_result = Ok(Some(Value::Null));
                                    break;
                                } else {
                                    iter_result = Err(RuntimeError::ControlFlow {
                                        flow: ControlFlow::Break(break_label),
                                    });
                                    break;
                                }
                            }
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Continue(continue_label),
                            }) => {
                                if should_continue_loop(label, &continue_label) {
                                    continue;
                                } else {
                                    iter_result = Err(RuntimeError::ControlFlow {
                                        flow: ControlFlow::Continue(continue_label),
                                    });
                                    break;
                                }
                            }
                            Err(other_error) => {
                                iter_result = Err(other_error);
                                break;
                            }
                        }
                    }
                    iter_result
                }
            }
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot iterate over {}", iterable_value.type_name()),
        }),
    };

    // Pop label from loop stack
    if label.is_some() {
        loop_stack.pop();
    }

    result
}

/// Evaluate a match statement
pub fn eval_match(
    scrutinee: &Expr,
    arms: &[MatchArm],
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
) -> EvalResult<Option<Value>> {
    use super::pattern_matches;

    let scrutinee_value = eval_expr(scrutinee, env.clone())?;

    for arm in arms {
        if pattern_matches(&arm.pattern, &scrutinee_value)? {
            return eval_stmt(&arm.body, env, loop_stack);
        }
    }

    // No pattern matched - this is valid, just do nothing
    Ok(Some(Value::Null))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal, Stmt};
    use crate::runtime::env::Env;
    use crate::token::Span;

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        crate::runtime::builtins::setup_global_env(&env);
        env
    }

    #[test]
    fn test_block_evaluation() {
        let env = create_test_env();

        let statements = vec![
            Stmt::Expr(Expr::Literal(Literal::Number(1.0, Span::default()))),
            Stmt::Expr(Expr::Literal(Literal::Number(2.0, Span::default()))),
            Stmt::Expr(Expr::Literal(Literal::Number(3.0, Span::default()))),
        ];

        let mut loop_stack = Vec::new();
        let result = eval_block(&statements, env, &mut loop_stack).unwrap();

        assert_eq!(result, Some(Value::Number(3.0)));
    }

    #[test]
    fn test_list_iteration() {
        let env = create_test_env();
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        env.define_or_set("items", list);

        let iterable = Expr::Literal(Literal::Identifier("items".to_string(), Span::default()));
        let bindings = LoopBindings::One("x".to_string());
        let body = Stmt::Expr(Expr::Literal(Literal::Identifier(
            "x".to_string(),
            Span::default(),
        )));

        let mut loop_stack = Vec::new();
        let result = eval_loop_through(None, &iterable, &bindings, &body, env, &mut loop_stack);

        assert!(result.is_ok());
    }
}
