use super::env::Env;
use super::value::{FunctionValue, MapKey, ParamSpec, RuntimeError, Value};
use crate::ast::Stmt;
use crate::token::Span;
use indexmap::IndexMap;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::rc::Rc;

/// Built-in function type
pub type BuiltinFunction = fn(&[Value]) -> Result<Value, RuntimeError>;

/// Built-in function wrapper for the runtime
#[derive(Debug, Clone)]
pub struct Builtin {
    pub name: &'static str,
    pub func: BuiltinFunction,
    pub min_args: usize,
    pub max_args: Option<usize>, // None for variadic
}

/// Create a special built-in function value that can be called normally
fn create_builtin_function_value(name: &str) -> Value {
    // Create a function value with a special marker that identifies it as built-in
    Value::Function(FunctionValue {
        params: vec![ParamSpec {
            name: format!("__builtin_{}__", name), // Special marker with function name
            default: None,
        }],
        body: Stmt::Block {
            statements: vec![],
            span: Span::default(),
        },
        env: Rc::new(Env::new()),
    })
}

/// println function - prints values to stdout with newline
pub fn builtin_println(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        println!();
    } else {
        let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
        println!("{}", output.join(" "));
    }
    Ok(Value::Nil)
}

/// JSON parse function - parses JSON string to nn value
pub fn builtin_json_parse(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "json:parse() takes exactly one argument".to_string(),
        });
    }

    let json_string = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "json:parse() argument must be a string".to_string(),
            });
        }
    };

    // Parse JSON string
    let json_value: JsonValue =
        serde_json::from_str(json_string).map_err(|e| RuntimeError::InvalidOperation {
            message: format!("Invalid JSON: {}", e),
        })?;

    // Convert JSON value to nn value
    json_to_nn_value(json_value)
}

/// JSON generate function - converts nn value to JSON string
pub fn builtin_json_generate(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "json:generate() takes exactly one argument".to_string(),
        });
    }

    let nn_value = &args[0];
    let json_value = nn_to_json_value(nn_value)?;

    let json_string =
        serde_json::to_string(&json_value).map_err(|e| RuntimeError::InvalidOperation {
            message: format!("Failed to serialize to JSON: {}", e),
        })?;

    Ok(Value::String(json_string))
}

/// Convert JSON value to nn value
fn json_to_nn_value(json_value: JsonValue) -> Result<Value, RuntimeError> {
    match json_value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Boolean(b)),
        JsonValue::Number(n) => {
            if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Err(RuntimeError::InvalidOperation {
                    message: "JSON number too large for nn number type".to_string(),
                })
            }
        }
        JsonValue::String(s) => Ok(Value::String(s)),
        JsonValue::Array(arr) => {
            let mut nn_array = Vec::new();
            for item in arr {
                nn_array.push(json_to_nn_value(item)?);
            }
            Ok(Value::List(nn_array))
        }
        JsonValue::Object(obj) => {
            let mut nn_map = IndexMap::new();
            for (key, value) in obj {
                let nn_key = MapKey::String(key);
                let nn_value = json_to_nn_value(value)?;
                nn_map.insert(nn_key, nn_value);
            }
            Ok(Value::Map(nn_map))
        }
    }
}

/// Convert nn value to JSON value
fn nn_to_json_value(nn_value: &Value) -> Result<JsonValue, RuntimeError> {
    match nn_value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
        Value::Number(n) => serde_json::Number::from_f64(*n)
            .ok_or_else(|| RuntimeError::InvalidOperation {
                message: "Number cannot be represented in JSON".to_string(),
            })
            .map(JsonValue::Number),
        Value::String(s) => Ok(JsonValue::String(s.clone())),
        Value::List(items) => {
            let mut json_array = Vec::new();
            for item in items {
                json_array.push(nn_to_json_value(item)?);
            }
            Ok(JsonValue::Array(json_array))
        }
        Value::Map(map) => {
            let mut json_obj = JsonMap::new();
            for (key, value) in map {
                let key_str = match key {
                    MapKey::String(s) => s.clone(),
                    MapKey::Number(n) => n.0.to_string(),
                    MapKey::Boolean(b) => b.to_string(),
                    MapKey::Tuple(_) => {
                        return Err(RuntimeError::InvalidOperation {
                            message: "Map keys with tuples cannot be converted to JSON".to_string(),
                        });
                    }
                };
                json_obj.insert(key_str, nn_to_json_value(value)?);
            }
            Ok(JsonValue::Object(json_obj))
        }
        Value::Tuple(items) => {
            let mut json_array = Vec::new();
            for item in items {
                json_array.push(nn_to_json_value(item)?);
            }
            Ok(JsonValue::Array(json_array))
        }
        Value::Regex(_) => Err(RuntimeError::InvalidOperation {
            message: "Regex values cannot be converted to JSON".to_string(),
        }),
        Value::Function(_) => Err(RuntimeError::InvalidOperation {
            message: "Function values cannot be converted to JSON".to_string(),
        }),
    }
}

