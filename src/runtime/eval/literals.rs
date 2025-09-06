use super::{EvalResult, eval_expr};
use crate::ast::Literal;
use crate::runtime::env::Env;
use crate::runtime::string_template::evaluate_string_template;
use crate::runtime::value::{RuntimeError, Value};
use regex::Regex;
use std::rc::Rc;

/// Evaluate a literal expression
pub fn eval_literal(literal: &Literal, env: Rc<Env>) -> EvalResult<Value> {
    match literal {
        Literal::Number(n, _) => Ok(Value::Number(*n)),
        Literal::Boolean(b, _) => Ok(Value::Boolean(*b)),
        Literal::Identifier(name, _) => env.get(name),
        Literal::StringTemplate(parts, _) => {
            let result = evaluate_string_template(parts, |expr| eval_expr(expr, env.clone()))?;
            Ok(Value::String(result))
        }
        Literal::List(elements, _) => {
            let mut values = Vec::new();
            for element in elements {
                values.push(eval_expr(element, env.clone())?);
            }
            Ok(Value::List(values))
        }
        Literal::Map(pairs, _) => {
            let mut map = indexmap::IndexMap::new();
            for (key_expr, value_expr) in pairs {
                let key = eval_expr(key_expr, env.clone())?;
                let value = eval_expr(value_expr, env.clone())?;
                let map_key = key.try_into_map_key()?;
                map.insert(map_key, value);
            }
            Ok(Value::Map(map))
        }
        Literal::Tuple(elements, _) => {
            let mut values = Vec::new();
            for element in elements {
                values.push(eval_expr(element, env.clone())?);
            }
            Ok(Value::Tuple(values))
        }
        Literal::RegexLiteral(pattern, _) => {
            let regex = Regex::new(pattern).map_err(|err| RuntimeError::RegexError {
                message: format!("Invalid regex pattern '{}': {}", pattern, err),
            })?;
            Ok(Value::Regex(regex))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::runtime::env::Env;
    use crate::token::Span;

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        crate::runtime::builtins::setup_global_env(&env);
        env
    }

    #[test]
    fn test_number_literal() {
        let env = create_test_env();
        let literal = Literal::Number(42.0, Span::default());
        let result = eval_literal(&literal, env).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_boolean_literal() {
        let env = create_test_env();
        let literal = Literal::Boolean(true, Span::default());
        let result = eval_literal(&literal, env).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_identifier_lookup() {
        let env = create_test_env();
        env.define_or_set("x", Value::Number(100.0));

        let literal = Literal::Identifier("x".to_string(), Span::default());
        let result = eval_literal(&literal, env).unwrap();
        assert_eq!(result, Value::Number(100.0));
    }
}
