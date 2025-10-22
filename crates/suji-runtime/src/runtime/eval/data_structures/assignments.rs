use super::super::super::eval::{EvalResult, eval_expr};
use crate::runtime::env::Env;
use crate::runtime::value::{MapKey, RuntimeError, Value};
use std::rc::Rc;
use suji_ast::ast::{Expr, Literal};

/// Evaluate assignment expression
pub fn eval_assignment(target: &Expr, value: &Expr, env: Rc<Env>) -> EvalResult<Value> {
    let rhs_value = eval_expr(value, env.clone())?;
    eval_complex_assignment(target, rhs_value, env)
}

/// Evaluate complex assignment with support for nested structures
pub fn eval_complex_assignment(target: &Expr, value: Value, env: Rc<Env>) -> EvalResult<Value> {
    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            // Simple variable assignment
            env.set_existing(name, value.clone())?;
            Ok(value)
        }
        Expr::Index { target, index, .. } => {
            // List/map element assignment: target[index] = value
            eval_index_assignment(target, index, value, env)
        }
        Expr::MapAccessByName { target, key, .. } => {
            // Map key assignment: target:key = value
            eval_map_key_assignment(target, key, value, env)
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
    let index_value = eval_expr(index, env.clone())?;

    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            let target_value = env.get(name)?;
            let updated_value = update_index_value(&target_value, &index_value, &value)?;
            env.set_existing(name, updated_value)?;
            Ok(value)
        }
        Expr::Index {
            target: nested_target,
            index: nested_index,
            ..
        } => {
            // Nested indexing: target[index1][index2] = value
            let nested_target_value = eval_expr(nested_target, env.clone())?;
            let nested_index_value = eval_expr(nested_index, env.clone())?;

            // Get the nested item
            let nested_item = get_index_value(&nested_target_value, &nested_index_value)?;

            // Update the nested item
            let updated_nested_item = update_index_value(&nested_item, &index_value, &value)?;

            // Update the parent structure
            let updated_parent = update_index_value(
                &nested_target_value,
                &nested_index_value,
                &updated_nested_item,
            )?;

            // Update the variable
            if let Expr::Literal(Literal::Identifier(name, _)) = &**nested_target {
                env.set_existing(name, updated_parent)?;
            } else {
                return Err(RuntimeError::InvalidOperation {
                    message: "Complex nested assignment targets not yet supported".to_string(),
                });
            }

            Ok(value)
        }
        Expr::MapAccessByName {
            target: nested_target,
            key: nested_key,
            ..
        } => {
            // Handle different types of nested targets
            match &**nested_target {
                Expr::Literal(Literal::Identifier(name, _)) => {
                    // Mixed nested access: target:key[index] = value
                    let nested_target_value = eval_expr(nested_target, env.clone())?;

                    // Get the nested item
                    let nested_item = get_map_access_value(&nested_target_value, nested_key)?;

                    // Update the nested item
                    let updated_nested_item =
                        update_index_value(&nested_item, &index_value, &value)?;

                    // Update the parent structure
                    let updated_parent = update_map_access_value(
                        &nested_target_value,
                        nested_key,
                        &updated_nested_item,
                    )?;

                    // Update the variable
                    env.set_existing(name, updated_parent)?;
                    Ok(value)
                }
                Expr::MapAccessByName {
                    target: grandparent_target,
                    key: grandparent_key,
                    ..
                } => {
                    // Deeply nested access: target:key1:key2[index] = value
                    let grandparent_value = eval_expr(grandparent_target, env.clone())?;

                    // Get the intermediate map
                    let intermediate_map =
                        get_map_access_value(&grandparent_value, grandparent_key)?;

                    // Get the final nested item
                    let nested_item = get_map_access_value(&intermediate_map, nested_key)?;

                    // Update the nested item
                    let updated_nested_item =
                        update_index_value(&nested_item, &index_value, &value)?;

                    // Update the intermediate map
                    let updated_intermediate_map = update_map_access_value(
                        &intermediate_map,
                        nested_key,
                        &updated_nested_item,
                    )?;

                    // Update the grandparent structure
                    let updated_grandparent = update_map_access_value(
                        &grandparent_value,
                        grandparent_key,
                        &updated_intermediate_map,
                    )?;

                    // Update the variable
                    if let Expr::Literal(Literal::Identifier(name, _)) = &**grandparent_target {
                        env.set_existing(name, updated_grandparent)?;
                    } else {
                        return Err(RuntimeError::InvalidOperation {
                            message: "Complex nested assignment targets not yet supported"
                                .to_string(),
                        });
                    }

                    Ok(value)
                }
                _ => Err(RuntimeError::InvalidOperation {
                    message: "Complex nested assignment targets not yet supported".to_string(),
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
            let updated_value = update_map_access_value(&target_value, key, &value)?;
            env.set_existing(name, updated_value)?;
            Ok(value)
        }
        Expr::Index {
            target: nested_target,
            index: nested_index,
            ..
        } => {
            // Mixed nested access: target[index]:key = value
            let nested_target_value = eval_expr(nested_target, env.clone())?;
            let nested_index_value = eval_expr(nested_index, env.clone())?;

            // Get the nested item
            let nested_item = get_index_value(&nested_target_value, &nested_index_value)?;

            // Update the nested item
            let updated_nested_item = update_map_access_value(&nested_item, key, &value)?;

            // Update the parent structure
            let updated_parent = update_index_value(
                &nested_target_value,
                &nested_index_value,
                &updated_nested_item,
            )?;

            // Update the variable
            if let Expr::Literal(Literal::Identifier(name, _)) = &**nested_target {
                env.set_existing(name, updated_parent)?;
            } else {
                return Err(RuntimeError::InvalidOperation {
                    message: "Complex nested assignment targets not yet supported".to_string(),
                });
            }

            Ok(value)
        }
        Expr::MapAccessByName {
            target: nested_target,
            key: nested_key,
            ..
        } => {
            // Handle different types of nested targets
            match &**nested_target {
                Expr::Literal(Literal::Identifier(name, _)) => {
                    // Nested map access: target:key1:key2 = value
                    let nested_target_value = eval_expr(nested_target, env.clone())?;

                    // Get the nested item
                    let nested_item = get_map_access_value(&nested_target_value, nested_key)?;

                    // Update the nested item
                    let updated_nested_item = update_map_access_value(&nested_item, key, &value)?;

                    // Update the parent structure
                    let updated_parent = update_map_access_value(
                        &nested_target_value,
                        nested_key,
                        &updated_nested_item,
                    )?;

                    // Update the variable
                    env.set_existing(name, updated_parent)?;
                    Ok(value)
                }
                Expr::MapAccessByName {
                    target: grandparent_target,
                    key: grandparent_key,
                    ..
                } => {
                    // Deeply nested access: target:key1:key2:key3 = value
                    let grandparent_value = eval_expr(grandparent_target, env.clone())?;

                    // Get the intermediate map
                    let intermediate_map =
                        get_map_access_value(&grandparent_value, grandparent_key)?;

                    // Get the final nested item
                    let nested_item = get_map_access_value(&intermediate_map, nested_key)?;

                    // Update the nested item
                    let updated_nested_item = update_map_access_value(&nested_item, key, &value)?;

                    // Update the intermediate map
                    let updated_intermediate_map = update_map_access_value(
                        &intermediate_map,
                        nested_key,
                        &updated_nested_item,
                    )?;

                    // Update the grandparent structure
                    let updated_grandparent = update_map_access_value(
                        &grandparent_value,
                        grandparent_key,
                        &updated_intermediate_map,
                    )?;

                    // Update the variable
                    if let Expr::Literal(Literal::Identifier(name, _)) = &**grandparent_target {
                        env.set_existing(name, updated_grandparent)?;
                    } else {
                        return Err(RuntimeError::InvalidOperation {
                            message: "Complex nested assignment targets not yet supported"
                                .to_string(),
                        });
                    }

                    Ok(value)
                }
                _ => Err(RuntimeError::InvalidOperation {
                    message: "Complex nested assignment targets not yet supported".to_string(),
                }),
            }
        }
        _ => Err(RuntimeError::InvalidOperation {
            message: "Complex assignment targets not yet supported".to_string(),
        }),
    }
}

/// Helper function to get a value by index
fn get_index_value(target: &Value, index: &Value) -> EvalResult<Value> {
    match (target, index) {
        (Value::List(items), Value::Number(n)) => {
            if !n.is_integer() {
                return Err(RuntimeError::TypeError {
                    message: "List index must be an integer".to_string(),
                });
            }
            let idx = n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
                message: "Index out of range".to_string(),
            })?;
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

            Ok(items[normalized_idx as usize].clone())
        }
        (Value::Map(map), key) => {
            let map_key = key.clone().try_into_map_key()?;
            map.get(&map_key)
                .cloned()
                .ok_or_else(|| RuntimeError::IndexOutOfBounds {
                    message: "Key not found in map".to_string(),
                })
        }
        (Value::EnvMap(env_proxy), key) => {
            let key_str = match key {
                Value::String(s) => s,
                _ => {
                    return Err(RuntimeError::TypeError {
                        message: "Environment keys must be strings".to_string(),
                    });
                }
            };
            match env_proxy.get(key_str) {
                Some(value) => Ok(Value::String(value)),
                None => Err(RuntimeError::KeyNotFound {
                    message: format!("Environment variable not found: {}", key_str),
                }),
            }
        }
        _ => Err(RuntimeError::TypeError {
            message: format!(
                "Cannot index {} with {}",
                target.type_name(),
                index.type_name()
            ),
        }),
    }
}

