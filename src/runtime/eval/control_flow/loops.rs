use super::super::super::eval::{
    EvalResult, eval_expr, eval_stmt, should_break_loop, should_continue_loop,
};
use crate::ast::{Expr, LoopBindings, Stmt};
use crate::runtime::env::Env;
use crate::runtime::module::ModuleRegistry;
use crate::runtime::value::{ControlFlow, RuntimeError, Value};
use std::rc::Rc;

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

/// Infinite loop evaluation with module registry
pub fn eval_infinite_loop_with_modules(
    label: Option<&str>,
    body: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    module_registry: &ModuleRegistry,
) -> EvalResult<Option<Value>> {
    if let Some(label) = label {
        loop_stack.push(label.to_string());
    }

    let result = loop {
        match super::super::eval_stmt_with_modules(body, env.clone(), loop_stack, module_registry) {
            Ok(_) => continue,
            Err(RuntimeError::ControlFlow { flow }) => match flow {
                ControlFlow::Break(None) => break Ok(Some(Value::Nil)),
                ControlFlow::Break(Some(ref target)) => {
                    if label.is_some_and(|l| l == target) {
                        break Ok(Some(Value::Nil));
                    } else {
                        break Err(RuntimeError::ControlFlow { flow });
                    }
                }
                ControlFlow::Continue(None) => continue,
                ControlFlow::Continue(Some(ref target)) => {
                    if label.is_some_and(|l| l == target) {
                        continue;
                    } else {
                        break Err(RuntimeError::ControlFlow { flow });
                    }
                }
                ControlFlow::Return(_) => break Err(RuntimeError::ControlFlow { flow }),
            },
            Err(e) => break Err(e),
        }
    };

    if label.is_some() {
        loop_stack.pop();
    }

    result
}

