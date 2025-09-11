use super::env::Env;
use super::module::ModuleRegistry;
use super::value::{ControlFlow, RuntimeError, Value};
use crate::ast::{Expr, Stmt};
use std::rc::Rc;

// Module declarations
mod assignments;
mod exports;
mod functions;
mod helpers;
mod imports;
mod indexing;
mod literals;
mod methods;
mod operators;
mod patterns;
mod postfix;
mod statements;

// Re-export the main evaluation functions
pub use assignments::*;
pub use exports::*;
pub use functions::*;
pub use helpers::*;
pub use imports::*;
pub use indexing::*;
pub use literals::*;
pub use methods::*;
pub use operators::*;
pub use patterns::*;
pub use postfix::*;
pub use statements::*;

/// Result type for evaluation that can return control flow signals
pub type EvalResult<T> = Result<T, RuntimeError>;

/// Evaluate an expression in the given environment
pub fn eval_expr(expr: &Expr, env: Rc<Env>) -> EvalResult<Value> {
    match expr {
        Expr::Literal(literal) => eval_literal(literal, env),

        Expr::Unary { op, expr, .. } => {
            let value = eval_expr(expr, env)?;
            eval_unary_op(op, value)
        }

        Expr::Binary {
            left, op, right, ..
        } => eval_binary_expr(left, op, right, env),

        Expr::PostfixIncrement { target, .. } => eval_postfix_increment(target, env),

        Expr::PostfixDecrement { target, .. } => eval_postfix_decrement(target, env),

        Expr::Call { callee, args, .. } => eval_function_call(callee, args, env),

        Expr::Grouping { expr, .. } => eval_expr(expr, env),

        Expr::FunctionLiteral { params, body, .. } => eval_function_literal(params, body, env),

        Expr::ShellCommandTemplate { parts, .. } => eval_shell_command_template(parts, env),

        Expr::Index { target, index, .. } => eval_index(target, index, env),

        Expr::Slice {
            target, start, end, ..
        } => eval_slice(target, start.as_deref(), end.as_deref(), env),

        Expr::MapAccessByName { target, key, .. } => eval_map_access_by_name(target, key, env),

        Expr::Assign { target, value, .. } => eval_assignment(target, value, env),

        Expr::CompoundAssign {
            target, op, value, ..
        } => eval_compound_assignment(target, *op, value, env),

        Expr::MethodCall {
            target,
            method,
            args,
            ..
        } => eval_method_call(target, method, args, env),

        Expr::Match {
            scrutinee, arms, ..
        } => eval_match_expression(scrutinee.as_deref(), arms, env),
    }
}

/// Evaluate a statement in the given environment
pub fn eval_stmt(
    stmt: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
) -> EvalResult<Option<Value>> {
    eval_stmt_with_modules(stmt, env, loop_stack, &ModuleRegistry::new())
}

/// Evaluate a statement with a module registry
pub fn eval_stmt_with_modules(
    stmt: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    module_registry: &ModuleRegistry,
) -> EvalResult<Option<Value>> {
    match stmt {
        Stmt::Expr(expr) => {
            let value = eval_expr(expr, env)?;
            Ok(Some(value))
        }

        Stmt::Block { statements, .. } => {
            eval_block_with_modules(statements, env, loop_stack, module_registry)
        }

        Stmt::Return { value, .. } => {
            let return_value = match value {
                Some(expr) => eval_expr(expr, env)?,
                None => Value::Nil,
            };
            Err(RuntimeError::ControlFlow {
                flow: ControlFlow::Return(Box::new(return_value)),
            })
        }

        Stmt::Break { label, .. } => Err(RuntimeError::ControlFlow {
            flow: ControlFlow::Break(label.clone()),
        }),

        Stmt::Continue { label, .. } => Err(RuntimeError::ControlFlow {
            flow: ControlFlow::Continue(label.clone()),
        }),

        Stmt::Loop { label, body, .. } => eval_infinite_loop_with_modules(
            label.as_deref(),
            body,
            env,
            loop_stack,
            module_registry,
        ),

        Stmt::LoopThrough {
            label,
            iterable,
            bindings,
            body,
            ..
        } => eval_loop_through_with_modules(
            label.as_deref(),
            iterable,
            bindings,
            body,
            env,
            loop_stack,
            module_registry,
        ),

        Stmt::Import { spec, .. } => {
            eval_import(spec, env, module_registry)?;
            Ok(Some(Value::Nil))
        }

        Stmt::Export { spec, .. } => {
            let export_result = eval_export(spec, env)?;
            Ok(Some(export_result.module))
        }
    }
}

