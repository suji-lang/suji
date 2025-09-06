use super::env::Env;
use super::value::{FunctionValue, MapKey, ParamSpec, RuntimeError, Value};
use crate::ast::Stmt;
use crate::token::Span;
use indexmap::IndexMap;
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
    Ok(Value::Null)
}

/// Get all built-in functions
pub fn get_builtins() -> Vec<Builtin> {
    vec![Builtin {
        name: "println",
        func: builtin_println,
        min_args: 0,
        max_args: None, // variadic
    }]
}

/// Create the std module as a Value::Map
pub fn create_std_module() -> Value {
    let mut std_map = IndexMap::new();

    // Add println to std module
    std_map.insert(
        MapKey::String("println".to_string()),
        create_builtin_function_value("println"),
    );

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
    matches!(name, "println")
}

/// Call a built-in function by name
pub fn call_builtin(name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
    match name {
        "println" => builtin_println(args),
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
        assert_eq!(builtin_println(&[]).unwrap(), Value::Null);

        // Test println with one argument
        let args = vec![Value::String("Hello".to_string())];
        assert_eq!(builtin_println(&args).unwrap(), Value::Null);

        // Test println with multiple arguments
        let args = vec![
            Value::String("Hello".to_string()),
            Value::Number(42.0),
            Value::Boolean(true),
        ];
        assert_eq!(builtin_println(&args).unwrap(), Value::Null);
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
        assert_eq!(call_builtin("println", &args).unwrap(), Value::Null);

        assert!(call_builtin("invalid", &args).is_err());
    }
}
