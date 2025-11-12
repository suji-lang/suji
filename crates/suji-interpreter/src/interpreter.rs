//! AST interpreter implementation of the Executor trait.

use std::rc::Rc;
use suji_ast::ast::{Expr, Stmt};
use suji_runtime::{Executor, ModuleRegistry};
use suji_values::{Env, FunctionValue, RuntimeError, Value};

/// AST-walking interpreter implementation
pub struct AstInterpreter;

impl Executor for AstInterpreter {
    fn execute_stmt(
        &self,
        stmt: &Stmt,
        env: Rc<Env>,
        module_registry: &ModuleRegistry,
    ) -> Result<Option<Value>, RuntimeError> {
        use crate::eval::eval_stmt;
        let mut loop_stack = Vec::new();
        eval_stmt(stmt, env, &mut loop_stack, Some(module_registry))
    }

    fn execute_expr(
        &self,
        expr: &Expr,
        env: Rc<Env>,
        module_registry: &ModuleRegistry,
    ) -> Result<Value, RuntimeError> {
        use crate::eval::eval_expr;
        eval_expr(expr, env, Some(module_registry))
    }

    fn call_function(
        &self,
        func: &FunctionValue,
        args: Vec<Value>,
        caller_env: Option<Rc<Env>>,
        module_registry: &ModuleRegistry,
    ) -> Result<Value, RuntimeError> {
        use crate::eval::call_function;
        call_function(func, args, caller_env, Some(module_registry), None)
    }

    fn eval_source(
        &self,
        source: &str,
        env: Rc<Env>,
        module_registry: &ModuleRegistry,
        expect_export: bool,
    ) -> Result<Value, RuntimeError> {
        // Parse the source (standardized error handling)
        let statements =
            suji_parser::parse_program(source).map_err(|e| RuntimeError::InvalidOperation {
                message: format!("Parse error: {}", e),
            })?;

        // Evaluate statements using execute_stmt
        let mut last_value = None;
        let mut export_value = None;

        for stmt in &statements {
            match self.execute_stmt(stmt, env.clone(), module_registry) {
                Ok(Some(v)) => {
                    if matches!(stmt, Stmt::Export { .. }) {
                        export_value = Some(v.clone());
                    }
                    last_value = Some(v);
                }
                Ok(None) => {}
                Err(e) => return Err(e.with_span(stmt.span().clone())),
            }
        }

        if expect_export {
            export_value.ok_or_else(|| RuntimeError::InvalidOperation {
                message: "Module has no export".to_string(),
            })
        } else {
            Ok(last_value.unwrap_or(Value::Nil))
        }
    }
}
