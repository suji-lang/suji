use super::value::{RuntimeError, Value};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

/// Lexically-scoped environment for variable bindings
#[derive(Debug)]
pub struct Env {
    /// Parent environment (None for root environment)
    parent: Option<Rc<Env>>,
    /// Variable bindings in this scope
    bindings: RefCell<IndexMap<String, Value>>,
}

impl Env {
    /// Create a new root environment
    pub fn new() -> Self {
        Env {
            parent: None,
            bindings: RefCell::new(IndexMap::new()),
        }
    }

    /// Create a new child environment with this environment as parent
    pub fn new_child(parent: Rc<Env>) -> Self {
        Env {
            parent: Some(parent),
            bindings: RefCell::new(IndexMap::new()),
        }
    }

    /// Define a new variable or update an existing one in the current scope
    pub fn define_or_set(&self, name: &str, value: Value) {
        self.bindings.borrow_mut().insert(name.to_string(), value);
    }

    /// Get a variable value, searching up the parent chain
    pub fn get(&self, name: &str) -> Result<Value, RuntimeError> {
        // First check current scope
        if let Some(value) = self.bindings.borrow().get(name) {
            return Ok(value.clone());
        }

        // Then check parent scopes
        if let Some(ref parent) = self.parent {
            return parent.get(name);
        }

        // Variable not found
        Err(RuntimeError::UndefinedVariable {
            name: name.to_string(),
        })
    }

    /// Set an existing variable, searching up the parent chain
    /// If the variable doesn't exist anywhere, define it in the current scope
    pub fn set_existing(&self, name: &str, value: Value) -> Result<(), RuntimeError> {
        // Check if variable exists in current scope
        if self.bindings.borrow().contains_key(name) {
            self.bindings.borrow_mut().insert(name.to_string(), value);
            return Ok(());
        }

        // Check parent scopes
        if let Some(ref parent) = self.parent {
            // If it exists in a parent scope, try to set it there
            if parent.contains(name) {
                return parent.set_existing(name, value);
            }
        }

        // Variable doesn't exist anywhere, define in current scope
        self.bindings.borrow_mut().insert(name.to_string(), value);
        Ok(())
    }

    /// Check if a variable exists in this environment or any parent
    pub fn contains(&self, name: &str) -> bool {
        if self.bindings.borrow().contains_key(name) {
            return true;
        }

        if let Some(ref parent) = self.parent {
            return parent.contains(name);
        }

        false
    }

    /// Get all variable names in the current scope (for debugging)
    pub fn local_names(&self) -> Vec<String> {
        self.bindings.borrow().keys().cloned().collect()
    }

    /// Get all variable names in this environment and all parents (for debugging)
    pub fn all_names(&self) -> Vec<String> {
        let mut names = self.local_names();

        if let Some(ref parent) = self.parent {
            let mut parent_names = parent.all_names();
            // Remove duplicates (current scope shadows parent)
            parent_names.retain(|name| !names.contains(name));
            names.extend(parent_names);
        }

        names
    }

    /// Get the number of bindings in this environment (for debugging)
    pub fn binding_count(&self) -> usize {
        self.bindings.borrow().len()
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_variable_operations() {
        let env = Env::new();

        // Define a variable
        env.define_or_set("x", Value::Number(42.0));

        // Get the variable
        assert_eq!(env.get("x").unwrap(), Value::Number(42.0));

        // Update the variable
        env.define_or_set("x", Value::Number(100.0));
        assert_eq!(env.get("x").unwrap(), Value::Number(100.0));

        // Try to get undefined variable
        assert!(matches!(
            env.get("y"),
            Err(RuntimeError::UndefinedVariable { .. })
        ));
    }

    #[test]
    fn test_nested_scopes() {
        let parent_env = Rc::new(Env::new());
        parent_env.define_or_set("x", Value::Number(1.0));
        parent_env.define_or_set("y", Value::Number(2.0));

        let child_env = Env::new_child(parent_env.clone());
        child_env.define_or_set("y", Value::Number(20.0)); // Shadow parent's y
        child_env.define_or_set("z", Value::Number(3.0));

        // Child can access parent's x
        assert_eq!(child_env.get("x").unwrap(), Value::Number(1.0));

        // Child's y shadows parent's y
        assert_eq!(child_env.get("y").unwrap(), Value::Number(20.0));

        // Child has its own z
        assert_eq!(child_env.get("z").unwrap(), Value::Number(3.0));

        // Parent still has original y
        assert_eq!(parent_env.get("y").unwrap(), Value::Number(2.0));

        // Parent doesn't have z
        assert!(matches!(
            parent_env.get("z"),
            Err(RuntimeError::UndefinedVariable { .. })
        ));
    }

    #[test]
    fn test_set_existing() {
        let parent_env = Rc::new(Env::new());
        parent_env.define_or_set("x", Value::Number(1.0));

        let child_env = Env::new_child(parent_env.clone());

        // Set existing variable in parent from child
        child_env.set_existing("x", Value::Number(100.0)).unwrap();
        assert_eq!(parent_env.get("x").unwrap(), Value::Number(100.0));

        // Set non-existing variable creates it in current scope
        child_env.set_existing("y", Value::Number(2.0)).unwrap();
        assert_eq!(child_env.get("y").unwrap(), Value::Number(2.0));
        assert!(matches!(
            parent_env.get("y"),
            Err(RuntimeError::UndefinedVariable { .. })
        ));
    }
}
