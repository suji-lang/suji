use super::value::{RuntimeError, Value};
use crate::ast::StringPart;

/// Evaluate a string template by processing its parts
pub fn evaluate_string_template<F>(
    parts: &[StringPart],
    eval_expr: F,
) -> Result<String, RuntimeError>
where
    F: Fn(&crate::ast::Expr) -> Result<Value, RuntimeError>,
{
    let mut result = String::new();

    for part in parts {
        match part {
            StringPart::Text(text) => {
                result.push_str(text);
            }
            StringPart::Expr(expr) => {
                let value = eval_expr(expr)?;
                result.push_str(&value.to_string());
            }
        }
    }

    Ok(result)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::token::Span;

    fn dummy_evaluator(expr: &crate::ast::Expr) -> Result<Value, RuntimeError> {
        // Simple evaluator for testing
        match expr {
            Expr::Literal(Literal::Number(n, _)) => Ok(Value::Number(*n)),
            Expr::Literal(Literal::StringTemplate(parts, _)) => {
                // For test purposes, if it's a simple string template, return the first text part
                if parts.len() == 1
                    && let StringPart::Text(s) = &parts[0]
                {
                    return Ok(Value::String(s.clone()));
                }
                Ok(Value::String("${...}".to_string()))
            }
            Expr::Literal(Literal::Boolean(b, _)) => Ok(Value::Boolean(*b)),
            Expr::Literal(Literal::Identifier(name, _)) => {
                // Return a dummy value based on name
                match name.as_str() {
                    "name" => Ok(Value::String("Alice".to_string())),
                    "age" => Ok(Value::Number(30.0)),
                    _ => Err(RuntimeError::UndefinedVariable { name: name.clone() }),
                }
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: "Unsupported expression in test".to_string(),
            }),
        }
    }

    #[test]
    fn test_text_only_template() {
        let parts = vec![StringPart::Text("Hello, world!".to_string())];
        let result = evaluate_string_template(&parts, dummy_evaluator).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_simple_interpolation() {
        let parts = vec![
            StringPart::Text("Hello, ".to_string()),
            StringPart::Expr(Expr::Literal(Literal::Identifier(
                "name".to_string(),
                Span::default(),
            ))),
            StringPart::Text("!".to_string()),
        ];

        let result = evaluate_string_template(&parts, dummy_evaluator).unwrap();
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_multiple_interpolations() {
        let parts = vec![
            StringPart::Text("Name: ".to_string()),
            StringPart::Expr(Expr::Literal(Literal::Identifier(
                "name".to_string(),
                Span::default(),
            ))),
            StringPart::Text(", Age: ".to_string()),
            StringPart::Expr(Expr::Literal(Literal::Identifier(
                "age".to_string(),
                Span::default(),
            ))),
        ];

        let result = evaluate_string_template(&parts, dummy_evaluator).unwrap();
        assert_eq!(result, "Name: Alice, Age: 30");
    }

    #[test]
    fn test_number_interpolation() {
        let parts = vec![
            StringPart::Text("The answer is ".to_string()),
            StringPart::Expr(Expr::Literal(Literal::Number(42.0, Span::default()))),
        ];

        let result = evaluate_string_template(&parts, dummy_evaluator).unwrap();
        assert_eq!(result, "The answer is 42");
    }

    #[test]
    fn test_boolean_interpolation() {
        let parts = vec![
            StringPart::Text("Is it true? ".to_string()),
            StringPart::Expr(Expr::Literal(Literal::Boolean(true, Span::default()))),
        ];

        let result = evaluate_string_template(&parts, dummy_evaluator).unwrap();
        assert_eq!(result, "Is it true? true");
    }

    #[test]
    fn test_evaluation_error_propagation() {
        let parts = vec![
            StringPart::Text("Hello, ".to_string()),
            StringPart::Expr(Expr::Literal(Literal::Identifier(
                "unknown".to_string(),
                Span::default(),
            ))),
        ];

        let result = evaluate_string_template(&parts, dummy_evaluator);
        assert!(matches!(
            result,
            Err(RuntimeError::UndefinedVariable { .. })
        ));
    }


    #[test]
    fn test_empty_template() {
        let parts = vec![];
        let result = evaluate_string_template(&parts, dummy_evaluator).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_expression_only_template() {
        let parts = vec![StringPart::Expr(Expr::Literal(Literal::Identifier(
            "name".to_string(),
            Span::default(),
        )))];

        let result = evaluate_string_template(&parts, dummy_evaluator).unwrap();
        assert_eq!(result, "Alice");
    }
}
