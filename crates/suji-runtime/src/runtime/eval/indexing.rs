use super::{EvalResult, eval_expr};
use crate::runtime::env::Env;
use crate::runtime::value::{MapKey, RuntimeError, Value};
use std::rc::Rc;
use suji_ast::ast::Expr;

#[cfg(test)]
use crate::runtime::builtins::setup_global_env;

/// Evaluate array/map indexing
pub fn eval_index(target: &Expr, index: &Expr, env: Rc<Env>) -> EvalResult<Value> {
    let target_value = eval_expr(target, env.clone())?;
    let index_value = eval_expr(index, env)?;

    match target_value {
        Value::List(ref items) => {
            let idx = match index_value {
                Value::Number(n) => {
                    if !n.is_integer() {
                        return Err(RuntimeError::TypeError {
                            message: "List index must be an integer".to_string(),
                        });
                    }
                    n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
                        message: "Index out of range".to_string(),
                    })?
                }
                _ => {
                    return Err(RuntimeError::TypeError {
                        message: format!(
                            "List index must be a number, got {}",
                            index_value.type_name()
                        ),
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

            Ok(items[normalized_idx as usize].clone())
        }
        Value::String(ref s) => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len() as i64;

            match index_value {
                Value::Number(n) => {
                    if !n.is_integer() {
                        return Err(RuntimeError::TypeError {
                            message: "String index must be an integer".to_string(),
                        });
                    }

                    let idx = n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
                        message: "Index out of range".to_string(),
                    })?;
                    let normalized_idx = if idx < 0 { len + idx } else { idx };

                    if normalized_idx < 0 || normalized_idx >= len {
                        return Err(RuntimeError::IndexOutOfBounds {
                            message: format!(
                                "String index {} out of bounds for length {}",
                                idx, len
                            ),
                        });
                    }

                    Ok(Value::String(chars[normalized_idx as usize].to_string()))
                }
                _ => Err(RuntimeError::TypeError {
                    message: "String index must be a number".to_string(),
                }),
            }
        }
        Value::Map(ref map) => {
            let key = index_value.try_into_map_key()?;
            match map.get(&key) {
                Some(value) => Ok(value.clone()),
                None => Err(RuntimeError::KeyNotFound {
                    message: format!("Key {} not found in map", key),
                }),
            }
        }
        Value::EnvMap(ref env_proxy) => {
            let key_str = match index_value {
                Value::String(s) => s,
                _ => {
                    return Err(RuntimeError::TypeError {
                        message: "Environment keys must be strings".to_string(),
                    });
                }
            };
            match env_proxy.get(&key_str) {
                Some(value) => Ok(Value::String(value)),
                None => Err(RuntimeError::KeyNotFound {
                    message: format!("Environment variable not found: {}", key_str),
                }),
            }
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot index {}", target_value.type_name()),
        }),
    }
}

