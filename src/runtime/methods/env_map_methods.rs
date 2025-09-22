//! Methods for `EnvMap` (environment variables with overlay support).

use super::super::value::{RuntimeError, Value};
use super::common::ValueRef;

/// Methods: contains(key), get(key, default=nil), keys(), values(), to_list(), length(), delete(key), merge(other_map)
pub fn call_env_map_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    if let Value::EnvMap(env_proxy) = receiver.get() {
        match method {
            "contains" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "contains() takes exactly one argument".to_string(),
                    });
                }

                let key = match &args[0] {
                    Value::String(s) => s,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Environment keys must be strings".to_string(),
                        });
                    }
                };

                Ok(Value::Boolean(env_proxy.contains(key)))
            }
            "get" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "get() takes one or two arguments".to_string(),
                    });
                }

                let key = match &args[0] {
                    Value::String(s) => s,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Environment keys must be strings".to_string(),
                        });
                    }
                };

                let default = if args.len() == 2 {
                    args[1].clone()
                } else {
                    Value::Nil
                };

                match env_proxy.get(key) {
                    Some(value) => Ok(Value::String(value)),
                    None => Ok(default),
                }
            }
            "keys" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "keys() takes no arguments".to_string(),
                    });
                }
                let keys: Vec<Value> = env_proxy.keys().into_iter().map(Value::String).collect();
                Ok(Value::List(keys))
            }
            "values" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "values() takes no arguments".to_string(),
                    });
                }
                let values: Vec<Value> =
                    env_proxy.values().into_iter().map(Value::String).collect();
                Ok(Value::List(values))
            }
            "to_list" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_list() takes no arguments".to_string(),
                    });
                }
                let pairs: Vec<Value> = env_proxy
                    .to_list()
                    .into_iter()
                    .map(|(k, v)| Value::Tuple(vec![Value::String(k), Value::String(v)]))
                    .collect();
                Ok(Value::List(pairs))
            }
            "length" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "length() takes no arguments".to_string(),
                    });
                }
                Ok(Value::Number(env_proxy.length() as f64))
            }
            "delete" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "delete() takes exactly one argument".to_string(),
                    });
                }

                let key = match &args[0] {
                    Value::String(s) => s,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Environment keys must be strings".to_string(),
                        });
                    }
                };

                Ok(Value::Boolean(env_proxy.delete(key)))
            }
            "merge" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "merge() takes exactly one argument".to_string(),
                    });
                }

                let other_map = match &args[0] {
                    Value::Map(map) => map,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "merge() argument must be a map".to_string(),
                        });
                    }
                };

                for (key, value) in other_map {
                    let key_str = match key {
                        super::super::value::MapKey::String(s) => s,
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "Environment keys must be strings".to_string(),
                            });
                        }
                    };
                    let value_str = value.to_string();
                    env_proxy.set(key_str, &value_str)?;
                }
                // Return the same EnvMap after merging
                Ok(Value::EnvMap(env_proxy.clone()))
            }
            "to_string" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_string() takes no arguments".to_string(),
                    });
                }
                // Format as a map-like string representation
                let pairs = env_proxy.to_list();
                if pairs.is_empty() {
                    Ok(Value::String("{}".to_string()))
                } else {
                    let formatted = pairs
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    Ok(Value::String(format!("{{{}}}", formatted)))
                }
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("ENV has no method '{}'", method),
            }),
        }
    } else {
        unreachable!("call_env_map_method called on non-EnvMap value")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::env_overlay::EnvProxy;
    use std::rc::Rc;

    fn create_env_map_value() -> Value {
        Value::EnvMap(Rc::new(EnvProxy::new()))
    }

    fn with_env_map<F, R>(f: F) -> R
    where
        F: FnOnce(ValueRef) -> R,
    {
        let mut value = create_env_map_value();
        let value_ref = ValueRef::Mutable(&mut value);
        f(value_ref)
    }

    #[test]
    fn test_env_map_contains() {
        with_env_map(|env_map| {
            // Set a test variable
            if let Value::EnvMap(env_proxy) = env_map.get() {
                env_proxy.set("TEST_VAR", "test_value").unwrap();
            }

            let result = call_env_map_method(
                env_map,
                "contains",
                vec![Value::String("TEST_VAR".to_string())],
            )
            .unwrap();
            assert_eq!(result, Value::Boolean(true));
        });

        with_env_map(|env_map| {
            let result = call_env_map_method(
                env_map,
                "contains",
                vec![Value::String("NONEXISTENT".to_string())],
            )
            .unwrap();
            assert_eq!(result, Value::Boolean(false));
        });
    }

    #[test]
    fn test_env_map_get() {
        with_env_map(|env_map| {
            // Set a test variable
            if let Value::EnvMap(env_proxy) = env_map.get() {
                env_proxy.set("TEST_VAR", "test_value").unwrap();
            }

            let result =
                call_env_map_method(env_map, "get", vec![Value::String("TEST_VAR".to_string())])
                    .unwrap();
            assert_eq!(result, Value::String("test_value".to_string()));
        });

        with_env_map(|env_map| {
            // Test with default
            let result = call_env_map_method(
                env_map,
                "get",
                vec![
                    Value::String("NONEXISTENT".to_string()),
                    Value::String("default".to_string()),
                ],
            )
            .unwrap();
            assert_eq!(result, Value::String("default".to_string()));
        });
    }

    #[test]
    fn test_env_map_keys_values() {
        with_env_map(|env_map| {
            // Set test variables
            if let Value::EnvMap(env_proxy) = env_map.get() {
                env_proxy.set("KEY1", "value1").unwrap();
                env_proxy.set("KEY2", "value2").unwrap();
            }

            let keys_result = call_env_map_method(env_map, "keys", vec![]).unwrap();
            if let Value::List(keys) = keys_result {
                assert!(keys.contains(&Value::String("KEY1".to_string())));
                assert!(keys.contains(&Value::String("KEY2".to_string())));
            } else {
                panic!("Expected List result from keys()");
            }
        });

        with_env_map(|env_map| {
            // Set test variables again for values test
            if let Value::EnvMap(env_proxy) = env_map.get() {
                env_proxy.set("KEY1", "value1").unwrap();
                env_proxy.set("KEY2", "value2").unwrap();
            }

            let values_result = call_env_map_method(env_map, "values", vec![]).unwrap();
            if let Value::List(values) = values_result {
                assert!(values.contains(&Value::String("value1".to_string())));
                assert!(values.contains(&Value::String("value2".to_string())));
            } else {
                panic!("Expected List result from values()");
            }
        });
    }

    #[test]
    fn test_env_map_delete() {
        // Use an unlikely environment variable name to avoid collisions with the real OS env
        let var_name = "NNLANG_TEST_ENV_MAP_DELETE_VAR";

        with_env_map(|env_map| {
            // Set a test variable
            if let Value::EnvMap(env_proxy) = env_map.get() {
                env_proxy.set(var_name, "test_value").unwrap();
            }

            let result =
                call_env_map_method(env_map, "delete", vec![Value::String(var_name.to_string())])
                    .unwrap();
            assert_eq!(result, Value::Boolean(true));
        });

        with_env_map(|env_map| {
            // Verify it's deleted in a fresh env map
            let result = call_env_map_method(
                env_map,
                "contains",
                vec![Value::String(var_name.to_string())],
            )
            .unwrap();
            assert_eq!(result, Value::Boolean(false));
        });
    }

    #[test]
    fn test_env_map_arity_errors() {
        with_env_map(|env_map| {
            // Test wrong arity for contains
            let result = call_env_map_method(env_map, "contains", vec![]);
            assert!(result.is_err());
        });

        with_env_map(|env_map| {
            // Test wrong arity for get
            let result = call_env_map_method(env_map, "get", vec![]);
            assert!(result.is_err());
        });

        with_env_map(|env_map| {
            // Test wrong arity for keys
            let result =
                call_env_map_method(env_map, "keys", vec![Value::String("extra".to_string())]);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_env_map_type_errors() {
        with_env_map(|env_map| {
            // Test non-string key for contains
            let result = call_env_map_method(env_map, "contains", vec![Value::Number(42.0)]);
            assert!(result.is_err());
        });

        with_env_map(|env_map| {
            // Test non-string key for get
            let result = call_env_map_method(env_map, "get", vec![Value::Number(42.0)]);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_env_map_method_not_found() {
        with_env_map(|env_map| {
            let result = call_env_map_method(env_map, "nonexistent_method", vec![]);
            assert!(result.is_err());
            if let Err(RuntimeError::InvalidOperation { message }) = result {
                assert!(message.contains("nonexistent_method"));
            } else {
                panic!("Expected InvalidOperation error");
            }
        });
    }
}
