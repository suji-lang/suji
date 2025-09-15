use super::super::value::{RuntimeError, Value};
use super::common::{ValueRef, eval_closure};

/// List methods: push(item), pop(), length(), join(separator=" "), index_of(), filter(), map(), fold(), sum(), product()
pub fn call_list_method(
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

#[cfg(test)]
mod tests {
    use super::super::common::ValueRef;
    use super::*;

    #[test]
    fn test_list_push_pop() {
        let mut list = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let receiver = ValueRef::Mutable(&mut list);

        // Test push
        let result = call_list_method(receiver, "push", vec![Value::Number(3.0)]).unwrap();
        assert_eq!(result, Value::Nil);

        if let Value::List(items) = &list {
            assert_eq!(items.len(), 3);
            assert_eq!(items[2], Value::Number(3.0));
        } else {
            panic!("Expected list");
        }

        // Test pop
        let receiver2 = ValueRef::Mutable(&mut list);
        let result = call_list_method(receiver2, "pop", vec![]).unwrap();
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
        let result =
            call_list_method(receiver, "join", vec![Value::String(",".to_string())]).unwrap();
        assert_eq!(result, Value::String("a,b,c".to_string()));

        // Join with default separator
        let receiver2 = ValueRef::Immutable(&list);
        let result2 = call_list_method(receiver2, "join", vec![]).unwrap();
        assert_eq!(result2, Value::String("a b c".to_string()));
    }

    #[test]
    fn test_list_index_of() {
        let list = Value::List(vec![
            Value::String("apple".to_string()),
            Value::String("banana".to_string()),
            Value::String("cherry".to_string()),
        ]);
        let receiver = ValueRef::Immutable(&list);

        let result = call_list_method(
            receiver,
            "index_of",
            vec![Value::String("banana".to_string())],
        )
        .unwrap();
        assert_eq!(result, Value::Number(1.0));

        let receiver2 = ValueRef::Immutable(&list);
        let result2 = call_list_method(
            receiver2,
            "index_of",
            vec![Value::String("grape".to_string())],
        )
        .unwrap();
        assert_eq!(result2, Value::Number(-1.0));
    }

    #[test]
    fn test_list_sum() {
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
        let receiver = ValueRef::Immutable(&list);
        let result = call_list_method(receiver, "sum", vec![]).unwrap();
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
        let result = call_list_method(receiver, "product", vec![]).unwrap();
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
        let result = call_list_method(receiver, "sum", vec![]);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }
}