/// Create the json module as a Value::Map
pub fn create_json_module() -> Value {
    let mut json_map = IndexMap::new();

    // Add json functions to json module
    json_map.insert(
        MapKey::String("parse".to_string()),
        create_builtin_function_value("json_parse"),
    );
    json_map.insert(
        MapKey::String("generate".to_string()),
        create_builtin_function_value("json_generate"),
    );

    Value::Map(json_map)
}

/// Get all built-in functions
pub fn get_builtins() -> Vec<Builtin> {
    vec![
        Builtin {
            name: "println",
            func: builtin_println,
            min_args: 0,
            max_args: None, // variadic
        },
        Builtin {
            name: "json_parse",
            func: builtin_json_parse,
            min_args: 1,
            max_args: Some(1),
        },
        Builtin {
            name: "json_generate",
            func: builtin_json_generate,
            min_args: 1,
            max_args: Some(1),
        },
    ]
}

/// Create the std module as a Value::Map
pub fn create_std_module() -> Value {
    let mut std_map = IndexMap::new();

    // Add println to std module
    std_map.insert(
        MapKey::String("println".to_string()),
        create_builtin_function_value("println"),
    );

    // Add json module to std module
    std_map.insert(MapKey::String("json".to_string()), create_json_module());

    Value::Map(std_map)
}

/// Setup the global environment with built-in functions and std module
pub fn setup_global_env(env: &Env) {
    // Add std module to global environment
    env.define_or_set("std", create_std_module());

    // Optionally add other global builtins here
}

/// Check if a function name refers to a built-in function
pub fn is_builtin_function(name: &str) -> bool {
    matches!(name, "println" | "json_parse" | "json_generate")
}

