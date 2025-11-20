//! Built-in: json:generate(value) -> string.

use super::super::json::suji_to_json_value;
use suji_values::value::{RuntimeError, Value};

/// Convert SUJI value to JSON string.
pub fn builtin_json_generate(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "json:generate() takes exactly one argument".to_string(),
        });
    }

    let suji_value = &args[0];
    let json_value = suji_to_json_value(suji_value)?;

    let json_string =
        serde_json::to_string(&json_value).map_err(|e| RuntimeError::JsonGenerateError {
            message: format!("Failed to serialize to JSON: {}", e),
            value_type: suji_value.type_name().to_string(),
        })?;

    Ok(Value::String(json_string))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use std::rc::Rc;
    use suji_ast::Stmt;
    use suji_lexer::Span;
    use suji_values::Env;
    use suji_values::value::{FunctionBody, FunctionValue, MapKey};

    #[test]
    fn test_json_generate_simple_values() {
        assert_eq!(
            builtin_json_generate(&[Value::Nil]).unwrap(),
            Value::String("null".to_string())
        );
        assert_eq!(
            builtin_json_generate(&[Value::Boolean(true)]).unwrap(),
            Value::String("true".to_string())
        );
        assert_eq!(
            builtin_json_generate(&[Value::Boolean(false)]).unwrap(),
            Value::String("false".to_string())
        );
        assert_eq!(
            builtin_json_generate(&[Value::Number(suji_values::value::DecimalNumber::from_i64(
                42
            ))])
            .unwrap(),
            Value::String("42".to_string())
        );
        assert_eq!(
            builtin_json_generate(&[Value::String("hello".to_string())]).unwrap(),
            Value::String("\"hello\"".to_string())
        );
    }

    #[test]
    fn test_json_generate_arrays() {
        let list = Value::List(vec![
            Value::Number(suji_values::value::DecimalNumber::from_i64(1)),
            Value::Number(suji_values::value::DecimalNumber::from_i64(2)),
            Value::Number(suji_values::value::DecimalNumber::from_i64(3)),
        ]);
        let result = builtin_json_generate(&[list]).unwrap();
        assert_eq!(result, Value::String("[1,2,3]".to_string()));
    }

    #[test]
    fn test_json_generate_objects() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map_data.insert(
            MapKey::String("age".to_string()),
            Value::Number(suji_values::value::DecimalNumber::from_i64(30)),
        );
        let map = Value::Map(map_data);

        let result = builtin_json_generate(&[map]).unwrap();
        // Note: Order may vary, so we check if it contains the expected keys
        let json_str = if let Value::String(s) = result {
            s
        } else {
            panic!("Expected string")
        };
        assert!(json_str.contains("\"name\":\"Alice\""));
        assert!(json_str.contains("\"age\":30"));
    }

    #[test]
    fn test_json_generate_unsupported_types() {
        // Test regex (should fail)
        let regex = Value::Regex(regex::Regex::new("test").unwrap());
        let result = builtin_json_generate(&[regex]);
        assert!(matches!(
            result,
            Err(RuntimeError::JsonGenerateError { .. })
        ));

        // Test function (should fail)
        let func = Value::Function(FunctionValue {
            params: vec![],
            body: FunctionBody::Ast(Stmt::Block {
                statements: vec![],
                span: Span::default(),
            }),
            env: Rc::new(Env::new()),
        });
        let result = builtin_json_generate(&[func]);
        assert!(matches!(
            result,
            Err(RuntimeError::JsonGenerateError { .. })
        ));
    }
}
