use super::value::{RuntimeError, Value};

/// Expand a range (start..end) into a list with half-open semantics
/// - Includes start, excludes end
/// - If start >= end, returns empty list
/// - Supports negative numbers
pub fn expand_range(start: f64, end: f64) -> Result<Value, RuntimeError> {
    // Check if start and end are integer values
    if start.fract() != 0.0 || end.fract() != 0.0 {
        return Err(RuntimeError::TypeError {
            message: "Range bounds must be integers".to_string(),
        });
    }

    let start_int = start as i64;
    let end_int = end as i64;

    // Handle empty range: start == end
    if start_int == end_int {
        return Ok(Value::List(vec![]));
    }

    // Handle descending ranges: start > end
    if start_int > end_int {
        let range_size = (start_int - end_int) as u64;
        if range_size > 1_000_000 {
            return Err(RuntimeError::InvalidOperation {
                message: format!("Range too large: {} elements", range_size),
            });
        }

        let values: Vec<Value> = (end_int + 1..start_int + 1)
            .rev()
            .map(|i| Value::Number(i as f64))
            .collect();
        return Ok(Value::List(values));
    }

    // Check for reasonable range size to prevent memory issues
    let range_size = (end_int - start_int) as u64;
    if range_size > 1_000_000 {
        return Err(RuntimeError::InvalidOperation {
            message: format!("Range too large: {} elements", range_size),
        });
    }

    let values: Vec<Value> = (start_int..end_int)
        .map(|i| Value::Number(i as f64))
        .collect();

    Ok(Value::List(values))
}

/// Convenience function to expand range from two Values
pub fn expand_range_values(start_val: &Value, end_val: &Value) -> Result<Value, RuntimeError> {
    let start = match start_val {
        Value::Number(n) => *n,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Range start must be a number, got {}",
                    start_val.type_name()
                ),
            });
        }
    };

    let end = match end_val {
        Value::Number(n) => *n,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!("Range end must be a number, got {}", end_val.type_name()),
            });
        }
    };

    expand_range(start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_range() {
        let result = expand_range(0.0, 3.0).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Number(0.0));
            assert_eq!(items[1], Value::Number(1.0));
            assert_eq!(items[2], Value::Number(2.0));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_negative_range() {
        let result = expand_range(-2.0, 2.0).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 4);
            assert_eq!(items[0], Value::Number(-2.0));
            assert_eq!(items[1], Value::Number(-1.0));
            assert_eq!(items[2], Value::Number(0.0));
            assert_eq!(items[3], Value::Number(1.0));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_empty_range_equal() {
        let result = expand_range(5.0, 5.0).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 0);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_descending_range_basic() {
        let result = expand_range(5.0, 2.0).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Number(5.0));
            assert_eq!(items[1], Value::Number(4.0));
            assert_eq!(items[2], Value::Number(3.0));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_single_element_range() {
        let result = expand_range(10.0, 11.0).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], Value::Number(10.0));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_large_negative_range() {
        let result = expand_range(-10.0, -5.0).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 5);
            assert_eq!(items[0], Value::Number(-10.0));
            assert_eq!(items[4], Value::Number(-6.0));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_non_integer_bounds_error() {
        let result = expand_range(1.5, 3.0);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));

        let result = expand_range(1.0, 3.7);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }

    #[test]
    fn test_expand_range_values() {
        let start = Value::Number(0.0);
        let end = Value::Number(3.0);

        let result = expand_range_values(&start, &end).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Number(0.0));
            assert_eq!(items[1], Value::Number(1.0));
            assert_eq!(items[2], Value::Number(2.0));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_expand_range_values_type_error() {
        let start = Value::String("not a number".to_string());
        let end = Value::Number(3.0);

        let result = expand_range_values(&start, &end);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));

        let start = Value::Number(0.0);
        let end = Value::Boolean(true);

        let result = expand_range_values(&start, &end);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }

    #[test]
    fn test_range_size_limit() {
        // This should exceed our limit of 1,000,000 elements
        let result = expand_range(0.0, 2_000_000.0);
        assert!(matches!(result, Err(RuntimeError::InvalidOperation { .. })));
    }

    #[test]
    fn test_range_from_spec_examples() {
        // Example from spec: 0..3 → [0, 1, 2]
        let result = expand_range(0.0, 3.0).unwrap();
        if let Value::List(items) = result {
            assert_eq!(
                items,
                vec![Value::Number(0.0), Value::Number(1.0), Value::Number(2.0)]
            );
        } else {
            panic!("Expected list");
        }

        // Example from spec: -2..2 → [-2, -1, 0, 1]
        let result = expand_range(-2.0, 2.0).unwrap();
        if let Value::List(items) = result {
            assert_eq!(
                items,
                vec![
                    Value::Number(-2.0),
                    Value::Number(-1.0),
                    Value::Number(0.0),
                    Value::Number(1.0)
                ]
            );
        } else {
            panic!("Expected list");
        }
    }

    // Tests for descending ranges (0.1.2 feature)
    #[test]
    fn test_descending_range_5_to_0() {
        let result = expand_range(5.0, 0.0).unwrap();
        if let Value::List(items) = result {
            assert_eq!(
                items,
                vec![
                    Value::Number(5.0),
                    Value::Number(4.0),
                    Value::Number(3.0),
                    Value::Number(2.0),
                    Value::Number(1.0)
                ]
            );
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_descending_range_10_to_5() {
        let result = expand_range(10.0, 5.0).unwrap();
        if let Value::List(items) = result {
            assert_eq!(
                items,
                vec![
                    Value::Number(10.0),
                    Value::Number(9.0),
                    Value::Number(8.0),
                    Value::Number(7.0),
                    Value::Number(6.0)
                ]
            );
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_descending_range_negative() {
        let result = expand_range(-2.0, -5.0).unwrap();
        if let Value::List(items) = result {
            assert_eq!(
                items,
                vec![
                    Value::Number(-2.0),
                    Value::Number(-3.0),
                    Value::Number(-4.0)
                ]
            );
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_descending_range_single_element() {
        let result = expand_range(5.0, 4.0).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], Value::Number(5.0));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_descending_range_size_limit() {
        // This should exceed our limit of 1,000,000 elements
        let result = expand_range(2_000_000.0, 0.0);
        assert!(matches!(result, Err(RuntimeError::InvalidOperation { .. })));
    }
}
