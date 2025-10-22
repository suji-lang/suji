use super::super::super::eval::{EvalResult, eval_expr, eval_stmt};
use crate::runtime::env::Env;
use crate::runtime::value::Value;
use std::rc::Rc;
use suji_ast::ast::{Expr, MatchArm, Stmt};

/// Evaluate a match statement
pub fn eval_match(
    scrutinee: Option<&Expr>,
    arms: &[MatchArm],
    env: Rc<Env>,
    loop_stack: &mut Vec<String>,
) -> EvalResult<Option<Value>> {
    use super::super::patterns::expression_pattern_matches;
    use super::super::patterns::pattern_matches;

    for arm in arms {
        let matches = if let Some(scrutinee_expr) = scrutinee {
            // Traditional match: evaluate scrutinee and use pattern matching
            let scrutinee_value = eval_expr(scrutinee_expr, env.clone())?;
            pattern_matches(&arm.pattern, &scrutinee_value)?
        } else {
            // Conditional match: evaluate expression pattern directly
            expression_pattern_matches(&arm.pattern, env.clone())?
        };

        if matches {
            // Evaluate the arm body and handle implicit returns
            match eval_stmt(&arm.body, env.clone(), loop_stack) {
                Ok(result) => {
                    // Handle implicit returns
                    match result {
                        Some(value) => return Ok(Some(value)), // Statement returned a value
                        None => {
                            // No explicit return, check if arm body was a single expression
                            match &arm.body {
                                Stmt::Expr(expr) => {
                                    // Single expression arm body - return its value
                                    return Ok(Some(eval_expr(expr, env)?));
                                }
                                Stmt::Block { statements, .. } => {
                                    // Block arm body - check if last statement was an expression
                                    if let Some(Stmt::Expr(expr)) = statements.last() {
                                        return Ok(Some(eval_expr(expr, env)?));
                                    } else {
                                        return Ok(Some(Value::Nil)); // Last statement was not an expression or empty block
                                    }
                                }
                                _ => return Ok(Some(Value::Nil)), // Other statement types
                            }
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    // No pattern matched - this is valid, just do nothing
    Ok(Some(Value::Nil))
}

/// Evaluate a match expression
pub fn eval_match_expression(
    scrutinee: Option<&Expr>,
    arms: &[suji_ast::ast::MatchArm],
    env: Rc<Env>,
) -> EvalResult<Value> {
    for arm in arms {
        let matches = if let Some(scrutinee_expr) = scrutinee {
            // Traditional match: evaluate scrutinee and use pattern matching
            let scrutinee_value = eval_expr(scrutinee_expr, env.clone())?;
            super::super::patterns::pattern_matches(&arm.pattern, &scrutinee_value)?
        } else {
            // Conditional match: evaluate expression pattern directly
            super::super::patterns::expression_pattern_matches(&arm.pattern, env.clone())?
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::builtins::setup_global_env;
    use crate::runtime::env::Env;
    use crate::runtime::value::DecimalNumber;
    use suji_ast::Span;
    use suji_ast::ast::{Expr, Literal, Stmt};

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        setup_global_env(&env);
        env
    }

    #[test]
    fn test_match_expressions() {
        let env = create_test_env();

        // Test match with single expression arms (implicit returns)
        let scrutinee = Expr::Literal(Literal::Number("3".to_string(), Span::default()));
        let arms = vec![
            suji_ast::ast::MatchArm {
                pattern: suji_ast::ast::Pattern::Literal {
                    value: suji_ast::ast::ValueLike::Number("3".to_string()),
                    span: Span::default(),
                },
                body: Stmt::Expr(Expr::Binary {
                    left: Box::new(Expr::Literal(Literal::Number(
                        "5".to_string(),
                        Span::default(),
                    ))),
                    op: suji_ast::ast::BinaryOp::Add,
                    right: Box::new(Expr::Literal(Literal::Number(
                        "5".to_string(),
                        Span::default(),
                    ))),
                    span: Span::default(),
                }),
                span: Span::default(),
            },
            suji_ast::ast::MatchArm {
                pattern: suji_ast::ast::Pattern::Literal {
                    value: suji_ast::ast::ValueLike::Number("4".to_string()),
                    span: Span::default(),
                },
                body: Stmt::Expr(Expr::Binary {
                    left: Box::new(Expr::Literal(Literal::Number(
                        "2".to_string(),
                        Span::default(),
                    ))),
                    op: suji_ast::ast::BinaryOp::Multiply,
                    right: Box::new(Expr::Literal(Literal::Number(
                        "3".to_string(),
                        Span::default(),
                    ))),
                    span: Span::default(),
                }),
                span: Span::default(),
            },
            suji_ast::ast::MatchArm {
                pattern: suji_ast::ast::Pattern::Wildcard {
                    span: Span::default(),
                },
                body: Stmt::Expr(Expr::Literal(Literal::Number(
                    "0".to_string(),
                    Span::default(),
                ))),
                span: Span::default(),
            },
        ];

        let mut loop_stack = Vec::new();
        let result = eval_match(Some(&scrutinee), &arms, env.clone(), &mut loop_stack).unwrap();
        assert_eq!(result, Some(Value::Number(DecimalNumber::from_i64(10)))); // 5 + 5 = 10

        // Test match with block arms (last statement as expression)
        let scrutinee2 = Expr::Literal(Literal::Number("4".to_string(), Span::default()));
        let arms2 = vec![suji_ast::ast::MatchArm {
            pattern: suji_ast::ast::Pattern::Literal {
                value: suji_ast::ast::ValueLike::Number("4".to_string()),
                span: Span::default(),
            },
            body: Stmt::Block {
                statements: vec![
                    Stmt::Expr(Expr::Literal(Literal::Number(
                        "10".to_string(),
                        Span::default(),
                    ))),
                    Stmt::Expr(Expr::Binary {
                        left: Box::new(Expr::Literal(Literal::Number(
                            "2".to_string(),
                            Span::default(),
                        ))),
                        op: suji_ast::ast::BinaryOp::Multiply,
                        right: Box::new(Expr::Literal(Literal::Number(
                            "3".to_string(),
                            Span::default(),
                        ))),
                        span: Span::default(),
                    }),
                ],
                span: Span::default(),
            },
            span: Span::default(),
        }];

        let result2 = eval_match(Some(&scrutinee2), &arms2, env, &mut loop_stack).unwrap();
        assert_eq!(result2, Some(Value::Number(DecimalNumber::from_i64(6)))); // 2 * 3 = 6
    }
}