/// Evaluate list slicing
pub fn eval_slice(
    target: &Expr,
    start: Option<&Expr>,
    end: Option<&Expr>,
    env: Rc<Env>,
) -> EvalResult<Value> {
    let target_value = eval_expr(target, env.clone())?;

    match target_value {
        Value::List(ref items) => {
            let len = items.len() as i64;

            // Evaluate start index
            let start_idx = match start {
                Some(expr) => {
                    let val = eval_expr(expr, env.clone())?;
                    match val {
                        Value::Number(n) => {
                            if !n.is_integer() {
                                return Err(RuntimeError::TypeError {
                                    message: "Slice start must be an integer".to_string(),
                                });
                            }
                            n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
                                message: "Index out of range".to_string(),
                            })?
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "Slice start must be a number".to_string(),
                            });
                        }
                    }
                }
                None => 0,
            };

            // Evaluate end index
            let end_idx = match end {
                Some(expr) => {
                    let val = eval_expr(expr, env)?;
                    match val {
                        Value::Number(n) => {
                            if !n.is_integer() {
                                return Err(RuntimeError::TypeError {
                                    message: "Slice end must be an integer".to_string(),
                                });
                            }
                            n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
                                message: "Index out of range".to_string(),
                            })?
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "Slice end must be a number".to_string(),
                            });
                        }
                    }
                }
                None => len,
            };

            // Normalize negative indices
            let norm_start = if start_idx < 0 {
                len + start_idx
            } else {
                start_idx
            };
            let norm_end = if end_idx < 0 { len + end_idx } else { end_idx };

            // Clamp to bounds
            let clamped_start = norm_start.max(0).min(len) as usize;
            let clamped_end = norm_end.max(0).min(len) as usize;

            // Return slice
            if clamped_start > clamped_end {
                Ok(Value::List(vec![]))
            } else {
                Ok(Value::List(items[clamped_start..clamped_end].to_vec()))
            }
        }
        Value::String(ref s) => {
            let len = s.chars().count() as i64;
            let chars: Vec<char> = s.chars().collect();

            // Evaluate start index
            let start_idx = match start {
                Some(expr) => {
                    let val = eval_expr(expr, env.clone())?;
                    match val {
                        Value::Number(n) => {
                            if !n.is_integer() {
                                return Err(RuntimeError::TypeError {
                                    message: "String slice start must be an integer".to_string(),
                                });
                            }
                            n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
                                message: "Index out of range".to_string(),
                            })?
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "String slice start must be a number".to_string(),
                            });
                        }
                    }
                }
                None => 0,
            };

            // Evaluate end index
            let end_idx = match end {
                Some(expr) => {
                    let val = eval_expr(expr, env)?;
                    match val {
                        Value::Number(n) => {
                            if !n.is_integer() {
                                return Err(RuntimeError::TypeError {
                                    message: "String slice end must be an integer".to_string(),
                                });
                            }
                            n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
                                message: "Index out of range".to_string(),
                            })?
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "String slice end must be a number".to_string(),
                            });
                        }
                    }
                }
                None => len,
            };

            // Normalize negative indices
            let norm_start = if start_idx < 0 {
                len + start_idx
            } else {
                start_idx
            };
            let norm_end = if end_idx < 0 { len + end_idx } else { end_idx };

            // Clamp to bounds
            let clamped_start = norm_start.max(0).min(len) as usize;
            let clamped_end = norm_end.max(0).min(len) as usize;

            // Return slice
            if clamped_start > clamped_end {
                Ok(Value::String(String::new()))
            } else {
                let sliced_chars: String = chars[clamped_start..clamped_end].iter().collect();
                Ok(Value::String(sliced_chars))
            }
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot slice {}", target_value.type_name()),
        }),
    }
}