/// Helper function to get a value by map key
fn get_map_access_value(target: &Value, key: &str) -> EvalResult<Value> {
    match target {
        Value::Map(map) => {
            let map_key = MapKey::String(key.to_string());
            map.get(&map_key)
                .cloned()
                .ok_or_else(|| RuntimeError::IndexOutOfBounds {
                    message: format!("Key '{}' not found in map", key),
                })
        }
        Value::EnvMap(env_proxy) => match env_proxy.get(key) {
            Some(value) => Ok(Value::String(value)),
            None => Err(RuntimeError::KeyNotFound {
                message: format!("Environment variable not found: {}", key),
            }),
        },
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot access key '{}' on {}", key, target.type_name()),
        }),
    }
}

/// Helper function to update a value by index
fn update_index_value(target: &Value, index: &Value, value: &Value) -> EvalResult<Value> {
    match (target, index) {
        (Value::List(items), Value::Number(n)) => {
            if !n.is_integer() {
                return Err(RuntimeError::TypeError {
                    message: "List index must be an integer".to_string(),
                });
            }
            let idx = n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
                message: "Index out of range".to_string(),
            })?;
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

            let mut updated_items = items.clone();
            updated_items[normalized_idx as usize] = value.clone();
            Ok(Value::List(updated_items))
        }
        (Value::Map(map), key) => {
            let map_key = key.clone().try_into_map_key()?;
            let mut updated_map = map.clone();
            updated_map.insert(map_key, value.clone());
            Ok(Value::Map(updated_map))
        }
        (Value::EnvMap(env_proxy), key) => {
            let key_str = match key {
                Value::String(s) => s,
                _ => {
                    return Err(RuntimeError::TypeError {
                        message: "Environment keys must be strings".to_string(),
                    });
                }
            };
            let value_str = value.to_string();
            env_proxy.set(key_str, &value_str)?;
            // Return the original EnvMap (since it's a proxy, changes are in-place)
            Ok(Value::EnvMap(env_proxy.clone()))
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot assign index on {}", target.type_name()),
        }),
    }
}

