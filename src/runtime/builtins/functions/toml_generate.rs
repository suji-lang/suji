//! TOML generate builtin function implementation
//!
//! This module contains the implementation of the toml:generate builtin function.

use super::super::toml::nn_to_toml_value;
use crate::runtime::value::{RuntimeError, Value};

/// TOML generate function - converts nn value to TOML string
pub fn builtin_toml_generate(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "toml:generate() takes exactly one argument".to_string(),
        });
    }

    let nn_value = &args[0];

    // Convert nn value to TOML value
    let toml_value = nn_to_toml_value(nn_value)?;

    // Generate TOML string
    // TOML requires a table structure, so we wrap non-table values in a table
    let toml_string = match toml_value {
        toml::Value::Table(_) => {
            toml::to_string(&toml_value).map_err(|e| RuntimeError::TomlGenerateError {
                message: format!("TOML generation failed: {}", e),
                value_type: nn_value.type_name().to_string(),
            })?
        }
        _ => {
            // Wrap non-table values in a table with a "value" key
            let mut table = toml::map::Map::new();
            table.insert("value".to_string(), toml_value);
            toml::to_string(&toml::Value::Table(table)).map_err(|e| {
                RuntimeError::TomlGenerateError {
                    message: format!("TOML generation failed: {}", e),
                    value_type: nn_value.type_name().to_string(),
                }
            })?
        }
    };

    Ok(Value::String(toml_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_generate_simple_values() {
        // Test generating simple TOML values (wrapped in table)
        assert_eq!(
            builtin_toml_generate(&[Value::Boolean(true)]).unwrap(),
            Value::String("value = true\n".to_string())
        );
        assert_eq!(
            builtin_toml_generate(&[Value::Boolean(false)]).unwrap(),
            Value::String("value = false\n".to_string())
        );
        assert_eq!(
            builtin_toml_generate(&[Value::Number(42.0)]).unwrap(),
            Value::String("value = 42\n".to_string())
        );
        assert_eq!(
            builtin_toml_generate(&[Value::Number(std::f64::consts::PI)]).unwrap(),
            Value::String("value = 3.141592653589793\n".to_string())
        );
        assert_eq!(
            builtin_toml_generate(&[Value::String("hello".to_string())]).unwrap(),
            Value::String("value = \"hello\"\n".to_string())
        );
    }

    #[test]
    fn test_toml_generate_arrays() {
        let result = builtin_toml_generate(&[Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])])
        .unwrap();
        assert_eq!(result, Value::String("value = [1, 2, 3]\n".to_string()));
    }

    #[test]
    fn test_toml_generate_maps() {
        let mut map = indexmap::IndexMap::new();
        map.insert(
            crate::runtime::value::MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map.insert(
            crate::runtime::value::MapKey::String("age".to_string()),
            Value::Number(30.0),
        );

        let result = builtin_toml_generate(&[Value::Map(map)]).unwrap();
        // The exact format may vary, but should contain the key-value pairs
        let toml_str = match result {
            Value::String(s) => s,
            _ => panic!("Expected string"),
        };
        assert!(toml_str.contains("name = \"Alice\""));
        assert!(toml_str.contains("age = 30"));
    }

    #[test]
    fn test_toml_generate_nil_error() {
        let result = builtin_toml_generate(&[Value::Nil]);
        assert!(matches!(
            result,
            Err(RuntimeError::TomlConversionError { .. })
        ));
    }

    #[test]
    fn test_toml_generate_non_string_map_key_error() {
        let mut map = indexmap::IndexMap::new();
        map.insert(
            crate::runtime::value::MapKey::Number(crate::runtime::value::OrderedFloat(42.0)),
            Value::String("value".to_string()),
        );

        let result = builtin_toml_generate(&[Value::Map(map)]);
        assert!(matches!(
            result,
            Err(RuntimeError::TomlConversionError { .. })
        ));
    }

    #[test]
    fn test_toml_generate_wrong_argument_count() {
        let result = builtin_toml_generate(&[]);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));

        let result = builtin_toml_generate(&[
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }
}
