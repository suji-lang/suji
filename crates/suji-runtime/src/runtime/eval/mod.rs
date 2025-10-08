use super::env::Env;
use super::module::ModuleRegistry;
use super::value::{ControlFlow, RuntimeError, Value};
use std::rc::Rc;
use suji_ast::ast::Stmt;

// Module declarations
mod assignments;
mod exports;
mod function_call;
mod functions;
mod helpers;
mod imports;
mod literals;
mod operators;
mod patterns;
mod postfix;

mod control_flow;
mod data_structures;
mod expressions;

// Re-export the main evaluation functions
pub use exports::eval_export;
pub use function_call::{call_function, call_function_with_modules};
pub use functions::*;
pub use helpers::*;
pub use imports::*;
pub use literals::*;
pub use operators::*;
pub use patterns::*;
pub use postfix::*;

pub use control_flow::{
    eval_infinite_loop, eval_infinite_loop_with_modules, eval_loop_through,
    eval_loop_through_with_modules, eval_match, eval_match_expression,
};
pub use data_structures::{eval_assignment, eval_index, eval_map_access_by_name, eval_slice};
pub use expressions::eval_expr;

/// Result type for evaluation that can return control flow signals
pub type EvalResult<T> = Result<T, RuntimeError>;

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
    let result = match stmt {
        Stmt::Expr(expr) => {
            let value = eval_expr(expr, env)?;
            Ok(Some(value))
        }

        Stmt::Block { statements, .. } => {
            eval_block_with_modules(statements, env, loop_stack, module_registry)
        }

        Stmt::Return { values, .. } => {
            let return_value = if values.is_empty() {
                Value::Nil
            } else if values.len() == 1 {
                eval_expr(&values[0], env)?
            } else {
                let mut tuple_values = Vec::new();
                for expr in values {
                    tuple_values.push(eval_expr(expr, env.clone())?);
                }
                Value::Tuple(tuple_values)
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
        } => {
            // Prefer pointing at the iterable expression for iteration type errors
            let result = eval_loop_through_with_modules(
                label.as_deref(),
                iterable,
                bindings,
                body,
                env,
                loop_stack,
                module_registry,
            );
            result.map_err(|e| e.with_span(iterable.covering_span()))
        }

        Stmt::Import { spec, .. } => match eval_import(spec, env, module_registry) {
            Ok(()) => Ok(Some(Value::Nil)),
            Err(e) => Err(e),
        },

        Stmt::Export { spec, .. } => match eval_export(spec, env) {
            Ok(export_result) => Ok(Some(export_result.module)),
            Err(e) => Err(e),
        },
    };

    // Attach the statement's span to errors that lack one
    result.map_err(|e| e.with_span(stmt.span().clone()))
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
            Err(e) => return Err(e.with_span(stmt.span().clone())),
        }
    }

    Ok(last_value)
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
            Err(e) => return Err(e.with_span(stmt.span().clone())),
        }
    }

    Ok(last_value)
}
