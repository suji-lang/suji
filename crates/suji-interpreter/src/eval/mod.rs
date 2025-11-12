use std::rc::Rc;
use suji_ast::ast::Stmt;
use suji_runtime::{Executor, ModuleRegistry};
use suji_values::{Env, RuntimeError, Value};

mod assignments;
mod exports;
mod function_call;
mod imports;
mod patterns;
mod postfix;

mod control_flow;
mod data_structures;
mod expressions;

pub use function_call::call_function;
pub use imports::*;

pub use control_flow::{eval_infinite_loop, eval_loop_through, eval_match_expression};
pub use expressions::eval_expr;

/// Result type for evaluation that can return control flow signals
pub type EvalResult<T> = Result<T, RuntimeError>;

/// Evaluate a statement in the given environment
pub fn eval_stmt(
    stmt: &Stmt,
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Option<Value>> {
    let result = match stmt {
        Stmt::Expr(expr) => {
            let value = expressions::eval_expr(expr, env, registry)?;
            Ok(Some(value))
        }

        Stmt::Block { statements, .. } => eval_block(statements, env, loop_stack, registry),

        Stmt::Loop { label, body, .. } => {
            eval_infinite_loop(label.as_deref(), body, env, loop_stack, registry)
        }

        Stmt::LoopThrough {
            label,
            iterable,
            bindings,
            body,
            ..
        } => {
            // Prefer pointing at the iterable expression for iteration type errors
            let result = eval_loop_through(
                label.as_deref(),
                iterable,
                bindings,
                body,
                env,
                loop_stack,
                registry,
            );
            result.map_err(|e| e.with_span(iterable.covering_span()))
        }

        Stmt::Import { spec, .. } => {
            let executor = crate::AstInterpreter;
            let registry = registry.ok_or_else(|| RuntimeError::InvalidOperation {
                message: "Import statements require a module registry".to_string(),
            })?;
            match eval_import(&executor, spec, env, registry) {
                Ok(()) => Ok(Some(Value::Nil)),
                Err(e) => Err(e),
            }
        }

        Stmt::Export { body, .. } => {
            // Evaluate both map and expression export bodies.
            // Map export returns a Value::Map; expression export returns its evaluated value.
            match exports::eval_export_body(body, env, None) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
            }
        }
    };

    // Attach the statement's span to errors that lack one
    result.map_err(|e| e.with_span(stmt.span().clone()))
}

/// Block evaluation with optional module registry
fn eval_block(
    statements: &[Stmt],
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Option<Value>> {
    let block_env = Rc::new(Env::new_child(env));
    let mut last_value = None;

    for stmt in statements {
        match eval_stmt(stmt, block_env.clone(), loop_stack, registry) {
            Ok(value) => last_value = value,
            Err(e) => return Err(e.with_span(stmt.span().clone())),
        }
    }

    Ok(last_value)
}

/// Adapter function for module registry callback
///
/// This is used to wire AstInterpreter's eval_source into the module loading system.
/// The module registry calls this function with an executor to evaluate module source code.
pub fn eval_module_source_callback(
    executor: &dyn Executor,
    source: &str,
    env: Rc<Env>,
    module_registry: &ModuleRegistry,
) -> Result<Value, RuntimeError> {
    executor.eval_source(source, env, module_registry, true)
}
