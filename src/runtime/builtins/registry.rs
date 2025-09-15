//! Builtin function registry
//! 
//! This module provides a centralized registry for builtin functions.

use std::collections::HashMap;
use crate::runtime::value::{RuntimeError, Value};
use super::functions::{Builtin, BuiltinFunction};

/// Registry for builtin functions
#[derive(Debug, Clone)]
pub struct BuiltinRegistry {
    functions: HashMap<String, Builtin>,
}

impl BuiltinRegistry {
    /// Create a new builtin registry with all builtin functions
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };
        
        // Register all builtin functions
        for builtin in super::functions::get_builtins() {
            registry.functions.insert(builtin.name.to_string(), builtin);
        }
        
        registry
    }
    
    /// Check if a function name refers to a built-in function
    pub fn is_builtin_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
    
    /// Call a built-in function by name
    pub fn call_builtin(&self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        if let Some(builtin) = self.functions.get(name) {
            (builtin.func)(args)
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("Unknown built-in function: {}", name),
            })
        }
    }
    
    /// Get all builtin functions
    pub fn get_builtins(&self) -> Vec<&Builtin> {
        self.functions.values().collect()
    }
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}
