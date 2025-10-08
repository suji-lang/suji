use super::{eval_expr, eval_stmt, eval_stmt_with_modules};
use crate::runtime::env::Env;
use crate::runtime::module::ModuleRegistry;
use crate::runtime::value::{ControlFlow, FunctionValue, RuntimeError, Value};
use std::rc::Rc;
use suji_ast::ast::Stmt;

#[cfg(test)]
use crate::runtime::builtins::setup_global_env;

/// Context for function call execution
pub struct CallContext {
    /// The function being called
    pub func: FunctionValue,
    /// Evaluated arguments
    pub args: Vec<Value>,
    /// Optional caller environment for default parameter evaluation
    pub caller_env: Option<Rc<Env>>,
    /// Function call environment (created during call)
    pub call_env: Rc<Env>,
}

impl CallContext {
    /// Create a new call context
    pub fn new(func: FunctionValue, args: Vec<Value>, caller_env: Option<Rc<Env>>) -> Self {
        // Create new environment for function execution
        let call_env = Rc::new(Env::new_child(func.env.clone()));

        Self {
            func,
            args,
            caller_env,
            call_env,
        }
    }
}

/// Unified function call implementation
pub fn call_function(
    func: &FunctionValue,
    args: Vec<Value>,
    caller_env: Option<Rc<Env>>,
) -> Result<Value, RuntimeError> {
    let mut context = CallContext::new(func.clone(), args, caller_env);
    // Delegate to internal executor with no module registry and no env overrides
    execute_function(&mut context, None, None)
}

/// Function call variant that allows supplying a custom module registry
pub fn call_function_with_modules(
    func: &FunctionValue,
    args: Vec<Value>,
    caller_env: Option<Rc<Env>>,
    module_registry: &ModuleRegistry,
    env_overrides: Option<Vec<(String, Value)>>,
) -> Result<Value, RuntimeError> {
    let mut context = CallContext::new(func.clone(), args, caller_env);
    // Delegate to internal executor with provided module registry and any env overrides
    execute_function(&mut context, Some(module_registry), env_overrides)
}

/// Phase 1: Resolve arguments with default parameter handling and arity checking
fn resolve_arguments(context: &mut CallContext) -> Result<(), RuntimeError> {
    let provided_args = context.args.len();
    let expected_params = context.func.params.len();

    // If we have fewer arguments than parameters, try to fill with defaults
    if provided_args < expected_params {
        for i in provided_args..expected_params {
            match &context.func.params[i].default {
                Some(default_expr) => {
                    // Evaluate default in the appropriate environment:
                    // - If caller_env is provided, use it (for eval_function_call compatibility)
                    // - Otherwise, use function's closure environment (for eval_closure compatibility)
                    let eval_env = context
                        .caller_env
                        .as_ref()
                        .unwrap_or(&context.func.env)
                        .clone();

                    let default_value = eval_expr(default_expr, eval_env)?;
                    context.args.push(default_value);
                }
                None => {
                    return Err(RuntimeError::ArityMismatch {
                        message: format!(
                            "Function expects {} arguments, got {}",
                            expected_params, provided_args
                        ),
                    });
                }
            }
        }
    } else if provided_args > expected_params {
        return Err(RuntimeError::ArityMismatch {
            message: format!(
                "Function expects {} arguments, got {}",
                expected_params, provided_args
            ),
        });
    }

    Ok(())
}

/// Phase 2: Bind parameters to arguments in the call environment
fn bind_parameters(context: &CallContext) -> Result<(), RuntimeError> {
    // Bind parameters to arguments
    for (param, arg_value) in context.func.params.iter().zip(&context.args) {
        context
            .call_env
            .define_or_set(&param.name, arg_value.clone());
    }

    Ok(())
}

/// Phase 3: Execute function body with proper control flow and implicit return handling
fn execute_function_body(context: &CallContext) -> Result<Value, RuntimeError> {
    let mut loop_stack = Vec::new();

    match eval_stmt(
        &context.func.body,
        context.call_env.clone(),
        &mut loop_stack,
    ) {
        Ok(result) => {
            // Handle implicit returns
            match result {
                Some(value) => Ok(value),
                None => handle_implicit_return(&context.func.body, context.call_env.clone()),
            }
        }
        Err(RuntimeError::ControlFlow {
            flow: ControlFlow::Return(value),
        }) => Ok(*value),
        Err(other_error) => Err(other_error),
    }
}

/// Internal unified executor used by both public call paths
fn execute_function(
    context: &mut CallContext,
    module_registry: Option<&ModuleRegistry>,
    env_overrides: Option<Vec<(String, Value)>>,
) -> Result<Value, RuntimeError> {
    // Apply environment overrides (e.g., std/io for pipe) before binding params
    if let Some(overrides) = env_overrides {
        for (name, value) in overrides {
            context.call_env.define_or_set(&name, value);
        }
    }

    // Phase 1: Argument validation and default resolution
    resolve_arguments(context)?;

    // Phase 2: Parameter binding
    bind_parameters(context)?;

    // Phase 3: Body execution
    if let Some(registry) = module_registry {
        // Execute body with module registry and implicit returns
        let mut loop_stack = Vec::new();
        match eval_stmt_with_modules(
            &context.func.body,
            context.call_env.clone(),
            &mut loop_stack,
            registry,
        ) {
            Ok(result) => match result {
                Some(value) => Ok(value),
                None => handle_implicit_return(&context.func.body, context.call_env.clone()),
            },
            Err(RuntimeError::ControlFlow {
                flow: ControlFlow::Return(value),
            }) => Ok(*value),
            Err(other) => Err(other),
        }
    } else {
        // Execute without module registry (standard path)
        execute_function_body(context)
    }
}

