mod binary;
mod function_calls;
mod literals;
mod method_calls;
pub mod pipe;
mod unary;

pub use binary::*;
pub use function_calls::*;
pub use literals::*;
pub use method_calls::*;
pub use pipe::{eval_pipe_apply_expression, eval_pipe_expression};
pub use unary::*;

use super::super::env::Env;
use super::super::value::{RuntimeError, Value};
use std::rc::Rc;
use suji_ast::ast::Expr;

use super::assignments::eval_assignment;
use super::data_structures::{
    eval_index, eval_map_access_by_name, eval_map_access_by_name_with_modules, eval_slice,
};
use super::eval_match_expression;
use super::postfix::{eval_postfix_decrement, eval_postfix_increment};

/// Result type for evaluation that can return control flow signals
pub type EvalResult<T> = Result<T, RuntimeError>;

/// Evaluate an expression with a module registry (for pipe operator and lazy modules)
pub fn eval_expr_with_registry(
    expr: &Expr,
    env: Rc<Env>,
    registry: &super::super::module::ModuleRegistry,
) -> EvalResult<Value> {
    let result = match expr {
        Expr::Binary {
            left,
            op: suji_ast::ast::BinaryOp::Pipe,
            right,
            ..
        } => pipe::eval_pipe_expression_with_registry(left, right, env.clone(), registry),

        Expr::MapAccessByName { target, key, .. } => {
            eval_map_access_by_name_with_modules(target, key, env.clone(), registry)
        }

        _ => eval_expr(expr, env),
    };

    result.map_err(|e| e.with_span(expr.covering_span()))
}

/// Evaluate an expression in the given environment
pub fn eval_expr(expr: &Expr, env: Rc<Env>) -> EvalResult<Value> {
    let result = match expr {
        Expr::Literal(literal) => eval_literal(literal, env),

        Expr::Unary {
            op, expr: inner, ..
        } => {
            let value = eval_expr(inner, env)?;
            eval_unary_op(op, value)
        }

        Expr::Binary {
            left, op, right, ..
        } => eval_binary_expr(left, op, right, env),

        Expr::PostfixIncrement { target, .. } => eval_postfix_increment(target, env),

        Expr::PostfixDecrement { target, .. } => eval_postfix_decrement(target, env),

        Expr::Call { callee, args, .. } => eval_function_call(callee, args, env),

        Expr::Grouping { expr: inner, .. } => eval_expr(inner, env),

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

        Expr::Destructure { .. } => Err(RuntimeError::InvalidOperation {
            message: "Destructuring pattern cannot appear as a standalone expression".to_string(),
        }),

        Expr::Return { values, .. } => {
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
                flow: super::super::value::ControlFlow::Return(Box::new(return_value)),
            })
        }

        Expr::Break { label, .. } => Err(RuntimeError::ControlFlow {
            flow: super::super::value::ControlFlow::Break(label.clone()),
        }),

        Expr::Continue { label, .. } => Err(RuntimeError::ControlFlow {
            flow: super::super::value::ControlFlow::Continue(label.clone()),
        }),
    };

    // Wrap any error with the expression's covering span
    result.map_err(|e| e.with_span(expr.covering_span()))
}
