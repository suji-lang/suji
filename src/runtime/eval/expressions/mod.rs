// Expression evaluation modules
mod binary;
mod function_calls;
mod literals;
mod method_calls;
mod unary;

// Re-export the main expression evaluation functions
pub use binary::*;
pub use function_calls::*;
pub use literals::*;
pub use method_calls::*;
pub use unary::*;

use super::super::env::Env;
use super::super::value::{RuntimeError, Value};
use crate::ast::Expr;
use std::rc::Rc;

// Import functions that are used in eval_expr but defined in other modules
use super::assignments::eval_assignment;
use super::eval_match_expression;
use super::indexing::{eval_index, eval_map_access_by_name, eval_slice};
use super::postfix::{eval_postfix_decrement, eval_postfix_increment};

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