/// Loop through evaluation with module registry
pub fn eval_loop_through_with_modules(
    label: Option<&str>,
    iterable: &Expr,
    bindings: &LoopBindings,
    body: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    module_registry: &ModuleRegistry,
) -> EvalResult<Option<Value>> {
    let iterable_value = eval_expr(iterable, env.clone())?;
    let iterable_type_name = iterable_value.type_name();

    if let Some(label) = label {
        loop_stack.push(label.to_string());
    }

    let result = match (iterable_value, bindings) {
        (Value::List(items), LoopBindings::None) => {
            let mut iter = items.into_iter();
            loop {
                if iter.next().is_none() {
                    break;
                }
                match super::super::eval_stmt_with_modules(
                    body,
                    env.clone(),
                    loop_stack,
                    module_registry,
                ) {
                    Ok(_) => continue,
                    Err(RuntimeError::ControlFlow { flow }) => match flow {
                        ControlFlow::Break(None) => return Ok(Some(Value::Nil)),
                        ControlFlow::Break(Some(ref target)) => {
                            if label.is_some_and(|l| l == target) {
                                return Ok(Some(Value::Nil));
                            } else {
                                return Err(RuntimeError::ControlFlow { flow });
                            }
                        }
                        ControlFlow::Continue(None) => continue,
                        ControlFlow::Continue(Some(ref target)) => {
                            if label.is_some_and(|l| l == target) {
                                continue;
                            } else {
                                return Err(RuntimeError::ControlFlow { flow });
                            }
                        }
                        ControlFlow::Return(_) => return Err(RuntimeError::ControlFlow { flow }),
                    },
                    Err(e) => return Err(e),
                }
            }
            Ok(Some(Value::Nil))
        }
        (Value::List(items), LoopBindings::One(var)) => {
            let mut iter = items.into_iter();
            loop {
                let item = match iter.next() {
                    Some(item) => item,
                    None => break,
                };

                let loop_env = Rc::new(Env::new_child(env.clone()));
                loop_env.define_or_set(var, item);

                match super::super::eval_stmt_with_modules(
                    body,
                    loop_env,
                    loop_stack,
                    module_registry,
                ) {
                    Ok(_) => continue,
                    Err(RuntimeError::ControlFlow { flow }) => match flow {
                        ControlFlow::Break(None) => return Ok(Some(Value::Nil)),
                        ControlFlow::Break(Some(ref target)) => {
                            if label.is_some_and(|l| l == target) {
                                return Ok(Some(Value::Nil));
                            } else {
                                return Err(RuntimeError::ControlFlow { flow });
                            }
                        }
                        ControlFlow::Continue(None) => continue,
                        ControlFlow::Continue(Some(ref target)) => {
                            if label.is_some_and(|l| l == target) {
                                continue;
                            } else {
                                return Err(RuntimeError::ControlFlow { flow });
                            }
                        }
                        ControlFlow::Return(_) => return Err(RuntimeError::ControlFlow { flow }),
                    },
                    Err(e) => return Err(e),
                }
            }
            Ok(Some(Value::Nil))
        }
        (Value::Map(map), LoopBindings::None) => {
            // loop through map { ... } - no bindings
            for _ in map.iter() {
                match super::super::eval_stmt_with_modules(
                    body,
                    env.clone(),
                    loop_stack,
                    module_registry,
                ) {
                    Ok(_) => continue,
                    Err(RuntimeError::ControlFlow { flow }) => match flow {
                        ControlFlow::Break(None) => return Ok(Some(Value::Nil)),
                        ControlFlow::Break(Some(ref target)) => {
                            if label.is_some_and(|l| l == target) {
                                return Ok(Some(Value::Nil));
                            } else {
                                return Err(RuntimeError::ControlFlow { flow });
                            }
                        }
                        ControlFlow::Continue(None) => continue,
                        ControlFlow::Continue(Some(ref target)) => {
                            if label.is_some_and(|l| l == target) {
                                continue;
                            } else {
                                return Err(RuntimeError::ControlFlow { flow });
                            }
                        }
                        ControlFlow::Return(_) => return Err(RuntimeError::ControlFlow { flow }),
                    },
                    Err(e) => return Err(e),
                }
            }
            Ok(Some(Value::Nil))
        }
        (Value::Map(map), LoopBindings::One(var)) => {
            // loop through map with k { ... } - bind keys only
            for (key, _) in map.iter() {
                let loop_env = Rc::new(Env::new_child(env.clone()));
                loop_env.define_or_set(var, key.to_value());

                match super::super::eval_stmt_with_modules(
                    body,
                    loop_env,
                    loop_stack,
                    module_registry,
                ) {
                    Ok(_) => continue,
                    Err(RuntimeError::ControlFlow { flow }) => match flow {
                        ControlFlow::Break(None) => return Ok(Some(Value::Nil)),
                        ControlFlow::Break(Some(ref target)) => {
                            if label.is_some_and(|l| l == target) {
                                return Ok(Some(Value::Nil));
                            } else {
                                return Err(RuntimeError::ControlFlow { flow });
                            }
                        }
                        ControlFlow::Continue(None) => continue,
                        ControlFlow::Continue(Some(ref target)) => {
                            if label.is_some_and(|l| l == target) {
                                continue;
                            } else {
                                return Err(RuntimeError::ControlFlow { flow });
                            }
                        }
                        ControlFlow::Return(_) => return Err(RuntimeError::ControlFlow { flow }),
                    },
                    Err(e) => return Err(e),
                }
            }
            Ok(Some(Value::Nil))
        }
        (Value::Map(map), LoopBindings::Two(key_var, value_var)) => {
            // loop through map with k, v { ... }
            for (key, value) in map.iter() {
                let loop_env = Rc::new(Env::new_child(env.clone()));
                loop_env.define_or_set(key_var, key.to_value());
                loop_env.define_or_set(value_var, value.clone());

                match super::super::eval_stmt_with_modules(
                    body,
                    loop_env,
                    loop_stack,
                    module_registry,
                ) {
                    Ok(_) => continue,
                    Err(RuntimeError::ControlFlow { flow }) => match flow {
                        ControlFlow::Break(None) => return Ok(Some(Value::Nil)),
                        ControlFlow::Break(Some(ref target)) => {
                            if label.is_some_and(|l| l == target) {
                                return Ok(Some(Value::Nil));
                            } else {
                                return Err(RuntimeError::ControlFlow { flow });
                            }
                        }
                        ControlFlow::Continue(None) => continue,
                        ControlFlow::Continue(Some(ref target)) => {
                            if label.is_some_and(|l| l == target) {
                                continue;
                            } else {
                                return Err(RuntimeError::ControlFlow { flow });
                            }
                        }
                        ControlFlow::Return(_) => return Err(RuntimeError::ControlFlow { flow }),
                    },
                    Err(e) => return Err(e),
                }
            }
            Ok(Some(Value::Nil))
        }
        _ => {
            // For unsupported iterables
            Err(RuntimeError::TypeError {
                message: format!("Cannot iterate over {}", iterable_type_name),
            })
        }
    };

    if label.is_some() {
        loop_stack.pop();
    }

    result
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
