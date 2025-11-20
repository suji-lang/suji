use crate::eval::{EvalResult, eval_expr};
use std::rc::Rc;
use suji_ast::Expr;
use suji_runtime::ModuleRegistry;
use suji_values::{Env, RuntimeError, Value};

/// Evaluate and normalize slice indices (start and end) with clamping.
pub fn evaluate_slice_indices(
    start: Option<&Expr>,
    end: Option<&Expr>,
    len: i64,
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<(usize, usize)> {
    // Evaluate start index (default 0)
    let start_idx = match start {
        Some(expr) => {
            let val = eval_expr(expr, env.clone(), registry)?;
            match val {
                Value::Number(n) => {
                    if !n.is_integer() {
                        return Err(RuntimeError::TypeError {
                            message: "Slice start must be an integer".to_string(),
                        });
                    }
                    n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
                        message: "Index out of range".to_string(),
                    })?
                }
                _ => {
                    return Err(RuntimeError::TypeError {
                        message: "Slice start must be a number".to_string(),
                    });
                }
            }
        }
        None => 0,
    };

    // Evaluate end index (default len)
    let end_idx = match end {
        Some(expr) => {
            let val = eval_expr(expr, env, registry)?;
            match val {
                Value::Number(n) => {
                    if !n.is_integer() {
                        return Err(RuntimeError::TypeError {
                            message: "Slice end must be an integer".to_string(),
                        });
                    }
                    n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
                        message: "Index out of range".to_string(),
                    })?
                }
                _ => {
                    return Err(RuntimeError::TypeError {
                        message: "Slice end must be a number".to_string(),
                    });
                }
            }
        }
        None => len,
    };

    // Normalize negative indices
    let norm_start = if start_idx < 0 {
        len + start_idx
    } else {
        start_idx
    };
    let norm_end = if end_idx < 0 { len + end_idx } else { end_idx };

    // Clamp to bounds [0, len]
    let clamped_start = norm_start.max(0).min(len) as usize;
    let clamped_end = norm_end.max(0).min(len) as usize;

    Ok((clamped_start, clamped_end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_ast::{Expr, Literal};
    use suji_lexer::Span;
    use suji_runtime::setup_global_env;

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        setup_global_env(&env);
        env
    }

    #[test]
    fn test_evaluate_slice_indices_defaults() {
        let env = create_test_env();
        let len = 10i64;

        // Both None should default to (0, len)
        let (start, end) = evaluate_slice_indices(None, None, len, env, None).unwrap();
        assert_eq!(start, 0);
        assert_eq!(end, 10);
    }

    #[test]
    fn test_evaluate_slice_indices_positive() {
        let env = create_test_env();
        let len = 10i64;

        let start_expr = Expr::Literal(Literal::Number("2".to_string(), Span::default()));
        let end_expr = Expr::Literal(Literal::Number("7".to_string(), Span::default()));

        let (start, end) =
            evaluate_slice_indices(Some(&start_expr), Some(&end_expr), len, env, None).unwrap();
        assert_eq!(start, 2);
        assert_eq!(end, 7);
    }

    #[test]
    fn test_evaluate_slice_indices_negative() {
        let env = create_test_env();
        let len = 10i64;

        // -2 should normalize to 8, -1 should normalize to 9
        let start_expr = Expr::Literal(Literal::Number("-2".to_string(), Span::default()));
        let end_expr = Expr::Literal(Literal::Number("-1".to_string(), Span::default()));

        let (start, end) =
            evaluate_slice_indices(Some(&start_expr), Some(&end_expr), len, env, None).unwrap();
        assert_eq!(start, 8);
        assert_eq!(end, 9);
    }

    #[test]
    fn test_evaluate_slice_indices_clamping() {
        let env = create_test_env();
        let len = 10i64;

        // Start that normalizes to < 0 should clamp to 0, end > len should clamp to len
        // -15 normalizes to 10 + (-15) = -5, which clamps to 0
        let start_expr = Expr::Literal(Literal::Number("-15".to_string(), Span::default()));
        let end_expr = Expr::Literal(Literal::Number("15".to_string(), Span::default()));

        let (start, end) =
            evaluate_slice_indices(Some(&start_expr), Some(&end_expr), len, env, None).unwrap();
        assert_eq!(start, 0);
        assert_eq!(end, 10);
    }

    #[test]
    fn test_evaluate_slice_indices_mixed_defaults() {
        let env = create_test_env();
        let len = 10i64;

        // Start provided, end None
        let start_expr = Expr::Literal(Literal::Number("3".to_string(), Span::default()));
        let (start, end) =
            evaluate_slice_indices(Some(&start_expr), None, len, env.clone(), None).unwrap();
        assert_eq!(start, 3);
        assert_eq!(end, 10);

        // Start None, end provided
        let end_expr = Expr::Literal(Literal::Number("7".to_string(), Span::default()));
        let (start, end) = evaluate_slice_indices(None, Some(&end_expr), len, env, None).unwrap();
        assert_eq!(start, 0);
        assert_eq!(end, 7);
    }

    #[test]
    fn test_evaluate_slice_indices_non_integer_error() {
        let env = create_test_env();
        let len = 10i64;

        // Non-integer start should error
        let start_expr = Expr::Literal(Literal::Number("2.5".to_string(), Span::default()));
        let result = evaluate_slice_indices(Some(&start_expr), None, len, env.clone(), None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().without_span(),
            RuntimeError::TypeError { .. }
        ));

        // Non-integer end should error
        let end_expr = Expr::Literal(Literal::Number("3.7".to_string(), Span::default()));
        let result = evaluate_slice_indices(None, Some(&end_expr), len, env, None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().without_span(),
            RuntimeError::TypeError { .. }
        ));
    }

    #[test]
    fn test_evaluate_slice_indices_non_number_error() {
        let env = create_test_env();
        let len = 10i64;

        // Non-number start should error (using Boolean as non-number)
        let start_expr = Expr::Literal(Literal::Boolean(true, Span::default()));
        let result = evaluate_slice_indices(Some(&start_expr), None, len, env.clone(), None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().without_span(),
            RuntimeError::TypeError { .. }
        ));

        // Non-number end should error (using Boolean as non-number)
        let end_expr = Expr::Literal(Literal::Boolean(false, Span::default()));
        let result = evaluate_slice_indices(None, Some(&end_expr), len, env, None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().without_span(),
            RuntimeError::TypeError { .. }
        ));
    }
}
