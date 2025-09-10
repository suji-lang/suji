use super::value::{RuntimeError, Value};
use crate::runtime::env::Env;
use std::rc::Rc;

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
        Value::Number(_) => call_number_method(receiver, method, args),
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
            "to_number" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_number() takes no arguments".to_string(),
                    });
                }
                s.parse::<f64>().map(Value::Number).map_err(|_| {
                    RuntimeError::InvalidNumberConversion {
                        message: format!("Cannot convert '{}' to number", s),
                    }
                })
            }
            "to_list" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_list() takes no arguments".to_string(),
                    });
                }
                let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
                Ok(Value::List(chars))
            }
            "index_of" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        message: "index_of() takes exactly one argument".to_string(),
                    });
                }
                match &args[0] {
                    Value::String(substring) => {
                        let index = s.find(substring).map(|i| i as f64).unwrap_or(-1.0);
                        Ok(Value::Number(index))
                    }
                    _ => Err(RuntimeError::TypeError {
                        message: "index_of() argument must be a string".to_string(),
                    }),
                }
            }
            _ => Err(RuntimeError::MethodError {
                message: format!("String has no method '{}'", method),
            }),
        },
        _ => unreachable!(),
    }
}

/// Number methods: to_string(), is_int()
fn call_number_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::Number(n) => match method {
            "to_string" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_string() takes no arguments".to_string(),
                    });
                }
                Ok(Value::String(n.to_string()))
            }
            "is_int" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "is_int() takes no arguments".to_string(),
                    });
                }
                Ok(Value::Boolean(n.fract() == 0.0))
            }
            _ => Err(RuntimeError::MethodError {
                message: format!("Number has no method '{}'", method),
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
                Ok(Value::Nil)
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
        "index_of" => {
            if args.len() != 1 {
                return Err(RuntimeError::ArityMismatch {
                    message: "index_of() takes exactly one argument".to_string(),
                });
            }
            if let Value::List(items) = receiver.get() {
                let search_value = &args[0];
                for (i, item) in items.iter().enumerate() {
                    if item == search_value {
                        return Ok(Value::Number(i as f64));
                    }
                }
                Ok(Value::Number(-1.0))
            } else {
                unreachable!()
            }
        }
        "filter" => {
            if args.len() != 1 {
                return Err(RuntimeError::ArityMismatch {
                    message: "filter() takes exactly one argument".to_string(),
                });
            }
            if let Value::List(items) = receiver.get() {
                let closure = &args[0];
                let mut filtered = Vec::new();
                for item in items {
                    if eval_closure(closure, vec![item.clone()])?.is_truthy() {
                        filtered.push(item.clone());
                    }
                }
                Ok(Value::List(filtered))
            } else {
                unreachable!()
            }
        }
        "map" => {
            if args.len() != 1 {
                return Err(RuntimeError::ArityMismatch {
                    message: "map() takes exactly one argument".to_string(),
                });
            }
            if let Value::List(items) = receiver.get() {
                let closure = &args[0];
                let mut mapped = Vec::new();
                for item in items {
                    mapped.push(eval_closure(closure, vec![item.clone()])?);
                }
                Ok(Value::List(mapped))
            } else {
                unreachable!()
            }
        }
        "fold" => {
            if args.len() != 2 {
                return Err(RuntimeError::ArityMismatch {
                    message: "fold() takes exactly two arguments".to_string(),
                });
            }
            if let Value::List(items) = receiver.get() {
                let mut accumulator = args[0].clone();
                let closure = &args[1];
                for item in items {
                    accumulator = eval_closure(closure, vec![accumulator, item.clone()])?;
                }
                Ok(accumulator)
            } else {
                unreachable!()
            }
        }
        "sum" => {
            if !args.is_empty() {
                return Err(RuntimeError::ArityMismatch {
                    message: "sum() takes no arguments".to_string(),
                });
            }
            if let Value::List(items) = receiver.get() {
                let mut sum = 0.0;
                for item in items {
                    match item {
                        Value::Number(n) => sum += n,
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "sum() can only be called on lists of numbers".to_string(),
                            });
                        }
                    }
                }
                Ok(Value::Number(sum))
            } else {
                unreachable!()
            }
        }
        "product" => {
            if !args.is_empty() {
                return Err(RuntimeError::ArityMismatch {
                    message: "product() takes no arguments".to_string(),
                });
            }
            if let Value::List(items) = receiver.get() {
                let mut product = 1.0;
                for item in items {
                    match item {
                        Value::Number(n) => product *= n,
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "product() can only be called on lists of numbers"
                                    .to_string(),
                            });
                        }
                    }
                }
                Ok(Value::Number(product))
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