/// Evaluate map access by name (map:key)
pub fn eval_map_access_by_name(target: &Expr, key: &str, env: Rc<Env>) -> EvalResult<Value> {
    let target_value = eval_expr(target, env)?;

    match target_value {
        Value::Map(ref map) => {
            let map_key = MapKey::String(key.to_string());
            match map.get(&map_key) {
                Some(value) => Ok(value.clone()),
                None => Err(RuntimeError::KeyNotFound {
                    message: format!("Key '{}' not found in map", key),
                }),
            }
        }
        Value::EnvMap(ref env_proxy) => match env_proxy.get(key) {
            Some(value) => Ok(Value::String(value)),
            None => Err(RuntimeError::KeyNotFound {
                message: format!("Environment variable not found: {}", key),
            }),
        },
        _ => Err(RuntimeError::TypeError {
            message: format!(
                "Cannot access key '{}' on {}",
                key,
                target_value.type_name()
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_list_indexing() {
        let env = create_test_env();

        // Create a list [1, 2, 3]
        let list = Value::List(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3)),
        ]);
        env.define_or_set("my_list", list);

        let target = Expr::Literal(Literal::Identifier("my_list".to_string(), Span::default()));
        let index = Expr::Literal(Literal::Number("1".to_string(), Span::default()));

        let result = eval_index(&target, &index, env).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));
    }

    #[test]
    fn test_negative_indexing() {
        let env = create_test_env();

        let list = Value::List(vec![
            Value::Number(DecimalNumber::from_i64(10)),
            Value::Number(DecimalNumber::from_i64(20)),
            Value::Number(DecimalNumber::from_i64(30)),
        ]);
        env.define_or_set("my_list", list);

        let target = Expr::Literal(Literal::Identifier("my_list".to_string(), Span::default()));
        let index = Expr::Literal(Literal::Number("-1".to_string(), Span::default()));

        let result = eval_index(&target, &index, env).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(30)));
    }

    #[test]
    fn test_list_slicing() {
        let env = create_test_env();

        let list = Value::List(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3)),
            Value::Number(DecimalNumber::from_i64(4)),
            Value::Number(DecimalNumber::from_i64(5)),
        ]);
        env.define_or_set("my_list", list);

        let target = Expr::Literal(Literal::Identifier("my_list".to_string(), Span::default()));
        let start = Expr::Literal(Literal::Number("1".to_string(), Span::default()));
        let end = Expr::Literal(Literal::Number("4".to_string(), Span::default()));

        let result = eval_slice(&target, Some(&start), Some(&end), env).unwrap();

        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(2)));
            assert_eq!(items[1], Value::Number(DecimalNumber::from_i64(3)));
            assert_eq!(items[2], Value::Number(DecimalNumber::from_i64(4)));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_map_access_by_name() {
        let env = create_test_env();

        let mut map = indexmap::IndexMap::new();
        map.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(30)),
        );

        env.define_or_set("person", Value::Map(map));

        let target = Expr::Literal(Literal::Identifier("person".to_string(), Span::default()));

        let result = eval_map_access_by_name(&target, "name", env).unwrap();
        assert_eq!(result, Value::String("Alice".to_string()));
    }

    #[test]
    fn test_string_slicing() {
        let env = create_test_env();

        let s = Value::String("hello".to_string());
        env.define_or_set("my_string", s);

        let target = Expr::Literal(Literal::Identifier(
            "my_string".to_string(),
            Span::default(),
        ));
        let start = Expr::Literal(Literal::Number("1".to_string(), Span::default()));
        let end = Expr::Literal(Literal::Number("3".to_string(), Span::default()));

        let result = eval_slice(&target, Some(&start), Some(&end), env.clone()).unwrap();
        assert_eq!(result, Value::String("el".to_string()));

        // Test negative indices
        let start_neg = Expr::Literal(Literal::Number("-2".to_string(), Span::default()));
        let result2 = eval_slice(&target, Some(&start_neg), None, env.clone()).unwrap();
        assert_eq!(result2, Value::String("lo".to_string()));

        // Test empty slice
        let start_empty = Expr::Literal(Literal::Number("3".to_string(), Span::default()));
        let end_empty = Expr::Literal(Literal::Number("1".to_string(), Span::default()));
        let result3 = eval_slice(&target, Some(&start_empty), Some(&end_empty), env).unwrap();
        assert_eq!(result3, Value::String("".to_string()));
    }

    #[test]
    fn test_string_slicing_edge_cases() {
        let env = create_test_env();

        let s = Value::String("hello".to_string());
        env.define_or_set("my_string", s);

        let target = Expr::Literal(Literal::Identifier(
            "my_string".to_string(),
            Span::default(),
        ));

        // Test [:2] (first two characters)
        let end = Expr::Literal(Literal::Number("2".to_string(), Span::default()));
        let result = eval_slice(&target, None, Some(&end), env.clone()).unwrap();
        assert_eq!(result, Value::String("he".to_string()));

        // Test [2:] (from third character to end)
        let start = Expr::Literal(Literal::Number("2".to_string(), Span::default()));
        let result2 = eval_slice(&target, Some(&start), None, env).unwrap();
        assert_eq!(result2, Value::String("llo".to_string()));
    }

    #[test]
    fn test_string_indexing_basic() {
        let env = create_test_env();

        let s = Value::String("hello".to_string());
        env.define_or_set("my_string", s);

        let target = Expr::Literal(Literal::Identifier(
            "my_string".to_string(),
            Span::default(),
        ));

        // Test positive indexing
        let index0 = Expr::Literal(Literal::Number("0".to_string(), Span::default()));
        let result0 = eval_index(&target, &index0, env.clone()).unwrap();
        assert_eq!(result0, Value::String("h".to_string()));

        let index1 = Expr::Literal(Literal::Number("1".to_string(), Span::default()));
        let result1 = eval_index(&target, &index1, env.clone()).unwrap();
        assert_eq!(result1, Value::String("e".to_string()));

        let index4 = Expr::Literal(Literal::Number("4".to_string(), Span::default()));
        let result4 = eval_index(&target, &index4, env).unwrap();
        assert_eq!(result4, Value::String("o".to_string()));
    }

    #[test]
    fn test_string_indexing_negative() {
        let env = create_test_env();

        let s = Value::String("hello".to_string());
        env.define_or_set("my_string", s);

        let target = Expr::Literal(Literal::Identifier(
            "my_string".to_string(),
            Span::default(),
        ));

        // Test negative indexing
        let index_neg1 = Expr::Literal(Literal::Number("-1".to_string(), Span::default()));
        let result_neg1 = eval_index(&target, &index_neg1, env.clone()).unwrap();
        assert_eq!(result_neg1, Value::String("o".to_string()));

        let index_neg2 = Expr::Literal(Literal::Number("-2".to_string(), Span::default()));
        let result_neg2 = eval_index(&target, &index_neg2, env.clone()).unwrap();
        assert_eq!(result_neg2, Value::String("l".to_string()));

        let index_neg5 = Expr::Literal(Literal::Number("-5".to_string(), Span::default()));
        let result_neg5 = eval_index(&target, &index_neg5, env).unwrap();
        assert_eq!(result_neg5, Value::String("h".to_string()));
    }

    #[test]
    fn test_string_indexing_unicode() {
        let env = create_test_env();

        let s = Value::String("café".to_string());
        env.define_or_set("my_string", s);

        let target = Expr::Literal(Literal::Identifier(
            "my_string".to_string(),
            Span::default(),
        ));

        // Test Unicode character indexing
        let index3 = Expr::Literal(Literal::Number("3".to_string(), Span::default()));
        let result3 = eval_index(&target, &index3, env).unwrap();
        assert_eq!(result3, Value::String("é".to_string()));
    }

    #[test]
    fn test_string_indexing_out_of_bounds() {
        let env = create_test_env();

        let s = Value::String("hello".to_string());
        env.define_or_set("my_string", s);

        let target = Expr::Literal(Literal::Identifier(
            "my_string".to_string(),
            Span::default(),
        ));

        // Test out of bounds positive index
        let index5 = Expr::Literal(Literal::Number("5".to_string(), Span::default()));
        let result = eval_index(&target, &index5, env.clone());
        assert!(result.is_err());
        if let Err(RuntimeError::IndexOutOfBounds { message }) = result {
            assert!(message.contains("String index 5 out of bounds for length 5"));
        } else {
            panic!("Expected IndexOutOfBounds error");
        }

        // Test out of bounds negative index
        let index_neg6 = Expr::Literal(Literal::Number("-6".to_string(), Span::default()));
        let result2 = eval_index(&target, &index_neg6, env);
        assert!(result2.is_err());
        if let Err(RuntimeError::IndexOutOfBounds { message }) = result2 {
            assert!(message.contains("String index -6 out of bounds for length 5"));
        } else {
            panic!("Expected IndexOutOfBounds error");
        }
    }

    #[test]
    fn test_string_indexing_empty_string() {
        let env = create_test_env();

        let s = Value::String("".to_string());
        env.define_or_set("empty_string", s);

        let target = Expr::Literal(Literal::Identifier(
            "empty_string".to_string(),
            Span::default(),
        ));

        // Test indexing empty string
        let index0 = Expr::Literal(Literal::Number("0".to_string(), Span::default()));
        let result = eval_index(&target, &index0, env);
        assert!(result.is_err());
        if let Err(RuntimeError::IndexOutOfBounds { message }) = result {
            assert!(message.contains("String index 0 out of bounds for length 0"));
        } else {
            panic!("Expected IndexOutOfBounds error");
        }
    }

    #[test]
    fn test_string_indexing_type_errors() {
        let env = create_test_env();

        let s = Value::String("hello".to_string());
        env.define_or_set("my_string", s);

        let target = Expr::Literal(Literal::Identifier(
            "my_string".to_string(),
            Span::default(),
        ));

        // Test non-integer number index
        let index_float = Expr::Literal(Literal::Number("1.5".to_string(), Span::default()));
        let result = eval_index(&target, &index_float, env.clone());
        assert!(result.is_err());
        if let Err(RuntimeError::TypeError { message }) = result {
            assert_eq!(message, "String index must be an integer");
        } else {
            panic!("Expected TypeError for non-integer index");
        }

        // Test non-number index
        let index_bool = Expr::Literal(Literal::Boolean(true, Span::default()));
        let result2 = eval_index(&target, &index_bool, env);
        assert!(result2.is_err());
        if let Err(RuntimeError::TypeError { message }) = result2 {
            assert_eq!(message, "String index must be a number");
        } else {
            panic!("Expected TypeError for non-number index");
        }
    }
}