/// Helper function to update a value by map key
fn update_map_access_value(target: &Value, key: &str, value: &Value) -> EvalResult<Value> {
    match target {
        Value::Map(map) => {
            let map_key = MapKey::String(key.to_string());
            let mut updated_map = map.clone();
            updated_map.insert(map_key, value.clone());
            Ok(Value::Map(updated_map))
        }
        Value::EnvMap(env_proxy) => {
            let value_str = value.to_string();
            env_proxy.set(key, &value_str)?;
            // Return the original EnvMap (since it's a proxy, changes are in-place)
            Ok(Value::EnvMap(env_proxy.clone()))
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot assign key '{}' on {}", key, target.type_name()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::builtins::setup_global_env;
    use crate::runtime::env::Env;
    use crate::runtime::value::DecimalNumber;
    use suji_ast::Span;
    use suji_ast::ast::{Expr, Literal};

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        setup_global_env(&env);
        env
    }

    #[test]
    fn test_variable_assignment() {
        let env = create_test_env();
        env.define_or_set("x", Value::Number(DecimalNumber::from_i64(10)));

        let target = Expr::Literal(Literal::Identifier("x".to_string(), Span::default()));
        let value = Expr::Literal(Literal::Number("42".to_string(), Span::default()));

        let result = eval_assignment(&target, &value, env.clone()).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(42)));
        assert_eq!(
            env.get("x").unwrap(),
            Value::Number(DecimalNumber::from_i64(42))
        );
    }

    #[test]
    fn test_list_element_assignment() {
        let env = create_test_env();

        let list = Value::List(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3)),
        ]);
        env.define_or_set("my_list", list);

        let target = Expr::Index {
            target: Box::new(Expr::Literal(Literal::Identifier(
                "my_list".to_string(),
                Span::default(),
            ))),
            index: Box::new(Expr::Literal(Literal::Number(
                "1".to_string(),
                Span::default(),
            ))),
            span: Span::default(),
        };
        let value = Expr::Literal(Literal::Number("99".to_string(), Span::default()));

        let result = eval_assignment(&target, &value, env.clone()).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(99)));

        // Check that the list was updated
        let updated_list = env.get("my_list").unwrap();
        if let Value::List(items) = updated_list {
            assert_eq!(items[1], Value::Number(DecimalNumber::from_i64(99)));
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
        let value = Expr::Literal(Literal::Number("30".to_string(), Span::default()));

        let result = eval_assignment(&target, &value, env.clone()).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(30)));

        // Check that the map was updated
        let updated_map = env.get("person").unwrap();
        if let Value::Map(map) = updated_map {
            let age_key = MapKey::String("age".to_string());
            assert_eq!(
                map.get(&age_key),
                Some(&Value::Number(DecimalNumber::from_i64(30)))
            );
        } else {
            panic!("Expected map");
        }
    }
}
