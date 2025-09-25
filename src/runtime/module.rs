use super::value::{RuntimeError, Value};
use std::collections::HashMap;

/// Registry for managing modules in the NN language runtime
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    /// Built-in modules that are always available
    builtins: HashMap<String, Value>,
    /// File-based modules that have been loaded (for future use)
    file_modules: HashMap<String, Value>,
}

impl ModuleRegistry {
    /// Create a new module registry with built-in modules
    pub fn new() -> Self {
        let mut registry = Self {
            builtins: HashMap::new(),
            file_modules: HashMap::new(),
        };

        // Register built-in modules
        registry.register_builtin_modules();
        registry
    }

    /// Create a clone of this registry with a custom override for the `std` module
    pub fn with_custom_std(&self, std_value: Value) -> Self {
        let mut new_registry = self.clone();
        new_registry.builtins.insert("std".to_string(), std_value);
        new_registry
    }

    /// Register all built-in modules
    fn register_builtin_modules(&mut self) {
        // Register the std module
        self.builtins
            .insert("std".to_string(), super::builtins::create_std_module());
    }

    /// Resolve a module by name, checking built-ins first, then file modules
    pub fn resolve_module(&self, name: &str) -> Result<Value, RuntimeError> {
        // Check built-ins first
        if let Some(module) = self.builtins.get(name) {
            return Ok(module.clone());
        }

        // Check file modules (for future file-based imports)
        if let Some(module) = self.file_modules.get(name) {
            return Ok(module.clone());
        }

        // Module not found
        Err(RuntimeError::InvalidOperation {
            message: format!("Module '{}' not found", name),
        })
    }

    /// Resolve a specific item from a module
    pub fn resolve_module_item(
        &self,
        module_name: &str,
        item_name: &str,
    ) -> Result<Value, RuntimeError> {
        let module = self.resolve_module(module_name)?;

        match module {
            Value::Map(map) => {
                let key = super::value::MapKey::String(item_name.to_string());
                match map.get(&key) {
                    Some(value) => Ok(value.clone()),
                    None => Err(RuntimeError::InvalidOperation {
                        message: format!(
                            "Item '{}' not found in module '{}'",
                            item_name, module_name
                        ),
                    }),
                }
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Module '{}' is not a valid module (not a map)", module_name),
            }),
        }
    }

    /// Resolve a nested module item (e.g., "std:json:parse")
    pub fn resolve_nested_module_item(
        &self,
        module_path: &str,
        item_name: &str,
    ) -> Result<Value, RuntimeError> {
        let parts: Vec<&str> = module_path.split(':').collect();
        if parts.is_empty() {
            return Err(RuntimeError::InvalidOperation {
                message: "Empty module path".to_string(),
            });
        }

        // Start with the root module
        let mut current_module = self.resolve_module(parts[0])?;

        // Navigate through nested modules
        for part in &parts[1..] {
            match current_module {
                Value::Map(map) => {
                    let key = super::value::MapKey::String(part.to_string());
                    match map.get(&key) {
                        Some(Value::Map(nested_map)) => {
                            current_module = Value::Map(nested_map.clone());
                        }
                        Some(_) => {
                            return Err(RuntimeError::InvalidOperation {
                                message: format!(
                                    "'{}' is not a module in '{}'",
                                    part,
                                    parts[..parts.len() - 1].join(":")
                                ),
                            });
                        }
                        None => {
                            return Err(RuntimeError::InvalidOperation {
                                message: format!(
                                    "Module '{}' not found in '{}'",
                                    part,
                                    parts[..parts.len() - 1].join(":")
                                ),
                            });
                        }
                    }
                }
                _ => {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("'{}' is not a valid module (not a map)", parts[0]),
                    });
                }
            }
        }

        // Now resolve the final item
        match current_module {
            Value::Map(map) => {
                let key = super::value::MapKey::String(item_name.to_string());
                match map.get(&key) {
                    Some(value) => Ok(value.clone()),
                    None => Err(RuntimeError::InvalidOperation {
                        message: format!(
                            "Item '{}' not found in module '{}'",
                            item_name, module_path
                        ),
                    }),
                }
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Module '{}' is not a valid module (not a map)", module_path),
            }),
        }
    }

    /// Check if a module exists
    pub fn has_module(&self, name: &str) -> bool {
        self.builtins.contains_key(name) || self.file_modules.contains_key(name)
    }

    /// Register a file-based module (for future use)
    #[allow(dead_code)]
    pub fn register_file_module(&mut self, name: String, module: Value) {
        self.file_modules.insert(name, module);
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_registry_creation() {
        let registry = ModuleRegistry::new();
        assert!(registry.has_module("std"));
        assert!(!registry.has_module("nonexistent"));
    }

    #[test]
    fn test_std_module_resolution() {
        let registry = ModuleRegistry::new();

        // Test resolving the std module
        let std_module = registry.resolve_module("std").unwrap();
        assert!(matches!(std_module, Value::Map(_)));
    }

    #[test]
    fn test_std_println_resolution() {
        let registry = ModuleRegistry::new();

        // Test resolving println from std module
        let println_func = registry.resolve_module_item("std", "println").unwrap();
        assert!(matches!(println_func, Value::Function(_)));
    }

    #[test]
    fn test_module_not_found() {
        let registry = ModuleRegistry::new();

        let result = registry.resolve_module("nonexistent");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Module 'nonexistent' not found")
        );
    }

    #[test]
    fn test_item_not_found() {
        let registry = ModuleRegistry::new();

        let result = registry.resolve_module_item("std", "nonexistent");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Item 'nonexistent' not found")
        );
    }

    #[test]
    fn test_nested_module_resolution() {
        let registry = ModuleRegistry::new();

        // Test resolving json:parse from std:json
        let parse_func = registry
            .resolve_nested_module_item("std:json", "parse")
            .unwrap();
        assert!(matches!(parse_func, Value::Function(_)));

        // Test resolving json:generate from std:json
        let generate_func = registry
            .resolve_nested_module_item("std:json", "generate")
            .unwrap();
        assert!(matches!(generate_func, Value::Function(_)));
    }

    #[test]
    fn test_nested_module_resolution_errors() {
        let registry = ModuleRegistry::new();

        // Test nonexistent nested module
        let result = registry.resolve_nested_module_item("std:nonexistent", "parse");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Module 'nonexistent' not found")
        );

        // Test nonexistent item in nested module
        let result = registry.resolve_nested_module_item("std:json", "nonexistent");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Item 'nonexistent' not found")
        );

        // Test invalid module path
        let result = registry.resolve_nested_module_item("", "parse");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Module '' not found"));
    }
}
