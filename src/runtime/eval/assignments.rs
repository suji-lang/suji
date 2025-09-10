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
            // Mixed nested access: target:key[index] = value
            let nested_target_value = eval_expr(nested_target, env.clone())?;

            // Get the nested item
            let nested_item = get_map_access_value(&nested_target_value, nested_key)?;

            // Update the nested item
            let updated_nested_item = update_index_value(&nested_item, &index_value, &value)?;

            // Update the parent structure
            let updated_parent =
                update_map_access_value(&nested_target_value, nested_key, &updated_nested_item)?;

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
            // Nested map access: target:key1:key2 = value
            let nested_target_value = eval_expr(nested_target, env.clone())?;

            // Get the nested item
            let nested_item = get_map_access_value(&nested_target_value, nested_key)?;

            // Update the nested item
            let updated_nested_item = update_map_access_value(&nested_item, key, &value)?;

            // Update the parent structure
            let updated_parent =
                update_map_access_value(&nested_target_value, nested_key, &updated_nested_item)?;

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
        _ => Err(RuntimeError::InvalidOperation {
            message: "Complex assignment targets not yet supported".to_string(),
        }),
    }
}

/// Helper function to get a value by index
fn get_index_value(target: &Value, index: &Value) -> EvalResult<Value> {
    match (target, index) {
        (Value::List(items), Value::Number(n)) => {
            if n.fract() != 0.0 {
                return Err(RuntimeError::TypeError {
                    message: "List index must be an integer".to_string(),
                });
            }
            let idx = *n as i64;
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
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot access key '{}' on {}", key, target.type_name()),
        }),
    }
}

/// Helper function to update a value by index
fn update_index_value(target: &Value, index: &Value, value: &Value) -> EvalResult<Value> {
    match (target, index) {
        (Value::List(items), Value::Number(n)) => {
            if n.fract() != 0.0 {
                return Err(RuntimeError::TypeError {
                    message: "List index must be an integer".to_string(),
                });
            }
            let idx = *n as i64;
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
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot assign key '{}' on {}", key, target.type_name()),
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

    #[test]
    fn test_no_variable_shadowing() {
        let env = create_test_env();

        // Define variable in parent scope
        env.define_or_set("x", Value::Number(1.0));
        assert_eq!(env.get("x").unwrap(), Value::Number(1.0));

        // Create child environment
        let child_env = Rc::new(Env::new_child(env.clone()));

        // Assign to x in child scope - should modify parent variable
        let target = Expr::Literal(Literal::Identifier("x".to_string(), Span::default()));
        let value = Expr::Literal(Literal::Number(2.0, Span::default()));

        let result = eval_assignment(&target, &value, child_env.clone()).unwrap();
        assert_eq!(result, Value::Number(2.0));

        // Check that parent variable was modified (no shadowing)
        assert_eq!(env.get("x").unwrap(), Value::Number(2.0));
        assert_eq!(child_env.get("x").unwrap(), Value::Number(2.0));

        // Check that no new variable was created in child scope
        assert_eq!(child_env.local_names().len(), 0);
    }

    #[test]
    fn test_complex_assignments() {
        let env = create_test_env();

        // Test nested list assignment: matrix[0][1] = 99
        let matrix = Value::List(vec![
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(3.0), Value::Number(4.0)]),
        ]);
        env.define_or_set("matrix", matrix);

        let target = Expr::Index {
            target: Box::new(Expr::Index {
                target: Box::new(Expr::Literal(Literal::Identifier(
                    "matrix".to_string(),
                    Span::default(),
                ))),
                index: Box::new(Expr::Literal(Literal::Number(0.0, Span::default()))),
                span: Span::default(),
            }),
            index: Box::new(Expr::Literal(Literal::Number(1.0, Span::default()))),
            span: Span::default(),
        };
        let value = Expr::Literal(Literal::Number(99.0, Span::default()));

        let result = eval_assignment(&target, &value, env.clone()).unwrap();
        assert_eq!(result, Value::Number(99.0));

        // Check that the nested structure was updated
        let updated_matrix = env.get("matrix").unwrap();
        if let Value::List(rows) = updated_matrix {
            if let Value::List(first_row) = &rows[0] {
                assert_eq!(first_row[1], Value::Number(99.0));
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
        user_map.insert(MapKey::String("age".to_string()), Value::Number(30.0));

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
        let value2 = Expr::Literal(Literal::Number(31.0, Span::default()));

        let result2 = eval_assignment(&target2, &value2, env.clone()).unwrap();
        assert_eq!(result2, Value::Number(31.0));

        // Check that the nested map was updated
        let updated_config = env.get("config").unwrap();
        if let Value::Map(config) = updated_config {
            if let Some(Value::Map(user)) = config.get(&MapKey::String("user".to_string())) {
                if let Some(Value::Number(age)) = user.get(&MapKey::String("age".to_string())) {
                    assert_eq!(*age, 31.0);
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
                index: Box::new(Expr::Literal(Literal::Number(0.0, Span::default()))),
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
}
