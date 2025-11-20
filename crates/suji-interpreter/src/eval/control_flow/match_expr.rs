use crate::eval::{EvalResult, eval_expr, eval_stmt, implicit_return::eval_implicit_return};
use std::rc::Rc;
use suji_ast::Expr;
use suji_runtime::ModuleRegistry;
use suji_values::Env;
use suji_values::Value;

/// Evaluate a match expression
pub fn eval_match_expression(
    scrutinee: Option<&Expr>,
    arms: &[suji_ast::MatchArm],
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
                            // No explicit return, use shared implicit return logic
                            return eval_implicit_return(&arm.body, env, registry);
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
