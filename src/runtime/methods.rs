use super::value::{RuntimeError, Value};

/// Reference to a value that may be mutable (for method calls)
pub enum ValueRef<'a> {
    /// Mutable reference to a value (can be modified by methods)
    Mutable(&'a mut Value),
    /// Immutable reference to a value (read-only methods only)
    Immutable(&'a Value),
}

impl<'a> ValueRef<'a> {
    /// Get the value (immutable access)
    pub fn get(&self) -> &Value {
        match self {
            ValueRef::Mutable(v) => v,
            ValueRef::Immutable(v) => v,
        }
    }

    /// Get mutable access to the value, or error if immutable
    pub fn get_mut(&mut self) -> Result<&mut Value, RuntimeError> {
        match self {
            ValueRef::Mutable(v) => Ok(v),
            ValueRef::Immutable(_) => Err(RuntimeError::MethodError {
                message: "Cannot call mutating method on immutable value".to_string(),
            }),
        }
    }
}

/// Call a method on a value with the given arguments
pub fn call_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::String(_) => call_string_method(receiver, method, args),
        Value::List(_) => call_list_method(receiver, method, args),
        Value::Map(_) => call_map_method(receiver, method, args),
        _ => Err(RuntimeError::MethodError {
            message: format!("{} has no method '{}'", receiver.get().type_name(), method),
        }),
    }
}

/// String methods: length(), split(separator=" ")
fn call_string_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::String(s) => match method {
            "length" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "length() takes no arguments".to_string(),
                    });
                }
                Ok(Value::Number(s.chars().count() as f64))
            }
            "split" => {
                let separator = if args.is_empty() {
                    " ".to_string()
                } else if args.len() == 1 {
                    match &args[0] {
                        Value::String(sep) => sep.clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "split() separator must be a string".to_string(),
                            });
                        }
                    }
                } else {
                    return Err(RuntimeError::ArityMismatch {
                        message: "split() takes at most one argument".to_string(),
                    });
                };

                let parts: Vec<Value> = s
                    .split(&separator)
                    .map(|part| Value::String(part.to_string()))
                    .collect();
                Ok(Value::List(parts))
            }
            _ => Err(RuntimeError::MethodError {
                message: format!("String has no method '{}'", method),
            }),
        },
        _ => unreachable!(),
    }
}

/// List methods: push(item), pop(), length(), join(separator=" ")
fn call_list_method(
    mut receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match method {
        "push" => {
            if args.len() != 1 {
                return Err(RuntimeError::ArityMismatch {
                    message: "push() takes exactly one argument".to_string(),
                });
            }

            let list = receiver.get_mut()?;
            if let Value::List(items) = list {
                items.push(args.into_iter().next().unwrap());
                Ok(Value::Null)
            } else {
                unreachable!()
            }
        }
        "pop" => {
            if !args.is_empty() {
                return Err(RuntimeError::ArityMismatch {
                    message: "pop() takes no arguments".to_string(),
                });
            }

            let list = receiver.get_mut()?;
            if let Value::List(items) = list {
                items.pop().ok_or_else(|| RuntimeError::InvalidOperation {
                    message: "Cannot pop from empty list".to_string(),
                })
            } else {
                unreachable!()
            }
        }
        "length" => {
            if !args.is_empty() {
                return Err(RuntimeError::ArityMismatch {
                    message: "length() takes no arguments".to_string(),
                });
            }

            if let Value::List(items) = receiver.get() {
                Ok(Value::Number(items.len() as f64))
            } else {
                unreachable!()
            }
        }
        "join" => {
            let separator = if args.is_empty() {
                " ".to_string()
            } else if args.len() == 1 {
                match &args[0] {
                    Value::String(sep) => sep.clone(),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "join() separator must be a string".to_string(),
                        });
                    }
                }
            } else {
                return Err(RuntimeError::ArityMismatch {
                    message: "join() takes at most one argument".to_string(),
                });
            };

            if let Value::List(items) = receiver.get() {
                let strings: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                Ok(Value::String(strings.join(&separator)))
            } else {
                unreachable!()
            }
        }
        _ => Err(RuntimeError::MethodError {
            message: format!("List has no method '{}'", method),
        }),
    }
}

