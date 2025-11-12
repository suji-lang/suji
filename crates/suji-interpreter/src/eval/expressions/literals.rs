use super::{EvalResult, eval_expr};
use regex::Regex;
use std::rc::Rc;
use suji_ast::ast::Literal;
use suji_runtime::ModuleRegistry;
use suji_values::Env;
use suji_values::evaluate_string_template;
use suji_values::{DecimalNumber, RuntimeError, Value};

/// Evaluate a literal expression
pub fn eval_literal(
    literal: &Literal,
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Value> {
    match literal {
        Literal::Number(n, _) => {
            let decimal =
                DecimalNumber::parse(n).map_err(|err| RuntimeError::InvalidNumberConversion {
                    message: format!("Invalid number '{}': {}", n, err),
                })?;
            Ok(Value::Number(decimal))
        }
        Literal::Boolean(b, _) => Ok(Value::Boolean(*b)),
        Literal::Identifier(name, _) => env.get(name),
        Literal::StringTemplate(parts, _) => {
            let result =
                evaluate_string_template(parts, |expr| eval_expr(expr, env.clone(), registry))?;
            Ok(Value::String(result))
        }
        Literal::List(elements, _) => {
            let mut values = Vec::new();
            for element in elements {
                values.push(eval_expr(element, env.clone(), registry)?);
            }
            Ok(Value::List(values))
        }
        Literal::Map(pairs, _) => {
            let mut map = indexmap::IndexMap::new();
            for (key_expr, value_expr) in pairs {
                let key = eval_expr(key_expr, env.clone(), registry)?;
                let value = eval_expr(value_expr, env.clone(), registry)?;
                let map_key = key.try_into_map_key()?;
                map.insert(map_key, value);
            }
            Ok(Value::Map(map))
        }
        Literal::Tuple(elements, _) => {
            let mut values = Vec::new();
            for element in elements {
                values.push(eval_expr(element, env.clone(), registry)?);
            }
            Ok(Value::Tuple(values))
        }
        Literal::RegexLiteral(pattern, _) => {
            let regex = Regex::new(pattern).map_err(|err| RuntimeError::RegexError {
                message: format!("Invalid regex pattern '{}': {}", pattern, err),
            })?;
            Ok(Value::Regex(regex))
        }
        Literal::Nil(_) => Ok(Value::Nil),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_ast::Span;
    use suji_ast::ast::Literal;
    use suji_runtime::setup_global_env;
    use suji_values::Env;

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        setup_global_env(&env);
        env
    }

    #[test]
    fn test_number_literal() {
        let env = create_test_env();
        let literal = Literal::Number("42".to_string(), Span::default());
        let result = eval_literal(&literal, env, None).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(42)));
    }

    #[test]
    fn test_boolean_literal() {
        let env = create_test_env();
        let literal = Literal::Boolean(true, Span::default());
        let result = eval_literal(&literal, env, None).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_identifier_lookup() {
        let env = create_test_env();
        env.define_or_set("x", Value::Number(DecimalNumber::from_i64(100)));

        let literal = Literal::Identifier("x".to_string(), Span::default());
        let result = eval_literal(&literal, env, None).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(100)));
    }

    #[test]
    fn test_nil_literal() {
        let env = create_test_env();
        let literal = Literal::Nil(Span::default());
        let result = eval_literal(&literal, env, None).unwrap();
        assert_eq!(result, Value::Nil);
    }
}