/// Call a built-in function by name
pub fn call_builtin(name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
    match name {
        "println" => builtin_println(args),
        "json_parse" => builtin_json_parse(args),
        "json_generate" => builtin_json_generate(args),
        _ => Err(RuntimeError::InvalidOperation {
            message: format!("Unknown built-in function: {}", name),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_println_builtin() {
        // Test println with no arguments
        assert_eq!(builtin_println(&[]).unwrap(), Value::Nil);

        // Test println with one argument
        let args = vec![Value::String("Hello".to_string())];
        assert_eq!(builtin_println(&args).unwrap(), Value::Nil);

        // Test println with multiple arguments
        let args = vec![
            Value::String("Hello".to_string()),
            Value::Number(42.0),
            Value::Boolean(true),
        ];
        assert_eq!(builtin_println(&args).unwrap(), Value::Nil);
    }

    #[test]
    fn test_std_module_creation() {
        let std_module = create_std_module();

        if let Value::Map(map) = std_module {
            assert!(map.contains_key(&MapKey::String("println".to_string())));
            let println_val = map.get(&MapKey::String("println".to_string())).unwrap();
            assert!(matches!(println_val, Value::Function(_)));
        } else {
            panic!("std module should be a map");
        }
    }

    #[test]
    fn test_builtin_detection() {
        assert!(is_builtin_function("println"));
        assert!(!is_builtin_function("not_a_builtin"));
    }

    #[test]
    fn test_call_builtin() {
        let args = vec![Value::String("test".to_string())];
        assert_eq!(call_builtin("println", &args).unwrap(), Value::Nil);

        assert!(call_builtin("invalid", &args).is_err());
    }

    // JSON functionality tests
    #[test]
    fn test_json_parse_simple_values() {
        // Test parsing simple JSON values
        assert_eq!(
            builtin_json_parse(&[Value::String("null".to_string())]).unwrap(),
            Value::Nil
        );
        assert_eq!(
            builtin_json_parse(&[Value::String("true".to_string())]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            builtin_json_parse(&[Value::String("false".to_string())]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            builtin_json_parse(&[Value::String("42".to_string())]).unwrap(),
            Value::Number(42.0)
        );
        assert_eq!(
            builtin_json_parse(&[Value::String("\"hello\"".to_string())]).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_json_parse_arrays() {
        let result = builtin_json_parse(&[Value::String("[1, 2, 3]".to_string())]).unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Number(1.0));
            assert_eq!(items[1], Value::Number(2.0));
            assert_eq!(items[2], Value::Number(3.0));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_json_parse_objects() {
        let result = builtin_json_parse(&[Value::String(
            "{\"name\": \"Alice\", \"age\": 30}".to_string(),
        )])
        .unwrap();
        if let Value::Map(map) = result {
            assert_eq!(map.len(), 2);
            assert_eq!(
                map.get(&MapKey::String("name".to_string())),
                Some(&Value::String("Alice".to_string()))
            );
            assert_eq!(
                map.get(&MapKey::String("age".to_string())),
                Some(&Value::Number(30.0))
            );
        } else {
            panic!("Expected map");
        }
    }

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
            builtin_json_generate(&[Value::Number(42.0)]).unwrap(),
            Value::String("42.0".to_string())
        );
        assert_eq!(
            builtin_json_generate(&[Value::String("hello".to_string())]).unwrap(),
            Value::String("\"hello\"".to_string())
        );
    }

    #[test]
    fn test_json_generate_arrays() {
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let result = builtin_json_generate(&[list]).unwrap();
        assert_eq!(result, Value::String("[1.0,2.0,3.0]".to_string()));
    }

    #[test]
    fn test_json_generate_objects() {
        let mut map_data = IndexMap::new();
        map_data.insert(
            MapKey::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map_data.insert(MapKey::String("age".to_string()), Value::Number(30.0));
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
    fn test_json_parse_invalid_json() {
        let result = builtin_json_parse(&[Value::String("invalid json".to_string())]);
        assert!(matches!(result, Err(RuntimeError::InvalidOperation { .. })));
    }

    #[test]
    fn test_json_parse_wrong_argument_type() {
        let result = builtin_json_parse(&[Value::Number(42.0)]);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }

    #[test]
    fn test_json_generate_unsupported_types() {
        // Test regex (should fail)
        let regex = Value::Regex(regex::Regex::new("test").unwrap());
        let result = builtin_json_generate(&[regex]);
        assert!(matches!(result, Err(RuntimeError::InvalidOperation { .. })));

        // Test function (should fail)
        let func = Value::Function(FunctionValue {
            params: vec![],
            body: Stmt::Block {
                statements: vec![],
                span: Span::default(),
            },
            env: Rc::new(Env::new()),
        });
        let result = builtin_json_generate(&[func]);
        assert!(matches!(result, Err(RuntimeError::InvalidOperation { .. })));
    }

    #[test]
    fn test_json_module_creation() {
        let json_module = create_json_module();
        if let Value::Map(map) = json_module {
            assert!(map.contains_key(&MapKey::String("parse".to_string())));
            assert!(map.contains_key(&MapKey::String("generate".to_string())));
        } else {
            panic!("JSON module should be a map");
        }
    }

    #[test]
    fn test_std_module_includes_json() {
        let std_module = create_std_module();
        if let Value::Map(map) = std_module {
            assert!(map.contains_key(&MapKey::String("println".to_string())));
            assert!(map.contains_key(&MapKey::String("json".to_string())));

            // Check that json is a module (map)
            if let Some(Value::Map(json_map)) = map.get(&MapKey::String("json".to_string())) {
                assert!(json_map.contains_key(&MapKey::String("parse".to_string())));
                assert!(json_map.contains_key(&MapKey::String("generate".to_string())));
            } else {
                panic!("JSON should be a module (map)");
            }
        } else {
            panic!("std module should be a map");
        }
    }

    #[test]
    fn test_json_builtin_detection() {
        assert!(is_builtin_function("json_parse"));
        assert!(is_builtin_function("json_generate"));
        assert!(!is_builtin_function("json_invalid"));
    }

    #[test]
    fn test_json_call_builtin() {
        let args = vec![Value::String("{\"test\": true}".to_string())];
        let result = call_builtin("json_parse", &args).unwrap();
        if let Value::Map(map) = result {
            assert_eq!(
                map.get(&MapKey::String("test".to_string())),
                Some(&Value::Boolean(true))
            );
        } else {
            panic!("Expected map");
        }

        let args = vec![Value::Boolean(true)];
        let result = call_builtin("json_generate", &args).unwrap();
        assert_eq!(result, Value::String("true".to_string()));
    }
}
