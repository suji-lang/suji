use super::EvalResult;
use crate::ast::{Expr, Literal};
use crate::runtime::env::Env;
use crate::runtime::value::{RuntimeError, Value};
use std::rc::Rc;

/// Evaluate postfix increment (x++)
pub fn eval_postfix_increment(target: &Expr, env: Rc<Env>) -> EvalResult<Value> {
    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            let current_value = env.get(name)?;
            match current_value {
                Value::Number(n) => {
                    let new_value = Value::Number(n + 1.0);
                    env.set_existing(name, new_value)?;
                    Ok(Value::Number(n + 1.0)) // Return the incremented value
                }
                _ => Err(RuntimeError::TypeError {
                    message: format!("Cannot increment {}", current_value.type_name()),
                }),
            }
        }
        _ => Err(RuntimeError::InvalidOperation {
            message: "Increment (++) can only be applied to variables".to_string(),
        }),
    }
}

/// Evaluate postfix decrement (x--)
pub fn eval_postfix_decrement(target: &Expr, env: Rc<Env>) -> EvalResult<Value> {
    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            let current_value = env.get(name)?;
            match current_value {
                Value::Number(n) => {
                    let new_value = Value::Number(n - 1.0);
                    env.set_existing(name, new_value)?;
                    Ok(Value::Number(n - 1.0)) // Return the decremented value
                }
                _ => Err(RuntimeError::TypeError {
                    message: format!("Cannot decrement {}", current_value.type_name()),
                }),
            }
        }
        _ => Err(RuntimeError::InvalidOperation {
            message: "Decrement (--) can only be applied to variables".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::runtime::env::Env;
    use crate::token::Span;

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        crate::runtime::builtins::setup_global_env(&env);
        env
    }

    #[test]
    fn test_postfix_increment() {
        let env = create_test_env();
        env.define_or_set("x", Value::Number(5.0));

        let expr = Expr::Literal(Literal::Identifier("x".to_string(), Span::default()));
        let result = eval_postfix_increment(&expr, env.clone()).unwrap();

        assert_eq!(result, Value::Number(6.0));
        assert_eq!(env.get("x").unwrap(), Value::Number(6.0));
    }

    #[test]
    fn test_postfix_decrement() {
        let env = create_test_env();
        env.define_or_set("y", Value::Number(10.0));

        let expr = Expr::Literal(Literal::Identifier("y".to_string(), Span::default()));
        let result = eval_postfix_decrement(&expr, env.clone()).unwrap();

        assert_eq!(result, Value::Number(9.0));
        assert_eq!(env.get("y").unwrap(), Value::Number(9.0));
    }

    #[test]
    fn test_increment_non_number_error() {
        let env = create_test_env();
        env.define_or_set("s", Value::String("hello".to_string()));

        let expr = Expr::Literal(Literal::Identifier("s".to_string(), Span::default()));
        let result = eval_postfix_increment(&expr, env);

        assert!(result.is_err());
        if let Err(RuntimeError::TypeError { message }) = result {
            assert!(message.contains("Cannot increment"));
        } else {
            panic!("Expected TypeError");
        }
    }
}
