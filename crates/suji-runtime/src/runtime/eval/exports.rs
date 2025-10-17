use super::super::env::Env;
use super::super::value::{MapKey, RuntimeError, Value};
use super::eval_expr;
use indexmap::IndexMap;
use std::rc::Rc;
use suji_ast::ast::{ExportBody, ExportSpec};

#[cfg(test)]
use suji_ast::Span;
#[cfg(test)]
use suji_ast::ast::{Expr, Literal};

/// Result of evaluating an export statement
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// The module map created from the export
    pub module: Value,
}

/// Evaluate an export statement and create a module map
pub fn eval_export(spec: &ExportSpec, env: Rc<Env>) -> Result<ExportResult, RuntimeError> {
    let mut module_map = IndexMap::new();

    // Evaluate each exported expression
    for (name, expr) in &spec.items {
        let value = eval_expr(expr, env.clone())?;
        let key = MapKey::String(name.clone());
        module_map.insert(key, value);
    }

    let module = Value::Map(module_map);
    Ok(ExportResult { module })
}

/// Evaluate an export body and return the exported value.
/// - For Map bodies, returns Value::Map of evaluated items
/// - For Expr bodies, evaluates and returns the expression value directly
pub fn eval_export_body(body: &ExportBody, env: Rc<Env>) -> Result<Value, RuntimeError> {
    match body {
        ExportBody::Map(spec) => {
            // Reuse existing map export evaluation
            let result = eval_export(spec, env)?;
            Ok(result.module)
        }
        ExportBody::Expr(expr) => {
            let value = eval_expr(expr, env)?;
            Ok(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::value::DecimalNumber;

    fn create_test_env() -> Rc<Env> {
        Rc::new(Env::new())
    }

    fn create_test_export_spec() -> ExportSpec {
        ExportSpec {
            items: vec![
                (
                    "CONSTANT".to_string(),
                    Expr::Literal(Literal::Number("42".to_string(), Span::default())),
                ),
                (
                    "message".to_string(),
                    Expr::Literal(Literal::StringTemplate(
                        vec![suji_ast::ast::StringPart::Text("Hello, world!".to_string())],
                        Span::default(),
                    )),
                ),
                (
                    "flag".to_string(),
                    Expr::Literal(Literal::Boolean(true, Span::default())),
                ),
            ],
            span: Span::default(),
        }
    }

    #[test]
    fn test_export_evaluation() {
        let env = create_test_env();
        let export_spec = create_test_export_spec();

        let result = eval_export(&export_spec, env);
        assert!(result.is_ok());

        let export_result = result.unwrap();
        if let Value::Map(map) = export_result.module {
            // Check that all exported items are present
            assert_eq!(
                map.get(&MapKey::String("CONSTANT".to_string())),
                Some(&Value::Number(DecimalNumber::from_i64(42)))
            );
            assert_eq!(
                map.get(&MapKey::String("message".to_string())),
                Some(&Value::String("Hello, world!".to_string()))
            );
            assert_eq!(
                map.get(&MapKey::String("flag".to_string())),
                Some(&Value::Boolean(true))
            );
        } else {
            panic!("Export result should be a map");
        }
    }

    #[test]
    fn test_export_with_variable_reference() {
        let env = create_test_env();

        // Set up some variables in the environment
        env.define_or_set("x", Value::Number(DecimalNumber::from_i64(10)));
        env.define_or_set("name", Value::String("test".to_string()));

        let export_spec = ExportSpec {
            items: vec![
                (
                    "value".to_string(),
                    Expr::Literal(Literal::Identifier("x".to_string(), Span::default())),
                ),
                (
                    "title".to_string(),
                    Expr::Literal(Literal::Identifier("name".to_string(), Span::default())),
                ),
            ],
            span: Span::default(),
        };

        let result = eval_export(&export_spec, env);
        assert!(result.is_ok());

        let export_result = result.unwrap();
        if let Value::Map(map) = export_result.module {
            assert_eq!(
                map.get(&MapKey::String("value".to_string())),
                Some(&Value::Number(DecimalNumber::from_i64(10)))
            );
            assert_eq!(
                map.get(&MapKey::String("title".to_string())),
                Some(&Value::String("test".to_string()))
            );
        } else {
            panic!("Export result should be a map");
        }
    }

    #[test]
    fn test_empty_export() {
        let env = create_test_env();
        let export_spec = ExportSpec {
            items: vec![],
            span: Span::default(),
        };

        let result = eval_export(&export_spec, env);
        assert!(result.is_ok());

        let export_result = result.unwrap();
        if let Value::Map(map) = export_result.module {
            assert!(map.is_empty());
        } else {
            panic!("Export result should be a map");
        }
    }

    #[test]
    fn test_export_undefined_variable() {
        let env = create_test_env();
        let export_spec = ExportSpec {
            items: vec![(
                "undefined".to_string(),
                Expr::Literal(Literal::Identifier(
                    "nonexistent".to_string(),
                    Span::default(),
                )),
            )],
            span: Span::default(),
        };

        let result = eval_export(&export_spec, env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Undefined variable")
        );
    }

    #[test]
    fn test_eval_export_body_expr_literal_number() {
        let env = create_test_env();
        let body = ExportBody::Expr(Expr::Literal(Literal::Number(
            "42".to_string(),
            Span::default(),
        )));

        let value = eval_export_body(&body, env).expect("export body evaluation should succeed");
        assert_eq!(value, Value::Number(DecimalNumber::from_i64(42)));
    }

    #[test]
    fn test_eval_export_body_expr_identifier() {
        let env = create_test_env();
        env.define_or_set("x", Value::Number(DecimalNumber::from_i64(7)));
        let body = ExportBody::Expr(Expr::Literal(Literal::Identifier(
            "x".to_string(),
            Span::default(),
        )));

        let value = eval_export_body(&body, env).expect("export body evaluation should succeed");
        assert_eq!(value, Value::Number(DecimalNumber::from_i64(7)));
    }

    #[test]
    fn test_eval_export_body_expr_undefined_identifier_errors() {
        let env = create_test_env();
        let body = ExportBody::Expr(Expr::Literal(Literal::Identifier(
            "missing".to_string(),
            Span::default(),
        )));

        let err = eval_export_body(&body, env).unwrap_err();
        assert!(err.to_string().contains("Undefined variable"));
    }

    #[test]
    fn test_eval_export_body_map_returns_map() {
        let env = create_test_env();
        let spec = ExportSpec {
            items: vec![(
                "a".to_string(),
                Expr::Literal(Literal::Number("1".to_string(), Span::default())),
            )],
            span: Span::default(),
        };
        let body = ExportBody::Map(spec);

        let value = eval_export_body(&body, env).expect("map export should succeed");
        match value {
            Value::Map(map) => {
                assert_eq!(
                    map.get(&MapKey::String("a".to_string())),
                    Some(&Value::Number(DecimalNumber::from_i64(1)))
                );
            }
            _ => panic!("expected a map value from map export body"),
        }
    }
}
