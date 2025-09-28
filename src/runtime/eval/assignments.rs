use super::{EvalResult, eval_expr};
use crate::ast::{Expr, Literal};
use crate::runtime::env::Env;
use crate::runtime::value::{MapKey, RuntimeError, Value};
use std::rc::Rc;

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
            eval_deep_index_assignment(target, index, value, env)
        }
        Expr::MapAccessByName { target, key, .. } => {
            // Map key assignment: target:key = value
            eval_deep_map_assignment(target, key, value, env)
        }
        Expr::Destructure { elements, .. } => match value {
            Value::Tuple(items) => {
                if items.len() != elements.len() {
                    return Err(RuntimeError::DestructureArityMismatch {
                        expected: elements.len(),
                        actual: items.len(),
                    });
                }

                for (element_expr, element_value) in elements.iter().zip(items.iter()) {
                    match element_expr {
                        Expr::Literal(Literal::Identifier(name, _)) if name == "_" => {
                            // discard binding
                        }
                        Expr::Literal(Literal::Identifier(name, _)) => {
                            env.set_existing(name, element_value.clone())?;
                        }
                        _ => {
                            return Err(RuntimeError::DestructureInvalidTarget {
                                message: "Only identifiers or '_' are allowed".to_string(),
                            });
                        }
                    }
                }

                Ok(Value::Tuple(items))
            }
            _ => Err(RuntimeError::DestructureTypeError),
        },
        _ => Err(RuntimeError::InvalidOperation {
            message: "Invalid assignment target".to_string(),
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
            Ok(Value::EnvMap(env_proxy.clone()))
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot assign key '{}' on {}", key, target.type_name()),
        }),
    }
}

/// Evaluate deep index assignment with unlimited nesting support
/// Handles cases like: target[index1][index2][index3]...[indexN] = value
pub fn eval_deep_index_assignment(
    target: &Expr,
    index: &Expr,
    value: Value,
    env: Rc<Env>,
) -> EvalResult<Value> {
    let index_value = eval_expr(index, env.clone())?;

    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            // Base case: simple variable assignment
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
            // Recursive case: nested indexing
            let nested_target_value = eval_expr(nested_target, env.clone())?;
            let nested_index_value = eval_expr(nested_index, env.clone())?;

            // Get the nested item
            let nested_item = get_index_value(&nested_target_value, &nested_index_value)?;

            // Update the nested item
            let updated_nested_item = update_index_value(&nested_item, &index_value, &value)?;

            // Update the parent structure recursively
            let updated_parent = update_index_value(
                &nested_target_value,
                &nested_index_value,
                &updated_nested_item,
            )?;

            // Update the variable in the environment
            update_nested_structure_in_env(nested_target, updated_parent, env)?;
            Ok(value)
        }
        Expr::MapAccessByName {
            target: nested_target,
            key: nested_key,
            ..
        } => {
            // Mixed case: target:key[index] = value
            let nested_target_value = eval_expr(nested_target, env.clone())?;

            // Get the nested item
            let nested_item = get_map_access_value(&nested_target_value, nested_key)?;

            // Update the nested item
            let updated_nested_item = update_index_value(&nested_item, &index_value, &value)?;

            // Update the parent structure
            let updated_parent =
                update_map_access_value(&nested_target_value, nested_key, &updated_nested_item)?;

            // Update the variable in the environment
            update_nested_structure_in_env(nested_target, updated_parent, env)?;
            Ok(value)
        }
        _ => Err(RuntimeError::InvalidOperation {
            message: "Invalid nested assignment target".to_string(),
        }),
    }
}

/// Evaluate deep map assignment with unlimited nesting support
/// Handles cases like: target:key1:key2:key3:...:keyN = value
pub fn eval_deep_map_assignment(
    target: &Expr,
    key: &str,
    value: Value,
    env: Rc<Env>,
) -> EvalResult<Value> {
    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            // Base case: simple variable assignment
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
            // Mixed case: target[index]:key = value
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

            // Update the variable in the environment
            update_nested_structure_in_env(nested_target, updated_parent, env)?;
            Ok(value)
        }
        Expr::MapAccessByName {
            target: nested_target,
            key: nested_key,
            ..
        } => {
            // Recursive case: nested map access
            let nested_target_value = eval_expr(nested_target, env.clone())?;

            // Get the nested item
            let nested_item = get_map_access_value(&nested_target_value, nested_key)?;

            // Update the nested item
            let updated_nested_item = update_map_access_value(&nested_item, key, &value)?;

            // Update the parent structure
            let updated_parent =
                update_map_access_value(&nested_target_value, nested_key, &updated_nested_item)?;

            // Update the variable in the environment
            update_nested_structure_in_env(nested_target, updated_parent, env)?;
            Ok(value)
        }
        _ => Err(RuntimeError::InvalidOperation {
            message: "Invalid nested assignment target".to_string(),
        }),
    }
}

