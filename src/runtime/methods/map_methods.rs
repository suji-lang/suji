use super::super::value::{DecimalNumber, RuntimeError, Value};
use super::common::ValueRef;

/// Map methods: delete(key), contains(key), keys(), values(), to_list(), length(), get(key, default=nil), merge(other_map)
pub fn call_map_method(
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
        "contains" => {
            if args.len() != 1 {
                return Err(RuntimeError::ArityMismatch {
                    message: "contains() takes exactly one argument".to_string(),
                });
            }

            let key = args[0].clone().try_into_map_key()?;
            if let Value::Map(map_data) = receiver.get() {
                Ok(Value::Boolean(map_data.contains_key(&key)))
            } else {
                unreachable!()
            }
        }
        "keys" => {
            if !args.is_empty() {
                return Err(RuntimeError::MapMethodError {
                    method: "keys".to_string(),
                    message: "keys() takes no arguments".to_string(),
                });
            }
            if let Value::Map(map_data) = receiver.get() {
                let keys: Vec<Value> = map_data.keys().map(|key| key.to_value()).collect();
                Ok(Value::List(keys))
            } else {
                unreachable!()
            }
        }
        "values" => {
            if !args.is_empty() {
                return Err(RuntimeError::MapMethodError {
                    method: "values".to_string(),
                    message: "values() takes no arguments".to_string(),
                });
            }
            if let Value::Map(map_data) = receiver.get() {
                let values: Vec<Value> = map_data.values().cloned().collect();
                Ok(Value::List(values))
            } else {
                unreachable!()
            }
        }
        "to_list" => {
            if !args.is_empty() {
                return Err(RuntimeError::MapMethodError {
                    method: "to_list".to_string(),
                    message: "to_list() takes no arguments".to_string(),
                });
            }
            if let Value::Map(map_data) = receiver.get() {
                let pairs: Vec<Value> = map_data
                    .iter()
                    .map(|(key, value)| Value::Tuple(vec![key.to_value(), value.clone()]))
                    .collect();
                Ok(Value::List(pairs))
            } else {
                unreachable!()
            }
        }
        "length" => {
            if !args.is_empty() {
                return Err(RuntimeError::MapMethodError {
                    method: "length".to_string(),
                    message: "length() takes no arguments".to_string(),
                });
            }
            if let Value::Map(map_data) = receiver.get() {
                Ok(Value::Number(DecimalNumber::from_usize(map_data.len())))
            } else {
                unreachable!()
            }
        }
        "get" => {
            if args.is_empty() || args.len() > 2 {
                return Err(RuntimeError::ArityMismatch {
                    message: "get() takes one or two arguments".to_string(),
                });
            }

            let key = args[0].clone().try_into_map_key()?;
            let default = if args.len() == 2 {
                args[1].clone()
            } else {
                Value::Nil
            };

            if let Value::Map(map_data) = receiver.get() {
                Ok(map_data.get(&key).cloned().unwrap_or(default))
            } else {
                unreachable!()
            }
        }
        "merge" => {
            if args.len() != 1 {
                return Err(RuntimeError::ArityMismatch {
                    message: "merge() takes exactly one argument".to_string(),
                });
            }

            let other_map = match &args[0] {
                Value::Map(other) => other,
                _ => {
                    return Err(RuntimeError::TypeError {
                        message: "merge() argument must be a map".to_string(),
                    });
                }
            };

            let map = receiver.get_mut()?;
            if let Value::Map(map_data) = map {
                for (key, value) in other_map {
                    map_data.insert(key.clone(), value.clone());
                }
                Ok(Value::Nil)
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
            if let Value::Map(map_data) = receiver.get() {
                Ok(Value::String(format!("{}", Value::Map(map_data.clone()))))
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
    use super::super::super::value::{DecimalNumber, MapKey, OrderedDecimal};
    use super::super::common::ValueRef;
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_map_delete() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("a".to_string()),
            Value::Number(DecimalNumber::from_i64(1)),
        );
        map_data.insert(
            MapKey::String("b".to_string()),
            Value::Number(DecimalNumber::from_i64(2)),
        );

        let mut map = Value::Map(map_data);
        let receiver = ValueRef::Mutable(&mut map);

        // Delete existing key
        let result =
            call_map_method(receiver, "delete", vec![Value::String("a".to_string())]).unwrap();
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
            call_map_method(receiver2, "delete", vec![Value::String("x".to_string())]).unwrap();
        assert_eq!(result2, Value::Boolean(false));
    }

    #[test]
    fn test_map_contains_string_keys() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map_data.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(30)),
        );
        map_data.insert(
            MapKey::String("city".to_string()),
            Value::String("New York".to_string()),
        );

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Test existing keys
        let result1 = call_map_method(
            receiver,
            "contains",
            vec![Value::String("name".to_string())],
        )
        .unwrap();
        assert_eq!(result1, Value::Boolean(true));

        let receiver2 = ValueRef::Immutable(&map);
        let result2 = call_map_method(
            receiver2,
            "contains",
            vec![Value::String("age".to_string())],
        )
        .unwrap();
        assert_eq!(result2, Value::Boolean(true));

        // Test non-existing key
        let receiver3 = ValueRef::Immutable(&map);
        let result3 = call_map_method(
            receiver3,
            "contains",
            vec![Value::String("email".to_string())],
        )
        .unwrap();
        assert_eq!(result3, Value::Boolean(false));
    }

    #[test]
    fn test_map_contains_numeric_keys() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(1))),
            Value::Number(DecimalNumber::from_i64(100)),
        );
        map_data.insert(
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(2))),
            Value::Number(DecimalNumber::from_i64(85)),
        );
        map_data.insert(
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(3))),
            Value::Number(DecimalNumber::from_i64(92)),
        );

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Test existing numeric key
        let result1 = call_map_method(
            receiver,
            "contains",
            vec![Value::Number(DecimalNumber::from_i64(2))],
        )
        .unwrap();
        assert_eq!(result1, Value::Boolean(true));

        // Test non-existing numeric key
        let receiver2 = ValueRef::Immutable(&map);
        let result2 = call_map_method(
            receiver2,
            "contains",
            vec![Value::Number(DecimalNumber::from_i64(4))],
        )
        .unwrap();
        assert_eq!(result2, Value::Boolean(false));
    }

    #[test]
    fn test_map_contains_boolean_keys() {
        let mut map_data = IndexMap::new();
        map_data.insert(MapKey::Boolean(true), Value::String("yes".to_string()));
        map_data.insert(MapKey::Boolean(false), Value::String("no".to_string()));

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Test existing boolean key
        let result1 = call_map_method(receiver, "contains", vec![Value::Boolean(true)]).unwrap();
        assert_eq!(result1, Value::Boolean(true));

        // Test existing boolean key (false)
        let receiver2 = ValueRef::Immutable(&map);
        let result2 = call_map_method(receiver2, "contains", vec![Value::Boolean(false)]).unwrap();
        assert_eq!(result2, Value::Boolean(true));
    }

    #[test]
    fn test_map_contains_tuple_keys() {
        let mut map_data = IndexMap::new();
        let key1 = MapKey::Tuple(vec![
            MapKey::String("user".to_string()),
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(1))),
        ]);
        let key2 = MapKey::Tuple(vec![
            MapKey::String("admin".to_string()),
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(2))),
        ]);

        map_data.insert(key1, Value::String("regular user".to_string()));
        map_data.insert(key2, Value::String("administrator".to_string()));

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Test existing tuple key
        let tuple_key = Value::Tuple(vec![
            Value::String("user".to_string()),
            Value::Number(DecimalNumber::from_i64(1)),
        ]);
        let result1 = call_map_method(receiver, "contains", vec![tuple_key]).unwrap();
        assert_eq!(result1, Value::Boolean(true));

        // Test non-existing tuple key
        let receiver2 = ValueRef::Immutable(&map);
        let tuple_key2 = Value::Tuple(vec![
            Value::String("guest".to_string()),
            Value::Number(DecimalNumber::from_i64(3)),
        ]);
        let result2 = call_map_method(receiver2, "contains", vec![tuple_key2]).unwrap();
        assert_eq!(result2, Value::Boolean(false));
    }

    #[test]
    fn test_map_contains_arity_mismatch() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Test no arguments
        let result1 = call_map_method(receiver, "contains", vec![]);
        assert!(matches!(result1, Err(RuntimeError::ArityMismatch { .. })));

        // Test too many arguments
        let receiver2 = ValueRef::Immutable(&map);
        let result2 = call_map_method(
            receiver2,
            "contains",
            vec![
                Value::String("key1".to_string()),
                Value::String("key2".to_string()),
            ],
        );
        assert!(matches!(result2, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_map_contains_invalid_key_type() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Test with invalid key type (list)
        let result = call_map_method(receiver, "contains", vec![Value::List(vec![])]);
        assert!(matches!(result, Err(RuntimeError::InvalidKeyType { .. })));
    }

    #[test]
    fn test_map_keys_method() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map_data.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(30)),
        );
        map_data.insert(
            MapKey::String("city".to_string()),
            Value::String("New York".to_string()),
        );

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "keys", vec![]).unwrap();

        if let Value::List(keys) = result {
            assert_eq!(keys.len(), 3);
            // Order is preserved by IndexMap
            assert_eq!(keys[0], Value::String("name".to_string()));
            assert_eq!(keys[1], Value::String("age".to_string()));
            assert_eq!(keys[2], Value::String("city".to_string()));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_map_values_method() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map_data.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(30)),
        );
        map_data.insert(MapKey::String("active".to_string()), Value::Boolean(true));

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "values", vec![]).unwrap();

        if let Value::List(values) = result {
            assert_eq!(values.len(), 3);
            // Order is preserved by IndexMap
            assert_eq!(values[0], Value::String("Alice".to_string()));
            assert_eq!(values[1], Value::Number(DecimalNumber::from_i64(30)));
            assert_eq!(values[2], Value::Boolean(true));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_map_to_list_method() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map_data.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(30)),
        );

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "to_list", vec![]).unwrap();

        if let Value::List(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // Check first pair
            if let Value::Tuple(first_pair) = &pairs[0] {
                assert_eq!(first_pair[0], Value::String("name".to_string()));
                assert_eq!(first_pair[1], Value::String("Alice".to_string()));
            } else {
                panic!("Expected tuple");
            }
            // Check second pair
            if let Value::Tuple(second_pair) = &pairs[1] {
                assert_eq!(second_pair[0], Value::String("age".to_string()));
                assert_eq!(second_pair[1], Value::Number(DecimalNumber::from_i64(30)));
            } else {
                panic!("Expected tuple");
            }
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_map_length_method() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("a".to_string()),
            Value::Number(DecimalNumber::from_i64(1)),
        );
        map_data.insert(
            MapKey::String("b".to_string()),
            Value::Number(DecimalNumber::from_i64(2)),
        );
        map_data.insert(
            MapKey::String("c".to_string()),
            Value::Number(DecimalNumber::from_i64(3)),
        );

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "length", vec![]).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(3)));
    }

    #[test]
    fn test_map_methods_empty_map() {
        let empty_map = Value::Map(IndexMap::new());

        // Test keys() on empty map
        let receiver1 = ValueRef::Immutable(&empty_map);
        let keys_result = call_map_method(receiver1, "keys", vec![]).unwrap();
        if let Value::List(keys) = keys_result {
            assert_eq!(keys.len(), 0);
        } else {
            panic!("Expected list");
        }

        // Test values() on empty map
        let receiver2 = ValueRef::Immutable(&empty_map);
        let values_result = call_map_method(receiver2, "values", vec![]).unwrap();
        if let Value::List(values) = values_result {
            assert_eq!(values.len(), 0);
        } else {
            panic!("Expected list");
        }

        // Test to_list() on empty map
        let receiver3 = ValueRef::Immutable(&empty_map);
        let to_list_result = call_map_method(receiver3, "to_list", vec![]).unwrap();
        if let Value::List(pairs) = to_list_result {
            assert_eq!(pairs.len(), 0);
        } else {
            panic!("Expected list");
        }

        // Test length() on empty map
        let receiver4 = ValueRef::Immutable(&empty_map);
        let length_result = call_map_method(receiver4, "length", vec![]).unwrap();
        assert_eq!(length_result, Value::Number(DecimalNumber::from_i64(0)));
    }

    #[test]
    fn test_map_no_arg_methods_arity_mismatch() {
        let empty_map = Value::Map(IndexMap::new());

        // Test keys() with arguments (should fail)
        let receiver1 = ValueRef::Immutable(&empty_map);
        let keys_result =
            call_map_method(receiver1, "keys", vec![Value::String("arg".to_string())]);
        assert!(matches!(
            keys_result,
            Err(RuntimeError::MapMethodError { .. })
        ));

        // Test values() with arguments (should fail)
        let receiver2 = ValueRef::Immutable(&empty_map);
        let values_result =
            call_map_method(receiver2, "values", vec![Value::String("arg".to_string())]);
        assert!(matches!(
            values_result,
            Err(RuntimeError::MapMethodError { .. })
        ));

        // Test to_list() with arguments (should fail)
        let receiver3 = ValueRef::Immutable(&empty_map);
        let to_list_result =
            call_map_method(receiver3, "to_list", vec![Value::String("arg".to_string())]);
        assert!(matches!(
            to_list_result,
            Err(RuntimeError::MapMethodError { .. })
        ));

        // Test length() with arguments (should fail)
        let receiver4 = ValueRef::Immutable(&empty_map);
        let length_result =
            call_map_method(receiver4, "length", vec![Value::String("arg".to_string())]);
        assert!(matches!(
            length_result,
            Err(RuntimeError::MapMethodError { .. })
        ));
    }

    #[test]
    fn test_map_keys_with_different_key_types() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map_data.insert(
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(1))),
            Value::String("first".to_string()),
        );
        map_data.insert(MapKey::Boolean(true), Value::String("yes".to_string()));

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "keys", vec![]).unwrap();

        if let Value::List(keys) = result {
            assert_eq!(keys.len(), 3);
            assert_eq!(keys[0], Value::String("name".to_string()));
            assert_eq!(keys[1], Value::Number(DecimalNumber::from_i64(1)));
            assert_eq!(keys[2], Value::Boolean(true));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_map_to_list_with_different_key_types() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map_data.insert(
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(42))),
            Value::Number(DecimalNumber::from_i64(100)),
        );

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "to_list", vec![]).unwrap();

        if let Value::List(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // Check first pair
            if let Value::Tuple(first_pair) = &pairs[0] {
                assert_eq!(first_pair[0], Value::String("name".to_string()));
                assert_eq!(first_pair[1], Value::String("Alice".to_string()));
            } else {
                panic!("Expected tuple");
            }
            // Check second pair
            if let Value::Tuple(second_pair) = &pairs[1] {
                assert_eq!(second_pair[0], Value::Number(DecimalNumber::from_i64(42)));
                assert_eq!(second_pair[1], Value::Number(DecimalNumber::from_i64(100)));
            } else {
                panic!("Expected tuple");
            }
        } else {
            panic!("Expected list");
        }
    }

    // Tests for map::get() method
    #[test]
    fn test_map_get_existing_key() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map_data.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(30)),
        );
        map_data.insert(MapKey::String("active".to_string()), Value::Boolean(true));

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Get existing values
        let name =
            call_map_method(receiver, "get", vec![Value::String("name".to_string())]).unwrap();
        assert_eq!(name, Value::String("Alice".to_string()));

        let receiver2 = ValueRef::Immutable(&map);
        let age =
            call_map_method(receiver2, "get", vec![Value::String("age".to_string())]).unwrap();
        assert_eq!(age, Value::Number(DecimalNumber::from_i64(30)));

        let receiver3 = ValueRef::Immutable(&map);
        let active =
            call_map_method(receiver3, "get", vec![Value::String("active".to_string())]).unwrap();
        assert_eq!(active, Value::Boolean(true));
    }

    #[test]
    fn test_map_get_missing_key_with_default() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Get missing key with custom default
        let email = call_map_method(
            receiver,
            "get",
            vec![
                Value::String("email".to_string()),
                Value::String("N/A".to_string()),
            ],
        )
        .unwrap();
        assert_eq!(email, Value::String("N/A".to_string()));

        let receiver2 = ValueRef::Immutable(&map);
        let phone = call_map_method(
            receiver2,
            "get",
            vec![
                Value::String("phone".to_string()),
                Value::String("Unknown".to_string()),
            ],
        )
        .unwrap();
        assert_eq!(phone, Value::String("Unknown".to_string()));
    }

    #[test]
    fn test_map_get_missing_key_with_nil_default() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Get missing key with nil default (implicit)
        let missing =
            call_map_method(receiver, "get", vec![Value::String("missing".to_string())]).unwrap();
        assert_eq!(missing, Value::Nil);
    }

    #[test]
    fn test_map_get_with_different_key_types() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(1))),
            Value::Number(DecimalNumber::from_i64(100)),
        );
        map_data.insert(
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(2))),
            Value::Number(DecimalNumber::from_i64(85)),
        );
        map_data.insert(MapKey::Boolean(true), Value::String("yes".to_string()));

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Get with numeric key
        let score1 = call_map_method(
            receiver,
            "get",
            vec![
                Value::Number(DecimalNumber::from_i64(1)),
                Value::Number(DecimalNumber::from_i64(0)),
            ],
        )
        .unwrap();
        assert_eq!(score1, Value::Number(DecimalNumber::from_i64(100)));

        let receiver2 = ValueRef::Immutable(&map);
        let score4 = call_map_method(
            receiver2,
            "get",
            vec![
                Value::Number(DecimalNumber::from_i64(4)),
                Value::Number(DecimalNumber::from_i64(0)),
            ],
        )
        .unwrap();
        assert_eq!(score4, Value::Number(DecimalNumber::from_i64(0)));

        // Get with boolean key
        let receiver3 = ValueRef::Immutable(&map);
        let yes = call_map_method(
            receiver3,
            "get",
            vec![Value::Boolean(true), Value::String("no".to_string())],
        )
        .unwrap();
        assert_eq!(yes, Value::String("yes".to_string()));
    }

    #[test]
    fn test_map_get_arity_mismatch() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Test no arguments
        let result1 = call_map_method(receiver, "get", vec![]);
        assert!(matches!(result1, Err(RuntimeError::ArityMismatch { .. })));

        // Test too many arguments
        let receiver2 = ValueRef::Immutable(&map);
        let result2 = call_map_method(
            receiver2,
            "get",
            vec![
                Value::String("key1".to_string()),
                Value::String("default1".to_string()),
                Value::String("extra".to_string()),
            ],
        );
        assert!(matches!(result2, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_map_get_invalid_key_type() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Test with invalid key type (list)
        let result = call_map_method(receiver, "get", vec![Value::List(vec![])]);
        assert!(matches!(result, Err(RuntimeError::InvalidKeyType { .. })));
    }

    // Tests for map::merge() method
    #[test]
    fn test_map_merge_basic() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Bob".to_string()),
        );
        map_data.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(25)),
        );

        let mut map = Value::Map(map_data);
        let receiver = ValueRef::Mutable(&mut map);

        let mut other_data = IndexMap::new();
        other_data.insert(
            MapKey::String("city".to_string()),
            Value::String("New York".to_string()),
        );
        other_data.insert(
            MapKey::String("email".to_string()),
            Value::String("bob@example.com".to_string()),
        );

        let other_map = Value::Map(other_data);

        // Merge additional info
        let result = call_map_method(receiver, "merge", vec![other_map]).unwrap();
        assert_eq!(result, Value::Nil);

        // Check that the map was modified
        if let Value::Map(map_data) = &map {
            assert_eq!(map_data.len(), 4);
            assert_eq!(
                map_data.get(&MapKey::String("name".to_string())),
                Some(&Value::String("Bob".to_string()))
            );
            assert_eq!(
                map_data.get(&MapKey::String("age".to_string())),
                Some(&Value::Number(DecimalNumber::from_i64(25)))
            );
            assert_eq!(
                map_data.get(&MapKey::String("city".to_string())),
                Some(&Value::String("New York".to_string()))
            );
            assert_eq!(
                map_data.get(&MapKey::String("email".to_string())),
                Some(&Value::String("bob@example.com".to_string()))
            );
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_map_merge_overwrite_existing() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Bob".to_string()),
        );
        map_data.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(25)),
        );

        let mut map = Value::Map(map_data);
        let receiver = ValueRef::Mutable(&mut map);

        let mut overlay_data = IndexMap::new();
        overlay_data.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(26)),
        );
        overlay_data.insert(
            MapKey::String("city".to_string()),
            Value::String("Boston".to_string()),
        );

        let overlay_map = Value::Map(overlay_data);

        // Merge overlay (should overwrite age)
        let result = call_map_method(receiver, "merge", vec![overlay_map]).unwrap();
        assert_eq!(result, Value::Nil);

        // Check that age was overwritten
        if let Value::Map(map_data) = &map {
            assert_eq!(map_data.len(), 3);
            assert_eq!(
                map_data.get(&MapKey::String("age".to_string())),
                Some(&Value::Number(DecimalNumber::from_i64(26)))
            );
            assert_eq!(
                map_data.get(&MapKey::String("city".to_string())),
                Some(&Value::String("Boston".to_string()))
            );
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_map_merge_with_empty_map() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Charlie".to_string()),
        );

        let mut map = Value::Map(map_data);
        let receiver = ValueRef::Mutable(&mut map);

        let empty_map = Value::Map(IndexMap::new());

        // Merge empty map (should not change anything)
        let result = call_map_method(receiver, "merge", vec![empty_map]).unwrap();
        assert_eq!(result, Value::Nil);

        // Check that the map is unchanged
        if let Value::Map(map_data) = &map {
            assert_eq!(map_data.len(), 1);
            assert_eq!(
                map_data.get(&MapKey::String("name".to_string())),
                Some(&Value::String("Charlie".to_string()))
            );
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_map_merge_empty_map_with_data() {
        let mut map = Value::Map(IndexMap::new());
        let receiver = ValueRef::Mutable(&mut map);

        let mut data = IndexMap::new();
        data.insert(
            MapKey::String("name".to_string()),
            Value::String("Charlie".to_string()),
        );
        data.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(30)),
        );

        let data_map = Value::Map(data);

        // Merge data into empty map
        let result = call_map_method(receiver, "merge", vec![data_map]).unwrap();
        assert_eq!(result, Value::Nil);

        // Check that the map now has the data
        if let Value::Map(map_data) = &map {
            assert_eq!(map_data.len(), 2);
            assert_eq!(
                map_data.get(&MapKey::String("name".to_string())),
                Some(&Value::String("Charlie".to_string()))
            );
            assert_eq!(
                map_data.get(&MapKey::String("age".to_string())),
                Some(&Value::Number(DecimalNumber::from_i64(30)))
            );
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_map_merge_arity_mismatch() {
        let map_data = IndexMap::new();
        let mut map = Value::Map(map_data);
        let receiver = ValueRef::Mutable(&mut map);

        // Test no arguments
        let result1 = call_map_method(receiver, "merge", vec![]);
        assert!(matches!(result1, Err(RuntimeError::ArityMismatch { .. })));

        // Test too many arguments
        let receiver2 = ValueRef::Mutable(&mut map);
        let other_map = Value::Map(IndexMap::new());
        let result2 = call_map_method(
            receiver2,
            "merge",
            vec![other_map, Value::String("extra".to_string())],
        );
        assert!(matches!(result2, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_map_merge_type_error() {
        let map_data = IndexMap::new();
        let mut map = Value::Map(map_data);
        let receiver = ValueRef::Mutable(&mut map);

        // Test with non-map argument
        let result = call_map_method(
            receiver,
            "merge",
            vec![Value::String("not a map".to_string())],
        );
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }

    #[test]
    fn test_map_merge_immutable_error() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        let other_map = Value::Map(IndexMap::new());

        // Test merge on immutable map
        let result = call_map_method(receiver, "merge", vec![other_map]);
        assert!(matches!(result, Err(RuntimeError::MethodError { .. })));
    }

    #[test]
    fn test_map_to_string() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map_data.insert(
            MapKey::String("age".to_string()),
            Value::Number(DecimalNumber::from_i64(30)),
        );

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        let result = call_map_method(receiver, "to_string", vec![]).unwrap();
        if let Value::String(s) = result {
            assert!(s.contains("name: Alice"));
            assert!(s.contains("age: 30"));
            assert!(s.starts_with("{"));
            assert!(s.ends_with("}"));
        } else {
            panic!("Expected string");
        }

        // Test empty map
        let empty_map = Value::Map(IndexMap::new());
        let receiver2 = ValueRef::Immutable(&empty_map);
        let result2 = call_map_method(receiver2, "to_string", vec![]).unwrap();
        assert_eq!(result2, Value::String("{}".to_string()));
    }

    #[test]
    fn test_map_to_string_arity_mismatch() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        let result = call_map_method(
            receiver,
            "to_string",
            vec![Value::Number(DecimalNumber::from_i64(1))],
        );
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }
}