/// Evaluate a closure (function value) with the given arguments
fn eval_closure(closure: &Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
    match closure {
        Value::Function(func) => {
            // Create a new environment for the closure
            let call_env = Rc::new(Env::new_child(func.env.clone()));

            // Bind arguments to parameters
            for (param, arg_value) in func.params.iter().zip(args) {
                call_env.define_or_set(&param.name, arg_value);
            }

            // Execute the function body
            let mut loop_stack = Vec::new();
            match super::eval::eval_stmt(&func.body, call_env, &mut loop_stack) {
                Ok(result) => Ok(result.unwrap_or(Value::Nil)),
                Err(RuntimeError::ControlFlow {
                    flow: super::value::ControlFlow::Return(value),
                }) => Ok(*value),
                Err(other_error) => Err(other_error),
            }
        }
        _ => Err(RuntimeError::TypeError {
            message: "Expected a function for closure evaluation".to_string(),
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
        assert_eq!(result, Value::Nil);

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

    // Cast method tests
    #[test]
    fn test_string_to_number() {
        let s = Value::String("123".to_string());
        let receiver = ValueRef::Immutable(&s);
        let result = call_method(receiver, "to_number", vec![]).unwrap();
        assert_eq!(result, Value::Number(123.0));

        let s2 = Value::String("123.45".to_string());
        let receiver2 = ValueRef::Immutable(&s2);
        let result2 = call_method(receiver2, "to_number", vec![]).unwrap();
        assert_eq!(result2, Value::Number(123.45));

        let s3 = Value::String("invalid".to_string());
        let receiver3 = ValueRef::Immutable(&s3);
        let result3 = call_method(receiver3, "to_number", vec![]);
        assert!(matches!(
            result3,
            Err(RuntimeError::InvalidNumberConversion { .. })
        ));
    }

    #[test]
    fn test_string_to_list() {
        let s = Value::String("hello".to_string());
        let receiver = ValueRef::Immutable(&s);
        let result = call_method(receiver, "to_list", vec![]).unwrap();

        if let Value::List(chars) = result {
            assert_eq!(chars.len(), 5);
            assert_eq!(chars[0], Value::String("h".to_string()));
            assert_eq!(chars[1], Value::String("e".to_string()));
            assert_eq!(chars[2], Value::String("l".to_string()));
            assert_eq!(chars[3], Value::String("l".to_string()));
            assert_eq!(chars[4], Value::String("o".to_string()));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_number_to_string() {
        let n = Value::Number(42.0);
        let receiver = ValueRef::Immutable(&n);
        let result = call_method(receiver, "to_string", vec![]).unwrap();
        assert_eq!(result, Value::String("42".to_string()));

        let n2 = Value::Number(3.14);
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_method(receiver2, "to_string", vec![]).unwrap();
        assert_eq!(result2, Value::String("3.14".to_string()));
    }

    #[test]
    fn test_number_is_int() {
        let n = Value::Number(42.0);
        let receiver = ValueRef::Immutable(&n);
        let result = call_method(receiver, "is_int", vec![]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        let n2 = Value::Number(3.14);
        let receiver2 = ValueRef::Immutable(&n2);
        let result2 = call_method(receiver2, "is_int", vec![]).unwrap();
        assert_eq!(result2, Value::Boolean(false));
    }

    // Search method tests
    #[test]
    fn test_string_index_of() {
        let s = Value::String("hello world".to_string());
        let receiver = ValueRef::Immutable(&s);

        let result = call_method(
            receiver,
            "index_of",
            vec![Value::String("world".to_string())],
        )
        .unwrap();
        assert_eq!(result, Value::Number(6.0));

        let receiver2 = ValueRef::Immutable(&s);
        let result2 = call_method(
            receiver2,
            "index_of",
            vec![Value::String("xyz".to_string())],
        )
        .unwrap();
        assert_eq!(result2, Value::Number(-1.0));
    }

    #[test]
    fn test_list_index_of() {
        let list = Value::List(vec![
            Value::String("apple".to_string()),
            Value::String("banana".to_string()),
            Value::String("cherry".to_string()),
        ]);
        let receiver = ValueRef::Immutable(&list);

        let result = call_method(
            receiver,
            "index_of",
            vec![Value::String("banana".to_string())],
        )
        .unwrap();
        assert_eq!(result, Value::Number(1.0));

        let receiver2 = ValueRef::Immutable(&list);
        let result2 = call_method(
            receiver2,
            "index_of",
            vec![Value::String("grape".to_string())],
        )
        .unwrap();
        assert_eq!(result2, Value::Number(-1.0));
    }

    // Iterator method tests (simplified - these would need more complex setup for closures)
    #[test]
    fn test_list_sum() {
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
        let receiver = ValueRef::Immutable(&list);
        let result = call_method(receiver, "sum", vec![]).unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_list_product() {
        let list = Value::List(vec![
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
        let receiver = ValueRef::Immutable(&list);
        let result = call_method(receiver, "product", vec![]).unwrap();
        assert_eq!(result, Value::Number(24.0));
    }

    #[test]
    fn test_list_sum_with_non_numbers() {
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::String("not a number".to_string()),
            Value::Number(3.0),
        ]);
        let receiver = ValueRef::Immutable(&list);
        let result = call_method(receiver, "sum", vec![]);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }
}
