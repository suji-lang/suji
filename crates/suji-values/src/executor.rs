//! Executor trait for abstracting execution strategies.
//!
//! This trait allows multiple execution backends (AST interpreter, bytecode VM)
//! to coexist and share the same value types, methods, and runtime infrastructure.

use super::env::Env;
use super::value::{FunctionValue, RuntimeError, Value};
use std::rc::Rc;
use suji_ast::{Expr, Stmt};

/// Trait for executing Suji code
///
/// This trait abstracts the execution strategy, allowing multiple backends
/// (AST interpreter, bytecode VM) to coexist and share the same value types,
/// methods, and runtime infrastructure.
///
pub trait Executor {
    /// Execute a statement and return its value (if any)
    fn execute_stmt(&self, stmt: &Stmt, env: Rc<Env>) -> Result<Option<Value>, RuntimeError>;

    /// Execute an expression and return its value
    fn execute_expr(&self, expr: &Expr, env: Rc<Env>) -> Result<Value, RuntimeError>;

    /// Call a function with the given arguments
    fn call_function(
        &self,
        func: &FunctionValue,
        args: Vec<Value>,
        caller_env: Option<Rc<Env>>,
    ) -> Result<Value, RuntimeError>;
}
