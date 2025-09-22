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
        "contains" => {
            if args.len() != 1 {
                return Err(RuntimeError::ArityMismatch {
                    message: "contains() takes exactly one argument".to_string(),
                });
            }
            if let Value::List(items) = receiver.get() {
                Ok(Value::Boolean(items.contains(&args[0])))
            } else {
                unreachable!()
            }
        }
        "reverse" => {
            if !args.is_empty() {
                return Err(RuntimeError::ArityMismatch {
                    message: "reverse() takes no arguments".to_string(),
                });
            }
            if let Value::List(items) = receiver.get() {
                let mut reversed_items = items.clone();
                reversed_items.reverse();
                Ok(Value::List(reversed_items))
            } else {
                unreachable!()
            }
        }
        "sort" => {
            if !args.is_empty() {
                return Err(RuntimeError::ArityMismatch {
                    message: "sort() takes no arguments".to_string(),
                });
            }
            if let Value::List(items) = receiver.get() {
                let mut sorted_items = items.clone();
                sorted_items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                Ok(Value::List(sorted_items))
            } else {
                unreachable!()
            }
        }
        "min" => {
            if !args.is_empty() {
                return Err(RuntimeError::ArityMismatch {
                    message: "min() takes no arguments".to_string(),
                });
            }
            if let Value::List(items) = receiver.get() {
                if items.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "min() called on empty list".to_string(),
                    });
                }
                let mut min_val = None;
                for item in items {
                    match item {
                        Value::Number(n) => {
                            if let Some(current_min) = min_val {
                                if n < &current_min {
                                    min_val = Some(*n);
                                }
                            } else {
                                min_val = Some(*n);
                            }
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "min() can only be called on lists of numbers".to_string(),
                            });
                        }
                    }
                }
                Ok(Value::Number(min_val.unwrap()))
            } else {
                unreachable!()
            }
        }
        "max" => {
            if !args.is_empty() {
                return Err(RuntimeError::ArityMismatch {
                    message: "max() takes no arguments".to_string(),
                });
            }
            if let Value::List(items) = receiver.get() {
                if items.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "max() called on empty list".to_string(),
                    });
                }
                let mut max_val = None;
                for item in items {
                    match item {
                        Value::Number(n) => {
                            if let Some(current_max) = max_val {
                                if n > &current_max {
                                    max_val = Some(*n);
                                }
                            } else {
                                max_val = Some(*n);
                            }
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "max() can only be called on lists of numbers".to_string(),
                            });
                        }
                    }
                }
                Ok(Value::Number(max_val.unwrap()))
            } else {
                unreachable!()
            }
        }
        "first" => {
            let default_value = match args.len() {
                0 => Value::Nil,
                1 => args[0].clone(),
                _ => {
                    return Err(RuntimeError::ArityMismatch {
                        message: "first() takes at most one argument (default value)".to_string(),
                    });
                }
            };

            if let Value::List(items) = receiver.get() {
                Ok(items.first().cloned().unwrap_or(default_value))
            } else {
                unreachable!()
            }
        }
        "last" => {
            let default_value = match args.len() {
                0 => Value::Nil,
                1 => args[0].clone(),
                _ => {
                    return Err(RuntimeError::ArityMismatch {
                        message: "last() takes at most one argument (default value)".to_string(),
                    });
                }
            };

            if let Value::List(items) = receiver.get() {
                Ok(items.last().cloned().unwrap_or(default_value))
            } else {
                unreachable!()
            }
        }
        "average" => {
            if !args.is_empty() {
                return Err(RuntimeError::ArityMismatch {
                    message: "average() takes no arguments".to_string(),
                });
            }

            if let Value::List(items) = receiver.get() {
                if items.is_empty() {
                    return Ok(Value::Nil);
                }

                let mut sum = 0.0;
                let mut count = 0;

                for item in items {
                    match item {
                        Value::Number(n) => {
                            sum += n;
                            count += 1;
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "average() requires all elements to be numbers"
                                    .to_string(),
                            });
                        }
                    }
                }

                Ok(Value::Number(sum / count as f64))
            } else {
                unreachable!()
            }
        }
        "to_string" => {
            if !args.is_empty() {
                return Err(RuntimeError::ArityMismatch {
                    message: "to_string() takes no arguments".to_string(),
                });
            }
            if let Value::List(items) = receiver.get() {
                Ok(Value::String(format!("{}", Value::List(items.clone()))))
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

    #[test]
    fn test_list_contains() {
        let list = Value::List(vec![
            Value::String("apple".to_string()),
            Value::String("banana".to_string()),
            Value::String("cherry".to_string()),
        ]);
        let receiver = ValueRef::Immutable(&list);

        // Test contains with existing item
        let result = call_list_method(
            receiver,
            "contains",
            vec![Value::String("banana".to_string())],
        )
        .unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test contains with non-existing item
        let receiver2 = ValueRef::Immutable(&list);
        let result2 = call_list_method(
            receiver2,
            "contains",
            vec![Value::String("grape".to_string())],
        )
        .unwrap();
        assert_eq!(result2, Value::Boolean(false));

        // Test contains with numbers
        let numbers = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let receiver3 = ValueRef::Immutable(&numbers);
        let result3 = call_list_method(receiver3, "contains", vec![Value::Number(2.0)]).unwrap();
        assert_eq!(result3, Value::Boolean(true));
    }

    #[test]
    fn test_list_reverse() {
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let receiver = ValueRef::Immutable(&list);

        let result = call_list_method(receiver, "reverse", vec![]).unwrap();
        if let Value::List(reversed) = result {
            assert_eq!(
                reversed,
                vec![Value::Number(3.0), Value::Number(2.0), Value::Number(1.0),]
            );
        } else {
            panic!("Expected list");
        }

        // Ensure original list is unchanged
        if let Value::List(original) = &list {
            assert_eq!(
                original,
                &vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),]
            );
        }

        // Test with empty list
        let empty = Value::List(vec![]);
        let receiver2 = ValueRef::Immutable(&empty);
        let result2 = call_list_method(receiver2, "reverse", vec![]).unwrap();
        if let Value::List(reversed_empty) = result2 {
            assert_eq!(reversed_empty, vec![]);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_list_sort() {
        let list = Value::List(vec![
            Value::Number(3.0),
            Value::Number(1.0),
            Value::Number(4.0),
            Value::Number(1.0),
            Value::Number(5.0),
        ]);
        let receiver = ValueRef::Immutable(&list);

        let result = call_list_method(receiver, "sort", vec![]).unwrap();
        if let Value::List(sorted) = result {
            assert_eq!(
                sorted,
                vec![
                    Value::Number(1.0),
                    Value::Number(1.0),
                    Value::Number(3.0),
                    Value::Number(4.0),
                    Value::Number(5.0),
                ]
            );
        } else {
            panic!("Expected list");
        }

        // Test with strings
        let words = Value::List(vec![
            Value::String("zebra".to_string()),
            Value::String("apple".to_string()),
            Value::String("banana".to_string()),
        ]);
        let receiver2 = ValueRef::Immutable(&words);
        let result2 = call_list_method(receiver2, "sort", vec![]).unwrap();
        if let Value::List(sorted_words) = result2 {
            assert_eq!(
                sorted_words,
                vec![
                    Value::String("apple".to_string()),
                    Value::String("banana".to_string()),
                    Value::String("zebra".to_string()),
                ]
            );
        } else {
            panic!("Expected list");
        }

        // Test with booleans
        let bools = Value::List(vec![
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Boolean(true),
            Value::Boolean(false),
        ]);
        let receiver3 = ValueRef::Immutable(&bools);
        let result3 = call_list_method(receiver3, "sort", vec![]).unwrap();
        if let Value::List(sorted_bools) = result3 {
            assert_eq!(
                sorted_bools,
                vec![
                    Value::Boolean(false),
                    Value::Boolean(false),
                    Value::Boolean(true),
                    Value::Boolean(true),
                ]
            );
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_list_min() {
        let list = Value::List(vec![
            Value::Number(85.0),
            Value::Number(92.0),
            Value::Number(78.0),
            Value::Number(96.0),
            Value::Number(88.0),
        ]);
        let receiver = ValueRef::Immutable(&list);

        let result = call_list_method(receiver, "min", vec![]).unwrap();
        assert_eq!(result, Value::Number(78.0));

        // Test with negative numbers
        let temps = Value::List(vec![
            Value::Number(-5.0),
            Value::Number(10.0),
            Value::Number(-15.0),
            Value::Number(20.0),
            Value::Number(0.0),
        ]);
        let receiver2 = ValueRef::Immutable(&temps);
        let result2 = call_list_method(receiver2, "min", vec![]).unwrap();
        assert_eq!(result2, Value::Number(-15.0));
    }

    #[test]
    fn test_list_max() {
        let list = Value::List(vec![
            Value::Number(85.0),
            Value::Number(92.0),
            Value::Number(78.0),
            Value::Number(96.0),
            Value::Number(88.0),
        ]);
        let receiver = ValueRef::Immutable(&list);

        let result = call_list_method(receiver, "max", vec![]).unwrap();
        assert_eq!(result, Value::Number(96.0));

        // Test with negative numbers
        let temps = Value::List(vec![
            Value::Number(-5.0),
            Value::Number(10.0),
            Value::Number(-15.0),
            Value::Number(20.0),
            Value::Number(0.0),
        ]);
        let receiver2 = ValueRef::Immutable(&temps);
        let result2 = call_list_method(receiver2, "max", vec![]).unwrap();
        assert_eq!(result2, Value::Number(20.0));
    }

    #[test]
    fn test_list_min_max_errors() {
        // Test empty list errors for both min and max
        let empty_list = Value::List(vec![]);

        let receiver1 = ValueRef::Immutable(&empty_list);
        let min_result = call_list_method(receiver1, "min", vec![]);
        assert!(matches!(
            min_result,
            Err(RuntimeError::InvalidOperation { .. })
        ));

        let receiver2 = ValueRef::Immutable(&empty_list);
        let max_result = call_list_method(receiver2, "max", vec![]);
        assert!(matches!(
            max_result,
            Err(RuntimeError::InvalidOperation { .. })
        ));

        // Test non-number type errors for both min and max
        let mixed_list = Value::List(vec![
            Value::Number(1.0),
            Value::String("not a number".to_string()),
            Value::Number(3.0),
        ]);

        let receiver3 = ValueRef::Immutable(&mixed_list);
        let min_type_result = call_list_method(receiver3, "min", vec![]);
        assert!(matches!(
            min_type_result,
            Err(RuntimeError::TypeError { .. })
        ));

        let receiver4 = ValueRef::Immutable(&mixed_list);
        let max_type_result = call_list_method(receiver4, "max", vec![]);
        assert!(matches!(
            max_type_result,
            Err(RuntimeError::TypeError { .. })
        ));
    }

    #[test]
    fn test_list_first() {
        let list = Value::List(vec![
            Value::String("first".to_string()),
            Value::String("middle".to_string()),
            Value::String("last".to_string()),
        ]);
        let receiver = ValueRef::Immutable(&list);

        let result = call_list_method(receiver, "first", vec![]).unwrap();
        assert_eq!(result, Value::String("first".to_string()));

        // Test with numbers
        let numbers = Value::List(vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(30.0),
        ]);
        let receiver2 = ValueRef::Immutable(&numbers);
        let result2 = call_list_method(receiver2, "first", vec![]).unwrap();
        assert_eq!(result2, Value::Number(10.0));
    }

    #[test]
    fn test_list_last() {
        let list = Value::List(vec![
            Value::String("first".to_string()),
            Value::String("middle".to_string()),
            Value::String("last".to_string()),
        ]);
        let receiver = ValueRef::Immutable(&list);

        let result = call_list_method(receiver, "last", vec![]).unwrap();
        assert_eq!(result, Value::String("last".to_string()));

        // Test with numbers
        let numbers = Value::List(vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(30.0),
        ]);
        let receiver2 = ValueRef::Immutable(&numbers);
        let result2 = call_list_method(receiver2, "last", vec![]).unwrap();
        assert_eq!(result2, Value::Number(30.0));
    }

    #[test]
    fn test_list_first_last_empty() {
        let empty_list = Value::List(vec![]);

        // Test first() on empty list returns nil
        let receiver1 = ValueRef::Immutable(&empty_list);
        let first_result = call_list_method(receiver1, "first", vec![]).unwrap();
        assert_eq!(first_result, Value::Nil);

        // Test last() on empty list returns nil
        let receiver2 = ValueRef::Immutable(&empty_list);
        let last_result = call_list_method(receiver2, "last", vec![]).unwrap();
        assert_eq!(last_result, Value::Nil);
    }

    #[test]
    fn test_list_to_string() {
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::String("test".to_string()),
            Value::Boolean(true),
        ]);
        let receiver = ValueRef::Immutable(&list);

        let result = call_list_method(receiver, "to_string", vec![]).unwrap();
        assert_eq!(result, Value::String("[1, test, true]".to_string()));

        // Test empty list
        let empty_list = Value::List(vec![]);
        let receiver2 = ValueRef::Immutable(&empty_list);
        let result2 = call_list_method(receiver2, "to_string", vec![]).unwrap();
        assert_eq!(result2, Value::String("[]".to_string()));
    }

    #[test]
    fn test_list_to_string_arity_mismatch() {
        let list = Value::List(vec![Value::Number(1.0)]);
        let receiver = ValueRef::Immutable(&list);

        let result = call_list_method(receiver, "to_string", vec![Value::Number(1.0)]);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_list_first_with_default() {
        // Test non-empty list ignores default
        let list = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let receiver = ValueRef::Immutable(&list);
        let result = call_list_method(receiver, "first", vec![Value::Number(99.0)]).unwrap();
        assert_eq!(result, Value::Number(1.0));

        // Test empty list uses default
        let empty = Value::List(vec![]);
        let receiver2 = ValueRef::Immutable(&empty);
        let result2 = call_list_method(
            receiver2,
            "first",
            vec![Value::String("default".to_string())],
        )
        .unwrap();
        assert_eq!(result2, Value::String("default".to_string()));

        // Test empty list without default returns nil
        let receiver3 = ValueRef::Immutable(&empty);
        let result3 = call_list_method(receiver3, "first", vec![]).unwrap();
        assert_eq!(result3, Value::Nil);
    }

    #[test]
    fn test_list_last_with_default() {
        // Test non-empty list ignores default
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let receiver = ValueRef::Immutable(&list);
        let result = call_list_method(receiver, "last", vec![Value::Number(99.0)]).unwrap();
        assert_eq!(result, Value::Number(3.0));

        // Test empty list uses default
        let empty = Value::List(vec![]);
        let receiver2 = ValueRef::Immutable(&empty);
        let result2 = call_list_method(
            receiver2,
            "last",
            vec![Value::String("default".to_string())],
        )
        .unwrap();
        assert_eq!(result2, Value::String("default".to_string()));

        // Test empty list without default returns nil
        let receiver3 = ValueRef::Immutable(&empty);
        let result3 = call_list_method(receiver3, "last", vec![]).unwrap();
        assert_eq!(result3, Value::Nil);
    }

    #[test]
    fn test_list_average() {
        // Test normal case
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
        let receiver = ValueRef::Immutable(&list);
        let result = call_list_method(receiver, "average", vec![]).unwrap();
        assert_eq!(result, Value::Number(2.5));

        // Test single element
        let single = Value::List(vec![Value::Number(10.0)]);
        let receiver2 = ValueRef::Immutable(&single);
        let result2 = call_list_method(receiver2, "average", vec![]).unwrap();
        assert_eq!(result2, Value::Number(10.0));

        // Test empty list
        let empty = Value::List(vec![]);
        let receiver3 = ValueRef::Immutable(&empty);
        let result3 = call_list_method(receiver3, "average", vec![]).unwrap();
        assert_eq!(result3, Value::Nil);
    }

    #[test]
    fn test_list_average_mixed_types() {
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::String("not a number".to_string()),
            Value::Number(3.0),
        ]);
        let receiver = ValueRef::Immutable(&list);
        let result = call_list_method(receiver, "average", vec![]);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }

    #[test]
    fn test_list_first_last_arity_errors() {
        let list = Value::List(vec![Value::Number(1.0)]);
        let receiver = ValueRef::Immutable(&list);

        // Test too many arguments for first
        let result = call_list_method(
            receiver,
            "first",
            vec![Value::Number(1.0), Value::Number(2.0)],
        );
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));

        let receiver2 = ValueRef::Immutable(&list);
        // Test too many arguments for last
        let result2 = call_list_method(
            receiver2,
            "last",
            vec![Value::Number(1.0), Value::Number(2.0)],
        );
        assert!(matches!(result2, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_list_average_arity_error() {
        let list = Value::List(vec![Value::Number(1.0)]);
        let receiver = ValueRef::Immutable(&list);

        // Test too many arguments for average
        let result = call_list_method(receiver, "average", vec![Value::Number(1.0)]);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }
}
