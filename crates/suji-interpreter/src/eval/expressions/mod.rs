mod binary;
mod function_calls;
mod literals;
mod method_calls;
pub mod pipe;
mod unary;

pub use binary::*;
pub use function_calls::*;
pub use literals::*;
pub use unary::*;

use std::rc::Rc;
use suji_ast::ast::{BinaryOp, Expr};
use suji_runtime::ModuleRegistry;
use suji_values::{ControlFlow, Env, RuntimeError, Value};

use super::data_structures::{eval_index, eval_map_access_by_name, eval_slice};
use super::eval_match_expression;
use super::postfix::{eval_postfix_decrement, eval_postfix_increment};

/// Result type for evaluation that can return control flow signals
pub type EvalResult<T> = Result<T, RuntimeError>;

/// Evaluate an expression with optional module registry support
///
/// When `registry` is `Some`, enables:
/// - Pipe operators (| and |> and <|)
/// - Module access
/// - Method calls
/// - Assignments
///
/// When `registry` is `None`, these features will return errors if attempted.
pub fn eval_expr(
    expr: &Expr,
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Value> {
    let result = match expr {
        Expr::Literal(literal) => eval_literal(literal, env, registry),

        Expr::Unary {
            op, expr: inner, ..
        } => {
            let value = eval_expr(inner, env, registry)?;
            eval_unary_op(op, value)
        }

        Expr::Binary {
            left,
            op: BinaryOp::Pipe,
            right,
            ..
        } => {
            // Pipe operator requires registry
            pipe::eval_pipe_expression(left, right, env.clone(), registry)
        }

        Expr::Binary {
            left,
            op: op @ (BinaryOp::PipeApplyFwd | BinaryOp::PipeApplyBwd),
            right,
            ..
        } => {
            // Pipe apply operators
            pipe::eval_pipe_apply_expression(left, op, right, env.clone(), registry)
        }

        Expr::Binary {
            left, op, right, ..
        } => eval_binary_expr(left, op, right, env, registry),

        Expr::PostfixIncrement { target, .. } => eval_postfix_increment(target, env, registry),

        Expr::PostfixDecrement { target, .. } => eval_postfix_decrement(target, env, registry),

        Expr::Call { callee, args, .. } => eval_function_call(callee, args, env, registry),

        Expr::Grouping { expr: inner, .. } => eval_expr(inner, env, registry),

        Expr::FunctionLiteral { params, body, .. } => eval_function_literal(params, body, env),

        Expr::ShellCommandTemplate { parts, .. } => {
            eval_shell_command_template(parts, env, registry)
        }

        Expr::Index { target, index, .. } => eval_index(target, index, env, registry),

        Expr::Slice {
            target, start, end, ..
        } => eval_slice(target, start.as_deref(), end.as_deref(), env, registry),

        Expr::MapAccessByName { target, key, .. } => {
            let executor = registry.map(|_| &crate::AstInterpreter as &dyn suji_runtime::Executor);
            eval_map_access_by_name(target, key, env, executor, registry)
        }

        Expr::Assign { target, value, .. } => {
            use crate::eval::assignments::eval_assignment;
            eval_assignment(target, value, env, registry)
        }

        Expr::CompoundAssign {
            target, op, value, ..
        } => {
            use crate::eval::assignments::eval_compound_assignment;
            eval_compound_assignment(target, *op, value, env, registry)
        }

        Expr::MethodCall {
            target,
            method,
            args,
            ..
        } => method_calls::eval_method_call(target, method, args, env, registry),

        Expr::Match {
            scrutinee, arms, ..
        } => eval_match_expression(scrutinee.as_deref(), arms, env.clone(), registry),

        Expr::Destructure { .. } => Err(RuntimeError::InvalidOperation {
            message: "Destructuring pattern cannot appear as a standalone expression".to_string(),
        }),

        Expr::Return { values, .. } => {
            let return_value = if values.is_empty() {
                Value::Nil
            } else if values.len() == 1 {
                eval_expr(&values[0], env, registry)?
            } else {
                let mut tuple_values = Vec::new();
                for expr in values {
                    tuple_values.push(eval_expr(expr, env.clone(), registry)?);
                }
                Value::Tuple(tuple_values)
            };
            Err(RuntimeError::ControlFlow {
                flow: ControlFlow::Return(Box::new(return_value)),
            })
        }

        Expr::Break { label, .. } => Err(RuntimeError::ControlFlow {
            flow: ControlFlow::Break(label.clone()),
        }),

        Expr::Continue { label, .. } => Err(RuntimeError::ControlFlow {
            flow: ControlFlow::Continue(label.clone()),
        }),
    };

    // Wrap any error with the expression's covering span
    result.map_err(|e| e.with_span(expr.covering_span()))
}