/// Handle implicit return logic for functions without explicit returns
fn handle_implicit_return(body: &Stmt, call_env: Rc<Env>) -> Result<Value, RuntimeError> {
    match body {
        Stmt::Expr(expr) => {
            // Single expression function body - return its value
            eval_expr(expr, call_env)
        }
        Stmt::Block { statements, .. } => {
            // Block function body - check if last statement was an expression
            if let Some(last_stmt) = statements.last() {
                match last_stmt {
                    Stmt::Expr(expr) => eval_expr(expr, call_env),
                    _ => Ok(Value::Nil), // Last statement was not an expression
                }
            } else {
                Ok(Value::Nil) // Empty block
            }
        }
        _ => Ok(Value::Nil), // Other statement types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::env::Env;
    use crate::runtime::value::DecimalNumber;
    use crate::runtime::value::ParamSpec;
    use suji_ast::Span;
    use suji_ast::ast::{Expr, Literal, Stmt};

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        setup_global_env(&env);
        env
    }

    fn create_test_function(params: Vec<ParamSpec>, body: Stmt, env: Rc<Env>) -> FunctionValue {
        FunctionValue { params, body, env }
    }

    #[test]
    fn test_simple_function_call() {
        let env = create_test_env();

        // Create function: fn(x) { x + 1 }
        let params = vec![ParamSpec {
            name: "x".to_string(),
            default: None,
        }];

        let body = Stmt::Expr(Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Identifier(
                "x".to_string(),
                Span::default(),
            ))),
            op: suji_ast::ast::BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Number(
                "1".to_string(),
                Span::default(),
            ))),
            span: Span::default(),
        });

        let func = create_test_function(params, body, env.clone());
        let args = vec![Value::Number(DecimalNumber::from_i64(5))];

        let result = call_function(&func, args, Some(env)).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(6)));
    }

    #[test]
    fn test_default_parameters() {
        let env = create_test_env();

        // Create function: fn(x, y = 10) { x + y }
        let params = vec![
            ParamSpec {
                name: "x".to_string(),
                default: None,
            },
            ParamSpec {
                name: "y".to_string(),
                default: Some(Expr::Literal(Literal::Number(
                    "10".to_string(),
                    Span::default(),
                ))),
            },
        ];

        let body = Stmt::Expr(Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Identifier(
                "x".to_string(),
                Span::default(),
            ))),
            op: suji_ast::ast::BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Identifier(
                "y".to_string(),
                Span::default(),
            ))),
            span: Span::default(),
        });

        let func = create_test_function(params, body, env.clone());

        // Test with one argument (should use default for y)
        let args = vec![Value::Number(DecimalNumber::from_i64(5))];
        let result = call_function(&func, args, Some(env.clone())).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(15))); // 5 + 10

        // Test with two arguments (should override default)
        let args = vec![
            Value::Number(DecimalNumber::from_i64(5)),
            Value::Number(DecimalNumber::from_i64(3)),
        ];
        let result = call_function(&func, args, Some(env)).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(8))); // 5 + 3
    }

    #[test]
    fn test_arity_mismatch() {
        let env = create_test_env();

        // Create function: fn(x) { x }
        let params = vec![ParamSpec {
            name: "x".to_string(),
            default: None,
        }];

        let body = Stmt::Expr(Expr::Literal(Literal::Identifier(
            "x".to_string(),
            Span::default(),
        )));
        let func = create_test_function(params, body, env.clone());

        // Test with no arguments (should fail)
        let args = vec![];
        let result = call_function(&func, args, Some(env.clone()));
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));

        // Test with too many arguments (should fail)
        let args = vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
        ];
        let result = call_function(&func, args, Some(env));
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_implicit_returns() {
        let env = create_test_env();

        // Test single expression body
        let params = vec![ParamSpec {
            name: "x".to_string(),
            default: None,
        }];

        let body = Stmt::Expr(Expr::Literal(Literal::Identifier(
            "x".to_string(),
            Span::default(),
        )));
        let func = create_test_function(params, body, env.clone());

        let args = vec![Value::Number(DecimalNumber::from_i64(42))];
        let result = call_function(&func, args, Some(env.clone())).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(42)));

        // Test block with expression as last statement
        let params = vec![ParamSpec {
            name: "x".to_string(),
            default: None,
        }];

        let body = Stmt::Block {
            statements: vec![
                Stmt::Expr(Expr::Literal(Literal::Number(
                    "1".to_string(),
                    Span::default(),
                ))), // This is ignored
                Stmt::Expr(Expr::Literal(Literal::Identifier(
                    "x".to_string(),
                    Span::default(),
                ))), // This is returned
            ],
            span: Span::default(),
        };

        let func = create_test_function(params, body, env.clone());
        let args = vec![Value::Number(DecimalNumber::from_i64(99))];
        let result = call_function(&func, args, Some(env)).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(99)));
    }

    // Removed legacy call_closure_simple test; behavior now covered by call_function tests
}
