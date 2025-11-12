//! Suji Runtime Crate - Execution coordination layer
//!
//! This crate provides the execution coordination layer for Suji, including:
//! - Executor trait for abstracting execution backends
//! - Module system (registry, caching, cycle detection)
//! - Builtin function registry

// Executor trait for abstracting execution backends
pub mod executor;
pub use executor::Executor;

// Module system
pub mod module_registry;
pub use module_registry::{
    CacheKey, LoadGuard, ModuleLoadAction, ModuleRegistry, SourceEvaluator, VirtualStdResolver,
    VirtualStdResult,
};

// Builtin function registry
pub mod builtins;
pub use builtins::{
    BuiltinFn, call_builtin, create_std_module, list_builtins, register_builtin, setup_global_env,
};