/// Wrapper functions to handle module registry
/// Block evaluation with module registry
fn eval_block_with_modules(
    statements: &[Stmt],
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    module_registry: &ModuleRegistry,
) -> EvalResult<Option<Value>> {
    let block_env = Rc::new(Env::new_child(env));
    let mut last_value = None;

    for stmt in statements {
        match eval_stmt_with_modules(stmt, block_env.clone(), loop_stack, module_registry) {
            Ok(value) => last_value = value,
            Err(e) => return Err(e),
        }
    }

    Ok(last_value)
}

/// Infinite loop evaluation with module registry
fn eval_infinite_loop_with_modules(
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
        match eval_stmt_with_modules(body, env.clone(), loop_stack, module_registry) {
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
fn eval_loop_through_with_modules(
    label: Option<&str>,
    iterable: &Expr,
    bindings: &crate::ast::LoopBindings,
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
        (Value::List(items), crate::ast::LoopBindings::None) => {
            let mut iter = items.into_iter();
            loop {
                if iter.next().is_none() {
                    break;
                }
                match eval_stmt_with_modules(body, env.clone(), loop_stack, module_registry) {
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
        (Value::List(items), crate::ast::LoopBindings::One(var)) => {
            let mut iter = items.into_iter();
            loop {
                let item = match iter.next() {
                    Some(item) => item,
                    None => break,
                };

                let loop_env = Rc::new(Env::new_child(env.clone()));
                loop_env.define_or_set(var, item);

                match eval_stmt_with_modules(body, loop_env, loop_stack, module_registry) {
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
        (Value::Map(map), crate::ast::LoopBindings::None) => {
            // loop through map { ... } - no bindings
            for _ in map.iter() {
                match eval_stmt_with_modules(body, env.clone(), loop_stack, module_registry) {
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
        (Value::Map(map), crate::ast::LoopBindings::One(var)) => {
            // loop through map with k { ... } - bind keys only
            for (key, _) in map.iter() {
                let loop_env = Rc::new(Env::new_child(env.clone()));
                loop_env.define_or_set(var, key.to_value());

                match eval_stmt_with_modules(body, loop_env, loop_stack, module_registry) {
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
        (Value::Map(map), crate::ast::LoopBindings::Two(key_var, value_var)) => {
            // loop through map with k, v { ... }
            for (key, value) in map.iter() {
                let loop_env = Rc::new(Env::new_child(env.clone()));
                loop_env.define_or_set(key_var, key.to_value());
                loop_env.define_or_set(value_var, value.clone());

                match eval_stmt_with_modules(body, loop_env, loop_stack, module_registry) {
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

/// Evaluate a program (list of statements) with module support
pub fn eval_program_with_modules(
    statements: &[Stmt],
    env: Rc<Env>,
    module_registry: &ModuleRegistry,
) -> EvalResult<Option<Value>> {
    let mut loop_stack = Vec::new();
    let mut last_value = None;

    for stmt in statements {
        match eval_stmt_with_modules(stmt, env.clone(), &mut loop_stack, module_registry) {
            Ok(value) => last_value = value,
            Err(e) => return Err(e),
        }
    }

    Ok(last_value)
}

/// Evaluate a program (list of statements) using default module registry
pub fn eval_program(statements: &[Stmt], env: Rc<Env>) -> EvalResult<Option<Value>> {
    eval_program_with_modules(statements, env, &ModuleRegistry::new())
}

/// Evaluate a match expression
pub fn eval_match_expression(
    scrutinee: Option<&Expr>,
    arms: &[crate::ast::MatchArm],
    env: Rc<Env>,
) -> EvalResult<Value> {
    for arm in arms {
        let matches = if let Some(scrutinee_expr) = scrutinee {
            // Traditional match: evaluate scrutinee and use pattern matching
            let scrutinee_value = eval_expr(scrutinee_expr, env.clone())?;
            pattern_matches(&arm.pattern, &scrutinee_value)?
        } else {
            // Conditional match: evaluate expression pattern directly
            patterns::expression_pattern_matches(&arm.pattern, env.clone())?
        };

        if matches {
            // Evaluate the arm body and handle implicit returns
            let mut loop_stack = Vec::new();
            match eval_stmt(&arm.body, env.clone(), &mut loop_stack) {
                Ok(result) => {
                    // Handle implicit returns
                    match result {
                        Some(value) => return Ok(value), // Statement returned a value
                        None => {
                            // No explicit return, check if arm body was a single expression
                            match &arm.body {
                                Stmt::Expr(expr) => {
                                    // Single expression arm body - return its value
                                    return eval_expr(expr, env);
                                }
                                Stmt::Block { statements, .. } => {
                                    // Block arm body - check if last statement was an expression
                                    if let Some(Stmt::Expr(expr)) = statements.last() {
                                        return eval_expr(expr, env);
                                    } else {
                                        return Ok(Value::Nil); // Last statement was not an expression or empty block
                                    }
                                }
                                _ => return Ok(Value::Nil), // Other statement types
                            }
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    // No pattern matched - this is valid, just return nil
    Ok(Value::Nil)
}
