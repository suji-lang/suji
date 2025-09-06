use super::{EvalResult, eval_expr};
use crate::ast::{Expr, Literal};
use crate::runtime::env::Env;
use crate::runtime::value::{MapKey, RuntimeError, Value};
use std::rc::Rc;

/// Evaluate assignment expression
pub fn eval_assignment(target: &Expr, value: &Expr, env: Rc<Env>) -> EvalResult<Value> {
    let rhs_value = eval_expr(value, env.clone())?;

    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            // Simple variable assignment
            env.set_existing(name, rhs_value.clone())?;
            Ok(rhs_value)
        }
        Expr::Index { target, index, .. } => {
            // List/map element assignment: target[index] = value
            eval_index_assignment(target, index, rhs_value, env)
        }
        Expr::MapAccessByName { target, key, .. } => {
            // Map key assignment: target:key = value
            eval_map_key_assignment(target, key, rhs_value, env)
        }
        _ => Err(RuntimeError::InvalidOperation {
            message: "Invalid assignment target".to_string(),
        }),
    }
}

/// Evaluate element assignment for lists and maps
pub fn eval_index_assignment(
    target: &Expr,
    index: &Expr,
    value: Value,
    env: Rc<Env>,
) -> EvalResult<Value> {
    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            let target_value = env.get(name)?;
            let index_value = eval_expr(index, env.clone())?;

            match target_value {
                Value::List(mut items) => {
                    let idx = match index_value {
                        Value::Number(n) => {
                            if n.fract() != 0.0 {
                                return Err(RuntimeError::TypeError {
                                    message: "List index must be an integer".to_string(),
                                });
                            }
                            n as i64
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "List index must be a number".to_string(),
                            });
                        }
                    };

                    // Handle negative indices
                    let normalized_idx = if idx < 0 {
                        items.len() as i64 + idx
                    } else {
                        idx
                    };

                    if normalized_idx < 0 || normalized_idx >= items.len() as i64 {
                        return Err(RuntimeError::IndexOutOfBounds {
                            message: format!(
                                "Index {} out of bounds for list of length {}",
                                idx,
                                items.len()
                            ),
                        });
                    }

                    items[normalized_idx as usize] = value.clone();
                    let updated_value = Value::List(items);
                    env.set_existing(name, updated_value)?;
                    Ok(value)
                }
                Value::Map(mut map) => {
                    let key = index_value.try_into_map_key()?;
                    map.insert(key, value.clone());
                    let updated_value = Value::Map(map);
                    env.set_existing(name, updated_value)?;
                    Ok(value)
                }
                _ => Err(RuntimeError::TypeError {
                    message: format!("Cannot assign index on {}", target_value.type_name()),
                }),
            }
        }
        _ => Err(RuntimeError::InvalidOperation {
            message: "Complex assignment targets not yet supported".to_string(),
        }),
    }
}

/// Evaluate map key assignment (map:key = value)
pub fn eval_map_key_assignment(
    target: &Expr,
    key: &str,
    value: Value,
    env: Rc<Env>,
) -> EvalResult<Value> {
    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            let target_value = env.get(name)?;

            match target_value {
                Value::Map(mut map) => {
                    let map_key = MapKey::String(key.to_string());
                    map.insert(map_key, value.clone());
                    let updated_value = Value::Map(map);
                    env.set_existing(name, updated_value)?;
                    Ok(value)
                }
                _ => Err(RuntimeError::TypeError {
                    message: format!(
                        "Cannot assign key '{}' on {}",
                        key,
                        target_value.type_name()
                    ),
                }),
            }
        }
        _ => Err(RuntimeError::InvalidOperation {
            message: "Complex assignment targets not yet supported".to_string(),
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
    fn test_variable_assignment() {
        let env = create_test_env();
        env.define_or_set("x", Value::Number(10.0));

        let target = Expr::Literal(Literal::Identifier("x".to_string(), Span::default()));
        let value = Expr::Literal(Literal::Number(42.0, Span::default()));

        let result = eval_assignment(&target, &value, env.clone()).unwrap();
        assert_eq!(result, Value::Number(42.0));
        assert_eq!(env.get("x").unwrap(), Value::Number(42.0));
    }

    #[test]
    fn test_list_element_assignment() {
        let env = create_test_env();

        let list = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        env.define_or_set("my_list", list);

        let target = Expr::Index {
            target: Box::new(Expr::Literal(Literal::Identifier(
                "my_list".to_string(),
                Span::default(),
            ))),
            index: Box::new(Expr::Literal(Literal::Number(1.0, Span::default()))),
            span: Span::default(),
        };
        let value = Expr::Literal(Literal::Number(99.0, Span::default()));

        let result = eval_assignment(&target, &value, env.clone()).unwrap();
        assert_eq!(result, Value::Number(99.0));

        // Check that the list was updated
        let updated_list = env.get("my_list").unwrap();
        if let Value::List(items) = updated_list {
            assert_eq!(items[1], Value::Number(99.0));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_map_key_assignment() {
        let env = create_test_env();

        let mut map = indexmap::IndexMap::new();
        map.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        env.define_or_set("person", Value::Map(map));

        let target = Expr::MapAccessByName {
            target: Box::new(Expr::Literal(Literal::Identifier(
                "person".to_string(),
                Span::default(),
            ))),
            key: "age".to_string(),
            span: Span::default(),
        };
        let value = Expr::Literal(Literal::Number(30.0, Span::default()));

        let result = eval_assignment(&target, &value, env.clone()).unwrap();
        assert_eq!(result, Value::Number(30.0));

        // Check that the map was updated
        let updated_map = env.get("person").unwrap();
        if let Value::Map(map) = updated_map {
            let age_key = MapKey::String("age".to_string());
            assert_eq!(map.get(&age_key), Some(&Value::Number(30.0)));
        } else {
            panic!("Expected map");
        }
    }
}
