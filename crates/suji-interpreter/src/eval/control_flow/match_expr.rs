use crate::eval::{EvalResult, eval_expr, eval_stmt};
use std::rc::Rc;
use suji_ast::ast::{Expr, Stmt};
use suji_runtime::ModuleRegistry;
use suji_values::Env;
use suji_values::Value;

/// Evaluate a match expression
pub fn eval_match_expression(
    scrutinee: Option<&Expr>,
    arms: &[suji_ast::ast::MatchArm],
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Value> {
    for arm in arms {
        let matches = if let Some(scrutinee_expr) = scrutinee {
            // Traditional match: evaluate scrutinee and use pattern matching
            let scrutinee_value = eval_expr(scrutinee_expr, env.clone(), registry)?;
            super::super::patterns::pattern_matches(&arm.pattern, &scrutinee_value)?
        } else {
            // Conditional match: evaluate expression pattern directly
            super::super::patterns::expression_pattern_matches(&arm.pattern, env.clone(), registry)?
        };

        if matches {
            // Evaluate the arm body and handle implicit returns
            let mut loop_stack = Vec::new();
            match eval_stmt(&arm.body, env.clone(), &mut loop_stack, registry) {
                Ok(result) => {
                    // Handle implicit returns
                    match result {
                        Some(value) => return Ok(value), // Statement returned a value
                        None => {
                            // No explicit return, check if arm body was a single expression
                            match &arm.body {
                                Stmt::Expr(expr) => {
                                    // Single expression arm body - return its value
                                    return eval_expr(expr, env, registry);
                                }
                                Stmt::Block { statements, .. } => {
                                    // Block arm body - check if last statement was an expression
                                    if let Some(Stmt::Expr(expr)) = statements.last() {
                                        return eval_expr(expr, env, registry);
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
