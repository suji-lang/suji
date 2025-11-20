use crate::eval::{EvalResult, eval_expr};
use std::rc::Rc;
use suji_ast::Expr;
use suji_runtime::ModuleRegistry;
use suji_values::{Env, Value};

/// Evaluate a slice of expressions into a vector of values.
pub fn evaluate_exprs(
    exprs: &[Expr],
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Vec<Value>> {
    let mut values = Vec::new();
    for expr in exprs {
        values.push(eval_expr(expr, env.clone(), registry)?);
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_ast::{Expr, Literal};
    use suji_lexer::Span;
    use suji_runtime::setup_global_env;
    use suji_values::DecimalNumber;

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        setup_global_env(&env);
        env
    }

    #[test]
    fn test_evaluate_exprs_empty() {
        let env = create_test_env();
        let exprs = vec![];
        let result = evaluate_exprs(&exprs, env, None).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_evaluate_exprs_single() {
        let env = create_test_env();
        let exprs = vec![Expr::Literal(Literal::Number(
            "42".to_string(),
            Span::default(),
        ))];
        let result = evaluate_exprs(&exprs, env, None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Value::Number(DecimalNumber::from_i64(42)));
    }

    #[test]
    fn test_evaluate_exprs_multiple() {
        let env = create_test_env();
        let exprs = vec![
            Expr::Literal(Literal::Number("1".to_string(), Span::default())),
            Expr::Literal(Literal::Number("2".to_string(), Span::default())),
            Expr::Literal(Literal::Number("3".to_string(), Span::default())),
        ];
        let result = evaluate_exprs(&exprs, env, None).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Value::Number(DecimalNumber::from_i64(1)));
        assert_eq!(result[1], Value::Number(DecimalNumber::from_i64(2)));
        assert_eq!(result[2], Value::Number(DecimalNumber::from_i64(3)));
    }

    #[test]
    fn test_evaluate_exprs_with_variables() {
        let env = create_test_env();
        env.define_or_set("x", Value::Number(DecimalNumber::from_i64(10)));
        env.define_or_set("y", Value::Number(DecimalNumber::from_i64(20)));

        let exprs = vec![
            Expr::Literal(Literal::Identifier("x".to_string(), Span::default())),
            Expr::Literal(Literal::Identifier("y".to_string(), Span::default())),
        ];
        let result = evaluate_exprs(&exprs, env, None).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Value::Number(DecimalNumber::from_i64(10)));
        assert_eq!(result[1], Value::Number(DecimalNumber::from_i64(20)));
    }

    #[test]
    fn test_evaluate_exprs_error_propagation() {
        let env = create_test_env();
        // Use an undefined variable to trigger an error
        let exprs = vec![Expr::Literal(Literal::Identifier(
            "undefined_var".to_string(),
            Span::default(),
        ))];
        let result = evaluate_exprs(&exprs, env, None);
        assert!(result.is_err());
    }
}
