use crate::runtime::value::{RuntimeError, Value};
use indexmap::IndexMap;

pub fn setup_global_env(_env: &crate::runtime::env::Env) {
    // No-op shim. CLI/tests should import std from suji-stdlib.
}

pub fn create_std_module() -> Value {
    // Provide an empty std map as a placeholder
    Value::Map(IndexMap::new())
}

pub fn call_builtin(_name: &str, _args: &[Value]) -> Result<Value, RuntimeError> {
    Err(RuntimeError::MethodError {
        message: "Builtins unavailable; ensure stdlib is imported in CLI/tests".to_string(),
    })
}
