use super::super::env::Env;
use super::super::module::ModuleRegistry;
use super::super::value::RuntimeError;
use crate::ast::ImportSpec;
use std::rc::Rc;

#[cfg(test)]
use super::super::value::Value;

/// Evaluate an import statement
pub fn eval_import(
    spec: &ImportSpec,
    env: Rc<Env>,
    module_registry: &ModuleRegistry,
) -> Result<(), RuntimeError> {
    match spec {
        ImportSpec::Module { name } => {
            // import module - bind the whole module to the module name
            let module = module_registry.resolve_module(name)?;
            env.define_or_set(name, module);
            Ok(())
        }

        ImportSpec::Item { module, name } => {
            // import module:item - bind the specific item to its name
            let item = module_registry.resolve_module_item(module, name)?;
            env.define_or_set(name, item);
            Ok(())
        }

        ImportSpec::ItemAs {
            module,
            name,
            alias,
        } => {
            // import module:item as alias - bind the specific item to the alias
            let item = module_registry.resolve_module_item(module, name)?;
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
}
