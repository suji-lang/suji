use super::EvalResult;
use crate::runtime::env::Env;
use crate::runtime::value::{DecimalNumber, RuntimeError, Value};
use std::rc::Rc;
use suji_ast::ast::{Expr, Literal};

/// Evaluate postfix increment (x++)
pub fn eval_postfix_increment(target: &Expr, env: Rc<Env>) -> EvalResult<Value> {
    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            let current_value = env.get(name)?;
            match current_value {
                Value::Number(n) => {
                    let incremented = n.add(&DecimalNumber::from_i64(1));
                    let new_value = Value::Number(incremented.clone());
                    env.set_existing(name, new_value)?;
                    Ok(Value::Number(incremented)) // Return the incremented value
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
                    let decremented = n.sub(&DecimalNumber::from_i64(1));
                    let new_value = Value::Number(decremented.clone());
                    env.set_existing(name, new_value)?;
                    Ok(Value::Number(decremented)) // Return the decremented value
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
    use crate::runtime::builtins::setup_global_env;
    use crate::runtime::env::Env;
    use suji_ast::Span;
    use suji_ast::ast::{Expr, Literal};

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        setup_global_env(&env);
        env
    }

    #[test]
    fn test_postfix_increment() {
        let env = create_test_env();
        env.define_or_set("x", Value::Number(DecimalNumber::from_i64(5)));

        let expr = Expr::Literal(Literal::Identifier("x".to_string(), Span::default()));
        let result = eval_postfix_increment(&expr, env.clone()).unwrap();

        assert_eq!(result, Value::Number(DecimalNumber::from_i64(6)));
        assert_eq!(
            env.get("x").unwrap(),
            Value::Number(DecimalNumber::from_i64(6))
        );
    }

    #[test]
    fn test_postfix_decrement() {
        let env = create_test_env();
        env.define_or_set("y", Value::Number(DecimalNumber::from_i64(10)));

        let expr = Expr::Literal(Literal::Identifier("y".to_string(), Span::default()));
        let result = eval_postfix_decrement(&expr, env.clone()).unwrap();

        assert_eq!(result, Value::Number(DecimalNumber::from_i64(9)));
        assert_eq!(
            env.get("y").unwrap(),
            Value::Number(DecimalNumber::from_i64(9))
        );
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
