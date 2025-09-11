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
                            break Ok(Some(Value::Nil));
                        } else {
                            // Break targets outer loop, propagate
                            break Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Break(break_label),
                            });
                        }
                    }
                    None => {
                        // Unlabeled break targets innermost loop (us)
                        break Ok(Some(Value::Nil));
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
                    let mut iter_result = Ok(Some(Value::Nil));
                    for _ in items {
                        match eval_stmt(body, env.clone(), loop_stack) {
                            Ok(_) => continue,
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Break(break_label),
                            }) => {
                                if should_break_loop(label, &break_label) {
                                    iter_result = Ok(Some(Value::Nil));
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
                    let mut iter_result = Ok(Some(Value::Nil));
                    for item in items {
                        let loop_env = Rc::new(Env::new_child(env.clone()));
                        loop_env.define_or_set(var_name, item);

                        match eval_stmt(body, loop_env, loop_stack) {
                            Ok(_) => continue,
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Break(break_label),
                            }) => {
                                if should_break_loop(label, &break_label) {
                                    iter_result = Ok(Some(Value::Nil));
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
                    let mut iter_result = Ok(Some(Value::Nil));
                    for _ in map.iter() {
                        match eval_stmt(body, env.clone(), loop_stack) {
                            Ok(_) => continue,
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Break(break_label),
                            }) => {
                                if should_break_loop(label, &break_label) {
                                    iter_result = Ok(Some(Value::Nil));
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
                    let mut iter_result = Ok(Some(Value::Nil));
                    for (key, _) in map.iter() {
                        let loop_env = Rc::new(Env::new_child(env.clone()));
                        loop_env.define_or_set(var_name, key.to_value());

                        match eval_stmt(body, loop_env, loop_stack) {
                            Ok(_) => continue,
                            Err(RuntimeError::ControlFlow {
                                flow: ControlFlow::Break(break_label),
                            }) => {
                                if should_break_loop(label, &break_label) {
                                    iter_result = Ok(Some(Value::Nil));
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
                    let mut iter_result = Ok(Some(Value::Nil));
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
                                    iter_result = Ok(Some(Value::Nil));
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
    scrutinee: Option<&Expr>,
    arms: &[MatchArm],
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
) -> EvalResult<Option<Value>> {
    use super::pattern_matches;
    use super::patterns::expression_pattern_matches;

    for arm in arms {
        let matches = if let Some(scrutinee_expr) = scrutinee {
            // Traditional match: evaluate scrutinee and use pattern matching
            let scrutinee_value = eval_expr(scrutinee_expr, env.clone())?;
            pattern_matches(&arm.pattern, &scrutinee_value)?
        } else {
            // Conditional match: evaluate expression pattern directly
            expression_pattern_matches(&arm.pattern, env.clone())?
        };

        if matches {
            // Evaluate the arm body and handle implicit returns
            match eval_stmt(&arm.body, env.clone(), loop_stack) {
                Ok(result) => {
                    // Handle implicit returns
                    match result {
                        Some(value) => return Ok(Some(value)), // Statement returned a value
                        None => {
                            // No explicit return, check if arm body was a single expression
                            match &arm.body {
                                Stmt::Expr(expr) => {
                                    // Single expression arm body - return its value
                                    return Ok(Some(eval_expr(expr, env)?));
                                }
                                Stmt::Block { statements, .. } => {
                                    // Block arm body - check if last statement was an expression
                                    if let Some(Stmt::Expr(expr)) = statements.last() {
                                        return Ok(Some(eval_expr(expr, env)?));
                                    } else {
                                        return Ok(Some(Value::Nil)); // Last statement was not an expression or empty block
                                    }
                                }
                                _ => return Ok(Some(Value::Nil)), // Other statement types
                            }
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    // No pattern matched - this is valid, just do nothing
    Ok(Some(Value::Nil))
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

    #[test]
    fn test_match_expressions() {
        let env = create_test_env();

        // Test match with single expression arms (implicit returns)
        let scrutinee = Expr::Literal(Literal::Number(3.0, Span::default()));
        let arms = vec![
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Literal {
                    value: crate::ast::ValueLike::Number(3.0),
                    span: Span::default(),
                },
                body: Stmt::Expr(Expr::Binary {
                    left: Box::new(Expr::Literal(Literal::Number(5.0, Span::default()))),
                    op: crate::ast::BinaryOp::Add,
                    right: Box::new(Expr::Literal(Literal::Number(5.0, Span::default()))),
                    span: Span::default(),
                }),
                span: Span::default(),
            },
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Literal {
                    value: crate::ast::ValueLike::Number(4.0),
                    span: Span::default(),
                },
                body: Stmt::Expr(Expr::Binary {
                    left: Box::new(Expr::Literal(Literal::Number(2.0, Span::default()))),
                    op: crate::ast::BinaryOp::Multiply,
                    right: Box::new(Expr::Literal(Literal::Number(3.0, Span::default()))),
                    span: Span::default(),
                }),
                span: Span::default(),
            },
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Wildcard {
                    span: Span::default(),
                },
                body: Stmt::Expr(Expr::Literal(Literal::Number(0.0, Span::default()))),
                span: Span::default(),
            },
        ];

        let mut loop_stack = Vec::new();
        let result = eval_match(Some(&scrutinee), &arms, env.clone(), &mut loop_stack).unwrap();
        assert_eq!(result, Some(Value::Number(10.0))); // 5 + 5 = 10

        // Test match with block arms (last statement as expression)
        let scrutinee2 = Expr::Literal(Literal::Number(4.0, Span::default()));
        let arms2 = vec![crate::ast::MatchArm {
            pattern: crate::ast::Pattern::Literal {
                value: crate::ast::ValueLike::Number(4.0),
                span: Span::default(),
            },
            body: Stmt::Block {
                statements: vec![
                    Stmt::Expr(Expr::Literal(Literal::Number(10.0, Span::default()))),
                    Stmt::Expr(Expr::Binary {
                        left: Box::new(Expr::Literal(Literal::Number(2.0, Span::default()))),
                        op: crate::ast::BinaryOp::Multiply,
                        right: Box::new(Expr::Literal(Literal::Number(3.0, Span::default()))),
                        span: Span::default(),
                    }),
                ],
                span: Span::default(),
            },
            span: Span::default(),
        }];

        let result2 = eval_match(Some(&scrutinee2), &arms2, env, &mut loop_stack).unwrap();
        assert_eq!(result2, Some(Value::Number(6.0))); // 2 * 3 = 6
    }
}
