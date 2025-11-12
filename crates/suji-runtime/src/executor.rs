//! Executor trait for abstracting execution strategies.
//!
//! This trait allows multiple execution backends (AST interpreter, bytecode VM)
//! to coexist and share the same value types, methods, and runtime infrastructure.

use std::rc::Rc;
use suji_ast::ast::{Expr, Stmt};
use suji_values::{Env, FunctionValue, RuntimeError, Value};

use crate::ModuleRegistry;

/// Trait for executing Suji code
///
/// This trait abstracts the execution strategy, allowing multiple backends
/// (AST interpreter, bytecode VM) to coexist and share the same value types,
/// methods, and runtime infrastructure.
///
/// All methods now take a `module_registry` parameter to support module imports
/// and provide access to built-in modules during execution.
///
pub trait Executor {
    /// Execute a statement and return its value (if any)
    ///
    /// # Parameters
    /// - `stmt`: The statement to execute
    /// - `env`: The environment for variable lookups
    /// - `module_registry`: Registry for module resolution and imports
    fn execute_stmt(
        &self,
        stmt: &Stmt,
        env: Rc<Env>,
        module_registry: &ModuleRegistry,
    ) -> Result<Option<Value>, RuntimeError>;

    /// Execute an expression and return its value
    ///
    /// # Parameters
    /// - `expr`: The expression to evaluate
    /// - `env`: The environment for variable lookups
    /// - `module_registry`: Registry for module resolution and imports
    fn execute_expr(
        &self,
        expr: &Expr,
        env: Rc<Env>,
        module_registry: &ModuleRegistry,
    ) -> Result<Value, RuntimeError>;

    /// Call a function with the given arguments
    ///
    /// # Parameters
    /// - `func`: The function value to call
    /// - `args`: Arguments to pass to the function
    /// - `caller_env`: Optional caller environment for closures
    /// - `module_registry`: Registry for module resolution and imports
    fn call_function(
        &self,
        func: &FunctionValue,
        args: Vec<Value>,
        caller_env: Option<Rc<Env>>,
        module_registry: &ModuleRegistry,
    ) -> Result<Value, RuntimeError>;

    /// Evaluate source code and return the result
    ///
    /// This method provides a unified interface for evaluating both full programs
    /// and individual modules from source code strings.
    ///
    /// # Parameters
    /// - `source`: The source code to parse and evaluate
    /// - `env`: The environment for variable lookups
    /// - `module_registry`: Registry for module resolution and imports
    /// - `expect_export`: Whether to require and return an export value
    ///
    /// # Behavior
    /// - When `expect_export` is `true` (module evaluation):
    ///   - Parses and evaluates the source
    ///   - Returns the value from the export statement
    ///   - Returns an error if no export is found
    ///
    /// - When `expect_export` is `false` (program evaluation):
    ///   - Parses and evaluates the source
    ///   - Returns the value of the last statement
    ///   - Returns `Value::Nil` if no value is produced
    ///
    /// # Errors
    /// Returns `RuntimeError` if:
    /// - Parsing fails (wrapped as InvalidOperation)
    /// - Evaluation encounters a runtime error
    /// - `expect_export` is true but no export statement is found
    fn eval_source(
        &self,
        source: &str,
        env: Rc<Env>,
        module_registry: &ModuleRegistry,
        expect_export: bool,
    ) -> Result<Value, RuntimeError>;
}
