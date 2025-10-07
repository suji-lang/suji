// Facade crate exposing the public suji_lang API used by integration tests

pub use suji_ast as ast;
pub use suji_lexer as lexer;
pub use suji_lexer::token;
pub use suji_parser as parser;
pub use suji_stdlib as stdlib;

// Recreate `suji_lang::runtime` namespace, but map `builtins` to stdlib's implementation
pub mod runtime {
    pub use suji_runtime::{env, eval, methods, range, regex, shell, string_template, value};

    pub mod builtins {
        pub use suji_stdlib::runtime::builtins::*;
    }

    // Override ModuleRegistry to ensure std module comes from suji-stdlib
    pub mod module {
        pub use suji_runtime::module::ModuleRegistry as RawModuleRegistry;

        use std::ops::{Deref, DerefMut};
        use suji_runtime::value::{RuntimeError, Value};

        #[derive(Debug, Clone)]
        pub struct ModuleRegistry(pub(crate) RawModuleRegistry);

        impl ModuleRegistry {
            pub fn new() -> Self {
                let base = RawModuleRegistry::new();
                let std_value = suji_stdlib::runtime::builtins::create_std_module();
                Self(base.with_custom_std(std_value))
            }

            pub fn with_custom_std(&self, std_value: Value) -> Self {
                Self(self.0.with_custom_std(std_value))
            }

            pub fn resolve_module(&self, name: &str) -> Result<Value, RuntimeError> {
                self.0.resolve_module(name)
            }

            pub fn resolve_module_item(
                &self,
                module_name: &str,
                item_name: &str,
            ) -> Result<Value, RuntimeError> {
                self.0.resolve_module_item(module_name, item_name)
            }

            pub fn resolve_nested_module_item(
                &self,
                module_path: &str,
                item_name: &str,
            ) -> Result<Value, RuntimeError> {
                self.0.resolve_nested_module_item(module_path, item_name)
            }

            pub fn has_module(&self, name: &str) -> bool {
                self.0.has_module(name)
            }
        }

        impl Deref for ModuleRegistry {
            type Target = RawModuleRegistry;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for ModuleRegistry {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl Default for ModuleRegistry {
            fn default() -> Self {
                Self::new()
            }
        }
    }
}
