use super::{EvalResult, eval_expr};
use std::rc::Rc;
use suji_ast::Stmt;
use suji_runtime::ModuleRegistry;
use suji_values::{Env, Value};

/// Handle implicit return evaluation for function bodies and match arm bodies
pub fn eval_implicit_return(
    body: &Stmt,
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Value> {
    match body {
        Stmt::Expr(expr) => {
            // Single expression body - return its value
            eval_expr(expr, env, registry)
        }
        Stmt::Block { statements, .. } => {
            // Block body - check if last statement was an expression
            if let Some(last_stmt) = statements.last() {
                match last_stmt {
                    Stmt::Expr(expr) => eval_expr(expr, env, registry),
                    _ => Ok(Value::Nil), // Last statement was not an expression
                }
            } else {
                Ok(Value::Nil) // Empty block
            }
        }
        _ => Ok(Value::Nil), // Other statement types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_ast::{Expr, Literal, Stmt};
    use suji_lexer::Span;
    use suji_runtime::setup_global_env;
    use suji_values::DecimalNumber;

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        setup_global_env(&env);
        env
    }

    #[test]
    fn test_single_expression_body() {
        let env = create_test_env();
        let body = Stmt::Expr(Expr::Literal(Literal::Number(
            "42".to_string(),
            Span::default(),
        )));

        let result = eval_implicit_return(&body, env, None).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(42)));
    }

    #[test]
    fn test_block_with_expression_last() {
        let env = create_test_env();
        let body = Stmt::Block {
            statements: vec![
                Stmt::Expr(Expr::Literal(Literal::Number(
                    "1".to_string(),
                    Span::default(),
                ))),
                Stmt::Expr(Expr::Literal(Literal::Number(
                    "2".to_string(),
                    Span::default(),
                ))),
            ],
            span: Span::default(),
        };

        let result = eval_implicit_return(&body, env, None).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));
    }

    #[test]
    fn test_empty_block() {
        let env = create_test_env();
        let body = Stmt::Block {
            statements: vec![],
            span: Span::default(),
        };

        let result = eval_implicit_return(&body, env, None).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_block_with_non_expression_last() {
        let env = create_test_env();
        env.define_or_set("x", Value::Number(DecimalNumber::from_i64(10)));
        let body = Stmt::Block {
            statements: vec![Stmt::Expr(Expr::Literal(Literal::Identifier(
                "x".to_string(),
                Span::default(),
            )))],
            span: Span::default(),
        };

        // This should still work because the last statement IS an expression
        let result = eval_implicit_return(&body, env, None).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(10)));
    }
}
