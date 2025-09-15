use super::super::value::{RuntimeError, Value};
use super::common::ValueRef;

/// Map methods: delete(key), contains(key), keys(), values(), to_list(), length()
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
                Ok(Value::Number(map_data.len() as f64))
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
    use super::super::super::value::{MapKey, OrderedFloat};
    use super::super::common::ValueRef;
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_map_delete() {
        let mut map_data = IndexMap::new();
        map_data.insert(MapKey::String("a".to_string()), Value::Number(1.0));
        map_data.insert(MapKey::String("b".to_string()), Value::Number(2.0));

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
        map_data.insert(MapKey::String("age".to_string()), Value::Number(30.0));
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
        map_data.insert(MapKey::Number(OrderedFloat(1.0)), Value::Number(100.0));
        map_data.insert(MapKey::Number(OrderedFloat(2.0)), Value::Number(85.0));
        map_data.insert(MapKey::Number(OrderedFloat(3.0)), Value::Number(92.0));

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Test existing numeric key
        let result1 = call_map_method(receiver, "contains", vec![Value::Number(2.0)]).unwrap();
        assert_eq!(result1, Value::Boolean(true));

        // Test non-existing numeric key
        let receiver2 = ValueRef::Immutable(&map);
        let result2 = call_map_method(receiver2, "contains", vec![Value::Number(4.0)]).unwrap();
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
            MapKey::Number(OrderedFloat(1.0)),
        ]);
        let key2 = MapKey::Tuple(vec![
            MapKey::String("admin".to_string()),
            MapKey::Number(OrderedFloat(2.0)),
        ]);

        map_data.insert(key1, Value::String("regular user".to_string()));
        map_data.insert(key2, Value::String("administrator".to_string()));

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        // Test existing tuple key
        let tuple_key = Value::Tuple(vec![Value::String("user".to_string()), Value::Number(1.0)]);
        let result1 = call_map_method(receiver, "contains", vec![tuple_key]).unwrap();
        assert_eq!(result1, Value::Boolean(true));

        // Test non-existing tuple key
        let receiver2 = ValueRef::Immutable(&map);
        let tuple_key2 = Value::Tuple(vec![Value::String("guest".to_string()), Value::Number(3.0)]);
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
        map_data.insert(MapKey::String("age".to_string()), Value::Number(30.0));
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
        map_data.insert(MapKey::String("age".to_string()), Value::Number(30.0));
        map_data.insert(MapKey::String("active".to_string()), Value::Boolean(true));

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "values", vec![]).unwrap();

        if let Value::List(values) = result {
            assert_eq!(values.len(), 3);
            // Order is preserved by IndexMap
            assert_eq!(values[0], Value::String("Alice".to_string()));
            assert_eq!(values[1], Value::Number(30.0));
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
        map_data.insert(MapKey::String("age".to_string()), Value::Number(30.0));

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
                assert_eq!(second_pair[1], Value::Number(30.0));
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
        map_data.insert(MapKey::String("a".to_string()), Value::Number(1.0));
        map_data.insert(MapKey::String("b".to_string()), Value::Number(2.0));
        map_data.insert(MapKey::String("c".to_string()), Value::Number(3.0));

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "length", vec![]).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_map_keys_empty_map() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "keys", vec![]).unwrap();

        if let Value::List(keys) = result {
            assert_eq!(keys.len(), 0);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_map_values_empty_map() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "values", vec![]).unwrap();

        if let Value::List(values) = result {
            assert_eq!(values.len(), 0);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_map_to_list_empty_map() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "to_list", vec![]).unwrap();

        if let Value::List(pairs) = result {
            assert_eq!(pairs.len(), 0);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_map_length_empty_map() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "length", vec![]).unwrap();
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_map_keys_arity_mismatch() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        let result = call_map_method(receiver, "keys", vec![Value::String("arg".to_string())]);
        assert!(matches!(result, Err(RuntimeError::MapMethodError { .. })));
    }

    #[test]
    fn test_map_values_arity_mismatch() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        let result = call_map_method(receiver, "values", vec![Value::String("arg".to_string())]);
        assert!(matches!(result, Err(RuntimeError::MapMethodError { .. })));
    }

    #[test]
    fn test_map_to_list_arity_mismatch() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        let result = call_map_method(receiver, "to_list", vec![Value::String("arg".to_string())]);
        assert!(matches!(result, Err(RuntimeError::MapMethodError { .. })));
    }

    #[test]
    fn test_map_length_arity_mismatch() {
        let map_data = IndexMap::new();
        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);

        let result = call_map_method(receiver, "length", vec![Value::String("arg".to_string())]);
        assert!(matches!(result, Err(RuntimeError::MapMethodError { .. })));
    }

    #[test]
    fn test_map_keys_with_different_key_types() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map_data.insert(
            MapKey::Number(OrderedFloat(1.0)),
            Value::String("first".to_string()),
        );
        map_data.insert(MapKey::Boolean(true), Value::String("yes".to_string()));

        let map = Value::Map(map_data);
        let receiver = ValueRef::Immutable(&map);
        let result = call_map_method(receiver, "keys", vec![]).unwrap();

        if let Value::List(keys) = result {
            assert_eq!(keys.len(), 3);
            assert_eq!(keys[0], Value::String("name".to_string()));
            assert_eq!(keys[1], Value::Number(1.0));
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
        map_data.insert(MapKey::Number(OrderedFloat(42.0)), Value::Number(100.0));

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
                assert_eq!(second_pair[0], Value::Number(42.0));
                assert_eq!(second_pair[1], Value::Number(100.0));
            } else {
                panic!("Expected tuple");
            }
        } else {
            panic!("Expected list");
        }
    }
}