/// Update a nested structure in the environment
/// This function handles the recursive updating of nested structures
fn update_nested_structure_in_env(target: &Expr, new_value: Value, env: Rc<Env>) -> EvalResult<()> {
    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            // Base case: update the variable directly
            env.set_existing(name, new_value)?;
            Ok(())
        }
        Expr::Index {
            target: nested_target,
            index: nested_index,
            ..
        } => {
            // Recursive case: update nested indexing
            let nested_target_value = eval_expr(nested_target, env.clone())?;
            let nested_index_value = eval_expr(nested_index, env.clone())?;

            // Update the nested structure
            let updated_nested =
                update_index_value(&nested_target_value, &nested_index_value, &new_value)?;

            // Continue updating up the chain
            update_nested_structure_in_env(nested_target, updated_nested, env)
        }
        Expr::MapAccessByName {
            target: nested_target,
            key: nested_key,
            ..
        } => {
            // Recursive case: update nested map access
            let nested_target_value = eval_expr(nested_target, env.clone())?;

            // Update the nested structure
            let updated_nested =
                update_map_access_value(&nested_target_value, nested_key, &new_value)?;

            // Continue updating up the chain
            update_nested_structure_in_env(nested_target, updated_nested, env)
        }
        _ => Err(RuntimeError::InvalidOperation {
            message: "Cannot update nested structure".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::runtime::env::Env;
    use crate::runtime::value::DecimalNumber;
    use crate::token::Span;

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        crate::runtime::builtins::setup_global_env(&env);
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

    #[test]
    fn test_no_variable_shadowing() {
        let env = create_test_env();

        // Define variable in parent scope
        env.define_or_set("x", Value::Number(DecimalNumber::from_i64(1)));
        assert_eq!(
            env.get("x").unwrap(),
            Value::Number(DecimalNumber::from_i64(1))
        );

        // Create child environment
        let child_env = Rc::new(Env::new_child(env.clone()));

        // Assign to x in child scope - should modify parent variable
        let target = Expr::Literal(Literal::Identifier("x".to_string(), Span::default()));
        let value = Expr::Literal(Literal::Number("2".to_string(), Span::default()));

        let result = eval_assignment(&target, &value, child_env.clone()).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));

        // Check that parent variable was modified (no shadowing)
        assert_eq!(
            env.get("x").unwrap(),
            Value::Number(DecimalNumber::from_i64(2))
        );
        assert_eq!(
            child_env.get("x").unwrap(),
            Value::Number(DecimalNumber::from_i64(2))
        );

        // Check that no new variable was created in child scope
        assert_eq!(child_env.local_names().len(), 0);
    }

    #[test]
    fn test_complex_assignments() {
        let env = create_test_env();

        // Test nested list assignment: matrix[0][1] = 99
        let matrix = Value::List(vec![
            Value::List(vec![
                Value::Number(DecimalNumber::from_i64(1)),
                Value::Number(DecimalNumber::from_i64(2)),
            ]),
            Value::List(vec![
                Value::Number(DecimalNumber::from_i64(3)),
                Value::Number(DecimalNumber::from_i64(4)),
            ]),
        ]);
        env.define_or_set("matrix", matrix);

        let target = Expr::Index {
            target: Box::new(Expr::Index {
                target: Box::new(Expr::Literal(Literal::Identifier(
                    "matrix".to_string(),
                    Span::default(),
                ))),
                index: Box::new(Expr::Literal(Literal::Number(
                    "0".to_string(),
                    Span::default(),
                ))),
                span: Span::default(),
            }),
            index: Box::new(Expr::Literal(Literal::Number(
                "1".to_string(),
                Span::default(),
            ))),
            span: Span::default(),
        };
        let value = Expr::Literal(Literal::Number("99".to_string(), Span::default()));

        let result = eval_assignment(&target, &value, env.clone()).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(99)));

        // Check that the nested structure was updated
        let updated_matrix = env.get("matrix").unwrap();
        if let Value::List(rows) = updated_matrix {
            if let Value::List(first_row) = &rows[0] {
                assert_eq!(first_row[1], Value::Number(DecimalNumber::from_i64(99)));
            } else {
                panic!("Expected list in first row");
            }
        } else {
            panic!("Expected list");
        }

        // Test nested map assignment: config:user:age = 31
        let mut user_map = indexmap::IndexMap::new();
        user_map.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        user_map.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(30)),
        );

        let mut config_map = indexmap::IndexMap::new();
        config_map.insert(MapKey::String("user".to_string()), Value::Map(user_map));
        config_map.insert(
            MapKey::String("theme".to_string()),
            Value::String("dark".to_string()),
        );

        env.define_or_set("config", Value::Map(config_map));

        let target2 = Expr::MapAccessByName {
            target: Box::new(Expr::MapAccessByName {
                target: Box::new(Expr::Literal(Literal::Identifier(
                    "config".to_string(),
                    Span::default(),
                ))),
                key: "user".to_string(),
                span: Span::default(),
            }),
            key: "age".to_string(),
            span: Span::default(),
        };
        let value2 = Expr::Literal(Literal::Number("31".to_string(), Span::default()));

        let result2 = eval_assignment(&target2, &value2, env.clone()).unwrap();
        assert_eq!(result2, Value::Number(DecimalNumber::from_i64(31)));

        // Check that the nested map was updated
        let updated_config = env.get("config").unwrap();
        if let Value::Map(config) = updated_config {
            if let Some(Value::Map(user)) = config.get(&MapKey::String("user".to_string())) {
                if let Some(Value::Number(age)) = user.get(&MapKey::String("age".to_string())) {
                    assert_eq!(*age, DecimalNumber::from_i64(31));
                } else {
                    panic!("Expected age in user map");
                }
            } else {
                panic!("Expected user map in config");
            }
        } else {
            panic!("Expected map");
        }

        // Test mixed nested assignment: data[0]:name = "updated"
        let mut item1_map = indexmap::IndexMap::new();
        item1_map.insert(
            MapKey::String("name".to_string()),
            Value::String("item1".to_string()),
        );

        let mut item2_map = indexmap::IndexMap::new();
        item2_map.insert(
            MapKey::String("name".to_string()),
            Value::String("item2".to_string()),
        );

        let data = Value::List(vec![Value::Map(item1_map), Value::Map(item2_map)]);
        env.define_or_set("data", data);

        let target3 = Expr::MapAccessByName {
            target: Box::new(Expr::Index {
                target: Box::new(Expr::Literal(Literal::Identifier(
                    "data".to_string(),
                    Span::default(),
                ))),
                index: Box::new(Expr::Literal(Literal::Number(
                    "0".to_string(),
                    Span::default(),
                ))),
                span: Span::default(),
            }),
            key: "name".to_string(),
            span: Span::default(),
        };
        let value3 = Expr::Literal(Literal::StringTemplate(
            vec![crate::ast::StringPart::Text("updated".to_string())],
            Span::default(),
        ));

        let result3 = eval_assignment(&target3, &value3, env.clone()).unwrap();
        assert_eq!(result3, Value::String("updated".to_string()));

        // Check that the mixed nested structure was updated
        let updated_data = env.get("data").unwrap();
        if let Value::List(items) = updated_data {
            if let Value::Map(item) = &items[0] {
                if let Some(Value::String(name)) = item.get(&MapKey::String("name".to_string())) {
                    assert_eq!(name, "updated");
                } else {
                    panic!("Expected name in item map");
                }
            } else {
                panic!("Expected map in first item");
            }
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_deep_list_nesting() {
        let env = create_test_env();

        // Create a 4-level deep nested list: matrix[0][1][2][3] = 99
        let deep_list = Value::List(vec![Value::List(vec![
            Value::List(vec![
                Value::List(vec![
                    Value::Number(DecimalNumber::from_i64(1)),
                    Value::Number(DecimalNumber::from_i64(2)),
                    Value::Number(DecimalNumber::from_i64(3)),
                    Value::Number(DecimalNumber::from_i64(4)),
                ]),
                Value::List(vec![
                    Value::Number(DecimalNumber::from_i64(5)),
                    Value::Number(DecimalNumber::from_i64(6)),
                    Value::Number(DecimalNumber::from_i64(7)),
                    Value::Number(DecimalNumber::from_i64(8)),
                ]),
            ]),
            Value::List(vec![
                Value::List(vec![
                    Value::Number(DecimalNumber::from_i64(9)),
                    Value::Number(DecimalNumber::from_i64(10)),
                    Value::Number(DecimalNumber::from_i64(11)),
                    Value::Number(DecimalNumber::from_i64(12)),
                ]),
                Value::List(vec![
                    Value::Number(DecimalNumber::from_i64(13)),
                    Value::Number(DecimalNumber::from_i64(14)),
                    Value::Number(DecimalNumber::from_i64(15)),
                    Value::Number(DecimalNumber::from_i64(16)),
                ]),
            ]),
        ])]);
        env.define_or_set("matrix", deep_list);

        // Test deep assignment: matrix[0][1][1][3] = 99
        let target = Expr::Index {
            target: Box::new(Expr::Index {
                target: Box::new(Expr::Index {
                    target: Box::new(Expr::Index {
                        target: Box::new(Expr::Literal(Literal::Identifier(
                            "matrix".to_string(),
                            Span::default(),
                        ))),
                        index: Box::new(Expr::Literal(Literal::Number(
                            "0".to_string(),
                            Span::default(),
                        ))),
                        span: Span::default(),
                    }),
                    index: Box::new(Expr::Literal(Literal::Number(
                        "1".to_string(),
                        Span::default(),
                    ))),
                    span: Span::default(),
                }),
                index: Box::new(Expr::Literal(Literal::Number(
                    "1".to_string(),
                    Span::default(),
                ))),
                span: Span::default(),
            }),
            index: Box::new(Expr::Literal(Literal::Number(
                "3".to_string(),
                Span::default(),
            ))),
            span: Span::default(),
        };
        let value = Expr::Literal(Literal::Number("99".to_string(), Span::default()));

        let result = eval_assignment(&target, &value, env.clone()).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(99)));

        // Verify the deep structure was updated correctly
        let updated_matrix = env.get("matrix").unwrap();
        if let Value::List(level1) = updated_matrix {
            if let Value::List(level2) = &level1[0] {
                if let Value::List(level3) = &level2[1] {
                    if let Value::List(level4) = &level3[1] {
                        assert_eq!(level4[3], Value::Number(DecimalNumber::from_i64(99)));
                    } else {
                        panic!("Expected list at level 4");
                    }
                } else {
                    panic!("Expected list at level 3");
                }
            } else {
                panic!("Expected list at level 2");
            }
        } else {
            panic!("Expected list at level 1");
        }
    }

    #[test]
    fn test_deep_map_nesting() {
        let env = create_test_env();

        // Create a 5-level deep nested map: config:user:profile:settings:display:theme = "dark"
        let mut display_map = indexmap::IndexMap::new();
        display_map.insert(
            MapKey::String("theme".to_string()),
            Value::String("light".to_string()),
        );
        display_map.insert(
            MapKey::String("layout".to_string()),
            Value::String("grid".to_string()),
        );

        let mut settings_map = indexmap::IndexMap::new();
        settings_map.insert(
            MapKey::String("display".to_string()),
            Value::Map(display_map),
        );
        settings_map.insert(
            MapKey::String("notifications".to_string()),
            Value::Boolean(true),
        );

        let mut profile_map = indexmap::IndexMap::new();
        profile_map.insert(
            MapKey::String("settings".to_string()),
            Value::Map(settings_map),
        );
        profile_map.insert(
            MapKey::String("avatar".to_string()),
            Value::String("user.png".to_string()),
        );

        let mut user_map = indexmap::IndexMap::new();
        user_map.insert(
            MapKey::String("profile".to_string()),
            Value::Map(profile_map),
        );
        user_map.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );

        let mut config_map = indexmap::IndexMap::new();
        config_map.insert(MapKey::String("user".to_string()), Value::Map(user_map));
        config_map.insert(
            MapKey::String("version".to_string()),
            Value::String("1.0".to_string()),
        );

        env.define_or_set("config", Value::Map(config_map));

        // Test deep assignment: config:user:profile:settings:display:theme = "dark"
        let target = Expr::MapAccessByName {
            target: Box::new(Expr::MapAccessByName {
                target: Box::new(Expr::MapAccessByName {
                    target: Box::new(Expr::MapAccessByName {
                        target: Box::new(Expr::MapAccessByName {
                            target: Box::new(Expr::Literal(Literal::Identifier(
                                "config".to_string(),
                                Span::default(),
                            ))),
                            key: "user".to_string(),
                            span: Span::default(),
                        }),
                        key: "profile".to_string(),
                        span: Span::default(),
                    }),
                    key: "settings".to_string(),
                    span: Span::default(),
                }),
                key: "display".to_string(),
                span: Span::default(),
            }),
            key: "theme".to_string(),
            span: Span::default(),
        };
        let value = Expr::Literal(Literal::StringTemplate(
            vec![crate::ast::StringPart::Text("dark".to_string())],
            Span::default(),
        ));

        let result = eval_assignment(&target, &value, env.clone()).unwrap();
        assert_eq!(result, Value::String("dark".to_string()));

        // Verify the deep structure was updated correctly
        let updated_config = env.get("config").unwrap();
        if let Value::Map(config) = updated_config {
            if let Some(Value::Map(user)) = config.get(&MapKey::String("user".to_string())) {
                if let Some(Value::Map(profile)) = user.get(&MapKey::String("profile".to_string()))
                {
                    if let Some(Value::Map(settings)) =
                        profile.get(&MapKey::String("settings".to_string()))
                    {
                        if let Some(Value::Map(display)) =
                            settings.get(&MapKey::String("display".to_string()))
                        {
                            if let Some(Value::String(theme)) =
                                display.get(&MapKey::String("theme".to_string()))
                            {
                                assert_eq!(theme, "dark");
                            } else {
                                panic!("Expected theme in display map");
                            }
                        } else {
                            panic!("Expected display map in settings");
                        }
                    } else {
                        panic!("Expected settings map in profile");
                    }
                } else {
                    panic!("Expected profile map in user");
                }
            } else {
                panic!("Expected user map in config");
            }
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_mixed_deep_nesting() {
        let env = create_test_env();

        // Create a complex mixed structure: data[0]:users[1]:config:preferences:notifications:email = true
        let mut notifications_map = indexmap::IndexMap::new();
        notifications_map.insert(MapKey::String("email".to_string()), Value::Boolean(false));
        notifications_map.insert(MapKey::String("push".to_string()), Value::Boolean(false));

        let mut preferences_map = indexmap::IndexMap::new();
        preferences_map.insert(
            MapKey::String("notifications".to_string()),
            Value::Map(notifications_map),
        );
        preferences_map.insert(
            MapKey::String("language".to_string()),
            Value::String("en".to_string()),
        );

        let mut config_map = indexmap::IndexMap::new();
        config_map.insert(
            MapKey::String("preferences".to_string()),
            Value::Map(preferences_map),
        );
        config_map.insert(
            MapKey::String("theme".to_string()),
            Value::String("dark".to_string()),
        );

        let mut user1_map = indexmap::IndexMap::new();
        user1_map.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        user1_map.insert(
            MapKey::String("config".to_string()),
            Value::Map(config_map.clone()),
        );

        let mut user2_map = indexmap::IndexMap::new();
        user2_map.insert(
            MapKey::String("name".to_string()),
            Value::String("Bob".to_string()),
        );
        user2_map.insert(MapKey::String("config".to_string()), Value::Map(config_map));

        let mut users_map = indexmap::IndexMap::new();
        users_map.insert(
            MapKey::String("users".to_string()),
            Value::List(vec![Value::Map(user1_map), Value::Map(user2_map)]),
        );
        users_map.insert(
            MapKey::String("count".to_string()),
            Value::Number(DecimalNumber::from_i64(2)),
        );

        let data = Value::List(vec![Value::Map(users_map)]);
        env.define_or_set("data", data);

        // Test mixed deep assignment: data[0]:users[1]:config:preferences:notifications:email = true
        let target = Expr::MapAccessByName {
            target: Box::new(Expr::MapAccessByName {
                target: Box::new(Expr::MapAccessByName {
                    target: Box::new(Expr::MapAccessByName {
                        target: Box::new(Expr::Index {
                            target: Box::new(Expr::MapAccessByName {
                                target: Box::new(Expr::Index {
                                    target: Box::new(Expr::Literal(Literal::Identifier(
                                        "data".to_string(),
                                        Span::default(),
                                    ))),
                                    index: Box::new(Expr::Literal(Literal::Number(
                                        "0".to_string(),
                                        Span::default(),
                                    ))),
                                    span: Span::default(),
                                }),
                                key: "users".to_string(),
                                span: Span::default(),
                            }),
                            index: Box::new(Expr::Literal(Literal::Number(
                                "1".to_string(),
                                Span::default(),
                            ))),
                            span: Span::default(),
                        }),
                        key: "config".to_string(),
                        span: Span::default(),
                    }),
                    key: "preferences".to_string(),
                    span: Span::default(),
                }),
                key: "notifications".to_string(),
                span: Span::default(),
            }),
            key: "email".to_string(),
            span: Span::default(),
        };
        let value = Expr::Literal(Literal::Boolean(true, Span::default()));

        let result = eval_assignment(&target, &value, env.clone()).unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Verify the mixed deep structure was updated correctly
        let updated_data = env.get("data").unwrap();
        if let Value::List(items) = updated_data {
            if let Value::Map(item) = &items[0] {
                if let Some(Value::List(users)) = item.get(&MapKey::String("users".to_string())) {
                    if let Value::Map(user) = &users[1] {
                        if let Some(Value::Map(config)) =
                            user.get(&MapKey::String("config".to_string()))
                        {
                            if let Some(Value::Map(preferences)) =
                                config.get(&MapKey::String("preferences".to_string()))
                            {
                                if let Some(Value::Map(notifications)) =
                                    preferences.get(&MapKey::String("notifications".to_string()))
                                {
                                    if let Some(Value::Boolean(email)) =
                                        notifications.get(&MapKey::String("email".to_string()))
                                    {
                                        assert!(*email);
                                    } else {
                                        panic!("Expected email in notifications map");
                                    }
                                } else {
                                    panic!("Expected notifications map in preferences");
                                }
                            } else {
                                panic!("Expected preferences map in config");
                            }
                        } else {
                            panic!("Expected config map in user");
                        }
                    } else {
                        panic!("Expected user map in users list");
                    }
                } else {
                    panic!("Expected users list in item");
                }
            } else {
                panic!("Expected map in first item");
            }
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_very_deep_nesting() {
        let env = create_test_env();

        // Create a very deep structure (6 levels): deep[0][1][2][3][4][5] = 42
        let level6 = vec![Value::Number(DecimalNumber::from_i64(0)); 6];
        let level5 = vec![Value::List(level6); 5];
        let level4 = vec![Value::List(level5); 4];
        let level3 = vec![Value::List(level4); 3];
        let level2 = vec![Value::List(level3); 2];
        let level1 = vec![Value::List(level2); 1];

        env.define_or_set("deep", Value::List(level1));

        // Test very deep assignment: deep[0][1][2][3][4][5] = 42
        let target = Expr::Index {
            target: Box::new(Expr::Index {
                target: Box::new(Expr::Index {
                    target: Box::new(Expr::Index {
                        target: Box::new(Expr::Index {
                            target: Box::new(Expr::Index {
                                target: Box::new(Expr::Literal(Literal::Identifier(
                                    "deep".to_string(),
                                    Span::default(),
                                ))),
                                index: Box::new(Expr::Literal(Literal::Number(
                                    "0".to_string(),
                                    Span::default(),
                                ))),
                                span: Span::default(),
                            }),
                            index: Box::new(Expr::Literal(Literal::Number(
                                "1".to_string(),
                                Span::default(),
                            ))),
                            span: Span::default(),
                        }),
                        index: Box::new(Expr::Literal(Literal::Number(
                            "2".to_string(),
                            Span::default(),
                        ))),
                        span: Span::default(),
                    }),
                    index: Box::new(Expr::Literal(Literal::Number(
                        "3".to_string(),
                        Span::default(),
                    ))),
                    span: Span::default(),
                }),
                index: Box::new(Expr::Literal(Literal::Number(
                    "4".to_string(),
                    Span::default(),
                ))),
                span: Span::default(),
            }),
            index: Box::new(Expr::Literal(Literal::Number(
                "5".to_string(),
                Span::default(),
            ))),
            span: Span::default(),
        };
        let value = Expr::Literal(Literal::Number("42".to_string(), Span::default()));

        let result = eval_assignment(&target, &value, env.clone()).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(42)));

        // Verify the very deep structure was updated correctly
        let updated_deep = env.get("deep").unwrap();
        if let Value::List(l1) = updated_deep {
            if let Value::List(l2) = &l1[0] {
                if let Value::List(l3) = &l2[1] {
                    if let Value::List(l4) = &l3[2] {
                        if let Value::List(l5) = &l4[3] {
                            if let Value::List(l6) = &l5[4] {
                                assert_eq!(l6[5], Value::Number(DecimalNumber::from_i64(42)));
                            } else {
                                panic!("Expected list at level 6");
                            }
                        } else {
                            panic!("Expected list at level 5");
                        }
                    } else {
                        panic!("Expected list at level 4");
                    }
                } else {
                    panic!("Expected list at level 3");
                }
            } else {
                panic!("Expected list at level 2");
            }
        } else {
            panic!("Expected list at level 1");
        }
    }
}
