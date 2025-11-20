use super::handler::{ControlFlowAction, handle_control_flow};
use crate::eval::{EvalResult, eval_expr, eval_stmt};
use std::rc::Rc;
use suji_ast::{Expr, LoopBindings, Stmt};
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
            Err(e) => {
                match handle_control_flow(&e, label) {
                    ControlFlowAction::Continue => continue,
                    ControlFlowAction::Break(value) => break Ok(Some(value)),
                    ControlFlowAction::Return(value) => {
                        // Return should escape the loop as an error
                        break Err(RuntimeError::ControlFlow {
                            flow: ControlFlow::Return(Box::new(value)),
                        });
                    }
                    ControlFlowAction::Propagate(err) => break Err(err),
                }
            }
        }
    };

    if label.is_some() {
        loop_stack.pop();
    }

    result
}

/// Helper to execute loop body and handle control flow
fn execute_loop_body(
    body: &Stmt,
    env: Rc<Env>,
    label: Option<&str>,
    loop_stack: &mut Vec<String>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Option<Value>> {
    match eval_stmt(body, env, loop_stack, registry) {
        Ok(_) => Ok(None), // Continue iteration
        Err(e) => match handle_control_flow(&e, label) {
            ControlFlowAction::Continue => Ok(None), // Continue iteration
            ControlFlowAction::Break(value) => Ok(Some(value)), // Break from loop
            ControlFlowAction::Return(value) => Err(RuntimeError::ControlFlow {
                flow: ControlFlow::Return(Box::new(value)),
            }),
            ControlFlowAction::Propagate(err) => Err(err),
        },
    }
}

/// List iteration with no bindings
fn eval_loop_through_list_none(
    items: Vec<Value>,
    label: Option<&str>,
    body: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Option<Value>> {
    let mut iter = items.into_iter();
    loop {
        if iter.next().is_none() {
            break;
        }
        match execute_loop_body(body, env.clone(), label, loop_stack, registry) {
            Ok(None) => continue,                      // Continue iteration
            Ok(Some(value)) => return Ok(Some(value)), // Break from loop
            Err(e) => return Err(e),                   // Propagate error (Return or other)
        }
    }
    Ok(Some(Value::Nil))
}

/// List iteration with one binding
fn eval_loop_through_list_one(
    items: Vec<Value>,
    var: &str,
    label: Option<&str>,
    body: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Option<Value>> {
    let mut iter = items.into_iter();
    loop {
        let item = match iter.next() {
            Some(item) => item,
            None => break,
        };

        let loop_env = Rc::new(Env::new_child(env.clone()));
        loop_env.define_or_set(var, item);

        match execute_loop_body(body, loop_env, label, loop_stack, registry) {
            Ok(None) => continue,                      // Continue iteration
            Ok(Some(value)) => return Ok(Some(value)), // Break from loop
            Err(e) => return Err(e),                   // Propagate error (Return or other)
        }
    }
    Ok(Some(Value::Nil))
}

/// Map iteration with no bindings
fn eval_loop_through_map_none(
    map: indexmap::IndexMap<suji_values::MapKey, Value>,
    label: Option<&str>,
    body: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Option<Value>> {
    for _ in map.iter() {
        match execute_loop_body(body, env.clone(), label, loop_stack, registry) {
            Ok(None) => continue,                      // Continue iteration
            Ok(Some(value)) => return Ok(Some(value)), // Break from loop
            Err(e) => return Err(e),                   // Propagate error (Return or other)
        }
    }
    Ok(Some(Value::Nil))
}

/// Map iteration with one binding (keys only)
fn eval_loop_through_map_one(
    map: indexmap::IndexMap<suji_values::MapKey, Value>,
    var: &str,
    label: Option<&str>,
    body: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Option<Value>> {
    for (key, _) in map.iter() {
        let loop_env = Rc::new(Env::new_child(env.clone()));
        loop_env.define_or_set(var, key.to_value());

        match execute_loop_body(body, loop_env, label, loop_stack, registry) {
            Ok(None) => continue,                      // Continue iteration
            Ok(Some(value)) => return Ok(Some(value)), // Break from loop
            Err(e) => return Err(e),                   // Propagate error (Return or other)
        }
    }
    Ok(Some(Value::Nil))
}

/// Map iteration with two bindings (key, value)
#[allow(clippy::too_many_arguments)]
fn eval_loop_through_map_two(
    map: indexmap::IndexMap<suji_values::MapKey, Value>,
    key_var: &str,
    value_var: &str,
    label: Option<&str>,
    body: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Option<Value>> {
    for (key, value) in map.iter() {
        let loop_env = Rc::new(Env::new_child(env.clone()));
        loop_env.define_or_set(key_var, key.to_value());
        loop_env.define_or_set(value_var, value.clone());

        match execute_loop_body(body, loop_env, label, loop_stack, registry) {
            Ok(None) => continue,                      // Continue iteration
            Ok(Some(value)) => return Ok(Some(value)), // Break from loop
            Err(e) => return Err(e),                   // Propagate error (Return or other)
        }
    }
    Ok(Some(Value::Nil))
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
            eval_loop_through_list_none(items, label, body, env, loop_stack, registry)
        }
        (Value::List(items), LoopBindings::One(var)) => {
            eval_loop_through_list_one(items, var, label, body, env, loop_stack, registry)
        }
        (Value::Map(map), LoopBindings::None) => {
            eval_loop_through_map_none(map, label, body, env, loop_stack, registry)
        }
        (Value::Map(map), LoopBindings::One(var)) => {
            eval_loop_through_map_one(map, var, label, body, env, loop_stack, registry)
        }
        (Value::Map(map), LoopBindings::Two(key_var, value_var)) => eval_loop_through_map_two(
            map, key_var, value_var, label, body, env, loop_stack, registry,
        ),
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
