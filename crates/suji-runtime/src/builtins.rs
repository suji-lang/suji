//! Dynamic builtin function registry

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;
use suji_values::{RuntimeError, Value};

/// Type for builtin function implementations
pub type BuiltinFn = fn(&[Value]) -> Result<Value, RuntimeError>;

/// Global registry of builtin functions
static BUILTIN_REGISTRY: Lazy<RwLock<HashMap<String, BuiltinFn>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Register a builtin function
pub fn register_builtin(name: impl Into<String>, func: BuiltinFn) {
    let mut registry = BUILTIN_REGISTRY
        .write()
        .expect("Failed to acquire write lock on builtin registry");
    registry.insert(name.into(), func);
}

/// Call a builtin function by name
pub fn call_builtin(name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
    let registry = BUILTIN_REGISTRY
        .read()
        .expect("Failed to acquire read lock on builtin registry");

    if let Some(func) = registry.get(name) {
        func(args)
    } else {
        Err(RuntimeError::MethodError {
            message: format!(
                "Builtin function '{}' not found. Ensure stdlib is initialized.",
                name
            ),
        })
    }
}

/// Stub functions for compatibility (used when stdlib hasn't been initialized)
pub fn setup_global_env(_env: &suji_values::Env) {
    // No-op shim. CLI/tests should import std from suji-stdlib and call its setup.
}

pub fn create_std_module() -> Value {
    // Provide an empty std map as a placeholder
    Value::Map(IndexMap::new())
}

/// List the names of all registered builtin functions
pub fn list_builtins() -> Vec<String> {
    let registry = BUILTIN_REGISTRY
        .read()
        .expect("Failed to acquire read lock on builtin registry");
    let mut names: Vec<String> = registry.keys().cloned().collect();
    names.sort();
    names
}
