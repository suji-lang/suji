use super::super::env::Env;
use super::super::module::ModuleRegistry;
use super::super::value::{RuntimeError, Value};
use std::rc::Rc;
use suji_ast::ast::ImportSpec;

/// Force-loads a value if it's a lazy module.
/// Returns the value unchanged if it's not a module.
fn force_load_if_module(value: Value, registry: &ModuleRegistry) -> Result<Value, RuntimeError> {
    match value {
        Value::Module(handle) => registry.force_load_module(&handle),
        other => Ok(other),
    }
}

/// Evaluate an import statement
pub fn eval_import(
    spec: &ImportSpec,
    env: Rc<Env>,
    module_registry: &ModuleRegistry,
) -> Result<(), RuntimeError> {
    match spec {
        ImportSpec::Module { name } => {
            // import module - bind the whole module to the module name
            // Env → FS → builtins
            let module = module_registry.resolve_module_root(&env, name)?;
            let module = force_load_if_module(module, module_registry)?;
            env.define_or_set(name, module);
            Ok(())
        }

        ImportSpec::Item { module, name } => {
            // import module:item - bind the specific item to its name
            // Env → FS → builtins for module path
            let base = module_registry.resolve_module_path(&env, module, true)?;
            let base = force_load_if_module(base, module_registry)?;
            let mut item = match base {
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
            // Force-load the item if it's also a module
            item = force_load_if_module(item, module_registry)?;
            env.define_or_set(name, item);
            Ok(())
        }

        ImportSpec::ItemAs {
            module,
            name,
            alias,
        } => {
            // import module:item as alias - bind the specific item to the alias
            let base = module_registry.resolve_module_path(&env, module, true)?;
            let base = force_load_if_module(base, module_registry)?;
            let mut item = match base {
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
            // Force-load the item if it's also a module
            item = force_load_if_module(item, module_registry)?;
            env.define_or_set(alias, item);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_ast::ast::ImportSpec;

    fn create_test_env() -> Rc<Env> {
        Rc::new(Env::new())
    }

    fn create_test_registry() -> ModuleRegistry {
        ModuleRegistry::new()
    }

    #[test]
    #[ignore] // std is now provided by facade layer with virtual std resolver
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
    #[ignore] // std is now provided by facade layer with virtual std resolver
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
    #[ignore] // std is now provided by facade layer with virtual std resolver
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
