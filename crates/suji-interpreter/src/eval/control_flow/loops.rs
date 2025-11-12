use crate::eval::{EvalResult, eval_expr, eval_stmt};
use std::rc::Rc;
use suji_ast::ast::{Expr, LoopBindings, Stmt};
use suji_runtime::ModuleRegistry;
use suji_values::Env;
use suji_values::{ControlFlow, RuntimeError, Value};

/// Infinite loop evaluation with optional module registry
pub fn eval_infinite_loop(
    label: Option<&str>,
    body: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Option<Value>> {
    if let Some(label) = label {
        loop_stack.push(label.to_string());
    }

    let result = loop {
        match eval_stmt(body, env.clone(), loop_stack, registry) {
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

/// Loop through evaluation with optional module registry
pub fn eval_loop_through(
    label: Option<&str>,
    iterable: &Expr,
    bindings: &LoopBindings,
    body: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Option<Value>> {
    let iterable_value = eval_expr(iterable, env.clone(), registry)?;
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
                match eval_stmt(body, env.clone(), loop_stack, registry) {
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

                match eval_stmt(body, loop_env, loop_stack, registry) {
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
                match eval_stmt(body, env.clone(), loop_stack, registry) {
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

                match eval_stmt(body, loop_env, loop_stack, registry) {
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

                match eval_stmt(body, loop_env, loop_stack, registry) {
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
