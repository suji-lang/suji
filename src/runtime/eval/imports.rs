use super::super::env::Env;
use super::super::module::ModuleRegistry;
use super::super::value::{RuntimeError, Value};
use crate::ast::ImportSpec;
use std::rc::Rc;

// no test-only imports

/// Evaluate an import statement
pub fn eval_import(
    spec: &ImportSpec,
    env: Rc<Env>,
    module_registry: &ModuleRegistry,
) -> Result<(), RuntimeError> {
    // Helper: resolve a module path with env-first semantics, then registry fallback
    fn resolve_module_path(
        env: &Rc<Env>,
        registry: &ModuleRegistry,
        module_path: &str,
    ) -> Result<Value, RuntimeError> {
        let parts: Vec<&str> = module_path.split(':').collect();
        if parts.is_empty() {
            return Err(RuntimeError::InvalidOperation {
                message: "Empty module path".to_string(),
            });
        }

        // Try env first for the root module
        let mut current = match env.get(parts[0]) {
            Ok(v) => v,
            Err(_) => registry.resolve_module(parts[0])?,
        };

        // Traverse nested parts
        for part in &parts[1..] {
            match current {
                Value::Map(ref map) => {
                    let key = super::super::value::MapKey::String((*part).to_string());
                    current =
                        map.get(&key)
                            .cloned()
                            .ok_or_else(|| RuntimeError::InvalidOperation {
                                message: format!(
                                    "Module '{}' not found in '{}'",
                                    part,
                                    parts[..parts.len() - 1].join(":")
                                ),
                            })?;
                }
                _ => {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("'{}' is not a module (not a map)", parts[0]),
                    });
                }
            }
        }

        Ok(current)
    }

    match spec {
        ImportSpec::Module { name } => {
            // import module - bind the whole module to the module name
            // Env-first resolution
            let module = match env.get(name) {
                Ok(v) => v,
                Err(_) => module_registry.resolve_module(name)?,
            };
            env.define_or_set(name, module);
            Ok(())
        }

        ImportSpec::Item { module, name } => {
            // import module:item - bind the specific item to its name
            // Env-first resolution for module path
            let base = resolve_module_path(&env, module_registry, module)?;
            let item = match base {
                Value::Map(map) => {
                    let key = super::super::value::MapKey::String(name.to_string());
                    map.get(&key)
                        .cloned()
                        .ok_or_else(|| RuntimeError::InvalidOperation {
                            message: format!("Item '{}' not found in module '{}'", name, module),
                        })?
                }
                _ => {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Module '{}' is not a valid module (not a map)", module),
                    });
                }
            };
            env.define_or_set(name, item);
            Ok(())
        }

        ImportSpec::ItemAs {
            module,
            name,
            alias,
        } => {
            // import module:item as alias - bind the specific item to the alias
            let base = resolve_module_path(&env, module_registry, module)?;
            let item = match base {
                Value::Map(map) => {
                    let key = super::super::value::MapKey::String(name.to_string());
                    map.get(&key)
                        .cloned()
                        .ok_or_else(|| RuntimeError::InvalidOperation {
                            message: format!("Item '{}' not found in module '{}'", name, module),
                        })?
                }
                _ => {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Module '{}' is not a valid module (not a map)", module),
                    });
                }
            };
            env.define_or_set(alias, item);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::ImportSpec;

    fn create_test_env() -> Rc<Env> {
        Rc::new(Env::new())
    }

    fn create_test_registry() -> ModuleRegistry {
        ModuleRegistry::new()
    }

    #[test]
    fn test_import_whole_module() {
        let env = create_test_env();
        let registry = create_test_registry();

        let import_spec = ImportSpec::Module {
            name: "std".to_string(),
        };

        let result = eval_import(&import_spec, env.clone(), &registry);
        assert!(result.is_ok());

        // Check that std module is bound to environment
        let std_module = env.get("std").unwrap();
        assert!(matches!(std_module, Value::Map(_)));
    }

    #[test]
    fn test_import_specific_item() {
        let env = create_test_env();
        let registry = create_test_registry();

        let import_spec = ImportSpec::Item {
            module: "std".to_string(),
            name: "println".to_string(),
        };

        let result = eval_import(&import_spec, env.clone(), &registry);
        assert!(result.is_ok());

        // Check that println function is bound to environment
        let println_func = env.get("println").unwrap();
        assert!(matches!(println_func, Value::Function(_)));
    }

    #[test]
    fn test_import_item_with_alias() {
        let env = create_test_env();
        let registry = create_test_registry();

        let import_spec = ImportSpec::ItemAs {
            module: "std".to_string(),
            name: "println".to_string(),
            alias: "say".to_string(),
        };

        let result = eval_import(&import_spec, env.clone(), &registry);
        assert!(result.is_ok());

        // Check that println function is bound to "say" in environment
        let say_func = env.get("say").unwrap();
        assert!(matches!(say_func, Value::Function(_)));

        // Check that "println" is not bound (only the alias should be)
        assert!(env.get("println").is_err());
    }

    #[test]
    fn test_import_nonexistent_module() {
        let env = create_test_env();
        let registry = create_test_registry();

        let import_spec = ImportSpec::Module {
            name: "nonexistent".to_string(),
        };

        let result = eval_import(&import_spec, env.clone(), &registry);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Module 'nonexistent' not found")
        );
    }

    #[test]
    fn test_import_nonexistent_item() {
        let env = create_test_env();
        let registry = create_test_registry();

        let import_spec = ImportSpec::Item {
            module: "std".to_string(),
            name: "nonexistent".to_string(),
        };

        let result = eval_import(&import_spec, env.clone(), &registry);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Item 'nonexistent' not found")
        );
    }

    #[test]
    fn test_import_nested_module_item() {
        let env = create_test_env();
        let registry = create_test_registry();

        let import_spec = ImportSpec::Item {
            module: "std:json".to_string(),
            name: "parse".to_string(),
        };

        let result = eval_import(&import_spec, env.clone(), &registry);
        assert!(result.is_ok());

        // Check that parse function is bound to environment
        let parse_func = env.get("parse").unwrap();
        assert!(matches!(parse_func, Value::Function(_)));
    }

    #[test]
    fn test_import_nested_module_item_with_alias() {
        let env = create_test_env();
        let registry = create_test_registry();

        let import_spec = ImportSpec::ItemAs {
            module: "std:json".to_string(),
            name: "generate".to_string(),
            alias: "to_json".to_string(),
        };

        let result = eval_import(&import_spec, env.clone(), &registry);
        assert!(result.is_ok());

        // Check that generate function is bound to "to_json" in environment
        let to_json_func = env.get("to_json").unwrap();
        assert!(matches!(to_json_func, Value::Function(_)));

        // Check that "generate" is not bound (only the alias should be)
        assert!(env.get("generate").is_err());
    }

    #[test]
    fn test_import_nonexistent_nested_module() {
        let env = create_test_env();
        let registry = create_test_registry();

        let import_spec = ImportSpec::Item {
            module: "std:nonexistent".to_string(),
            name: "parse".to_string(),
        };

        let result = eval_import(&import_spec, env.clone(), &registry);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Module 'nonexistent' not found")
        );
    }
}