/// Map methods: delete(key)
fn call_map_method(
    mut receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match method {
        "delete" => {
            if args.len() != 1 {
                return Err(RuntimeError::ArityMismatch {
                    message: "delete() takes exactly one argument".to_string(),
                });
            }

            let key = args.into_iter().next().unwrap().try_into_map_key()?;
            let map = receiver.get_mut()?;
            if let Value::Map(map_data) = map {
                let was_present = map_data.shift_remove(&key).is_some();
                Ok(Value::Boolean(was_present))
            } else {
                unreachable!()
            }
        }
        _ => Err(RuntimeError::MethodError {
            message: format!("Map has no method '{}'", method),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::value::MapKey;
    use indexmap::IndexMap;

    #[test]
    fn test_string_length() {
        let s = Value::String("hello".to_string());
        let receiver = ValueRef::Immutable(&s);
        let result = call_method(receiver, "length", vec![]).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_string_split() {
        let s = Value::String("a,b,c".to_string());
        let receiver = ValueRef::Immutable(&s);

        // Split with custom separator
        let result = call_method(receiver, "split", vec![Value::String(",".to_string())]).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::String("a".to_string()));
            assert_eq!(items[1], Value::String("b".to_string()));
            assert_eq!(items[2], Value::String("c".to_string()));
        } else {
            panic!("Expected list");
        }

        // Split with default separator
        let s2 = Value::String("hello world".to_string());
        let receiver2 = ValueRef::Immutable(&s2);
        let result2 = call_method(receiver2, "split", vec![]).unwrap();
        if let Value::List(items) = result2 {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], Value::String("hello".to_string()));
            assert_eq!(items[1], Value::String("world".to_string()));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_list_push_pop() {
        let mut list = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let receiver = ValueRef::Mutable(&mut list);

        // Test push
        let result = call_method(receiver, "push", vec![Value::Number(3.0)]).unwrap();
        assert_eq!(result, Value::Null);

        if let Value::List(items) = &list {
            assert_eq!(items.len(), 3);
            assert_eq!(items[2], Value::Number(3.0));
        } else {
            panic!("Expected list");
        }

        // Test pop
        let receiver2 = ValueRef::Mutable(&mut list);
        let result = call_method(receiver2, "pop", vec![]).unwrap();
        assert_eq!(result, Value::Number(3.0));

        if let Value::List(items) = &list {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_list_join() {
        let list = Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ]);
        let receiver = ValueRef::Immutable(&list);

        // Join with custom separator
        let result = call_method(receiver, "join", vec![Value::String(",".to_string())]).unwrap();
        assert_eq!(result, Value::String("a,b,c".to_string()));

        // Join with default separator
        let receiver2 = ValueRef::Immutable(&list);
        let result2 = call_method(receiver2, "join", vec![]).unwrap();
        assert_eq!(result2, Value::String("a b c".to_string()));
    }

    #[test]
    fn test_map_delete() {
        let mut map_data = IndexMap::new();
        map_data.insert(MapKey::String("a".to_string()), Value::Number(1.0));
        map_data.insert(MapKey::String("b".to_string()), Value::Number(2.0));

        let mut map = Value::Map(map_data);
        let receiver = ValueRef::Mutable(&mut map);

        // Delete existing key
        let result = call_method(receiver, "delete", vec![Value::String("a".to_string())]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        if let Value::Map(map_data) = &map {
            assert_eq!(map_data.len(), 1);
            assert!(!map_data.contains_key(&MapKey::String("a".to_string())));
        } else {
            panic!("Expected map");
        }

        // Delete non-existing key
        let receiver2 = ValueRef::Mutable(&mut map);
        let result2 =
            call_method(receiver2, "delete", vec![Value::String("x".to_string())]).unwrap();
        assert_eq!(result2, Value::Boolean(false));
    }

    #[test]
    fn test_immutable_mutating_method_error() {
        let list = Value::List(vec![Value::Number(1.0)]);
        let receiver = ValueRef::Immutable(&list);

        let result = call_method(receiver, "push", vec![Value::Number(2.0)]);
        assert!(matches!(result, Err(RuntimeError::MethodError { .. })));
    }
}
