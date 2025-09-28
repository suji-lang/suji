use super::{eval_expr, eval_stmt, eval_stmt_with_modules};
use crate::ast::Stmt;
use crate::runtime::env::Env;
use crate::runtime::module::ModuleRegistry;
use crate::runtime::value::{ControlFlow, FunctionValue, RuntimeError, Value};
use std::rc::Rc;

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

    // Phase 1: Argument validation and default resolution
    resolve_arguments(&mut context)?;

    // Phase 2: Parameter binding
    bind_parameters(&context)?;

    // Phase 3: Body execution with proper return handling
    execute_function_body(&context)
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

    // Apply environment overrides (e.g., std/io for pipe) before binding params
    if let Some(overrides) = env_overrides {
        for (name, value) in overrides {
            context.call_env.define_or_set(&name, value);
        }
    }

    // Resolve arguments (defaults, arity)
    resolve_arguments(&mut context)?;

    // Bind parameters
    bind_parameters(&context)?;

    // Execute body with module registry and implicit returns
    let mut loop_stack = Vec::new();
    match eval_stmt_with_modules(
        &context.func.body,
        context.call_env.clone(),
        &mut loop_stack,
        module_registry,
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
                Some(value) => Ok(value), // Statement returned a value
                None => {
                    // No explicit return, check if function body was a single expression
                    handle_implicit_return(&context.func.body, context.call_env.clone())
                }
            }
        }
        Err(RuntimeError::ControlFlow {
            flow: ControlFlow::Return(value),
        }) => Ok(*value),
        Err(other_error) => Err(other_error),
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

/// Simplified function call for closures (backward compatibility with eval_closure)
/// This version doesn't perform arity checking or default parameter resolution
pub fn call_closure_simple(func: &FunctionValue, args: Vec<Value>) -> Result<Value, RuntimeError> {
    // Create new environment for function execution
    let call_env = Rc::new(Env::new_child(func.env.clone()));

    // Bind arguments to parameters (no arity checking for backward compatibility)
    for (param, arg_value) in func.params.iter().zip(args) {
        call_env.define_or_set(&param.name, arg_value);
    }

    // Execute function body with simple return handling
    let mut loop_stack = Vec::new();
    match eval_stmt(&func.body, call_env, &mut loop_stack) {
        Ok(result) => Ok(result.unwrap_or(Value::Nil)),
        Err(RuntimeError::ControlFlow {
            flow: ControlFlow::Return(value),
        }) => Ok(*value),
        Err(other_error) => Err(other_error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal, Stmt};
    use crate::runtime::env::Env;
    use crate::runtime::value::DecimalNumber;
    use crate::runtime::value::ParamSpec;
    use crate::token::Span;

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        crate::runtime::builtins::setup_global_env(&env);
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
            op: crate::ast::BinaryOp::Add,
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
            op: crate::ast::BinaryOp::Add,
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

    #[test]
    fn test_closure_simple_backward_compatibility() {
        let env = create_test_env();

        // Create function: fn(x) { x * 2 }
        let params = vec![ParamSpec {
            name: "x".to_string(),
            default: None,
        }];

        let body = Stmt::Expr(Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Identifier(
                "x".to_string(),
                Span::default(),
            ))),
            op: crate::ast::BinaryOp::Multiply,
            right: Box::new(Expr::Literal(Literal::Number(
                "2".to_string(),
                Span::default(),
            ))),
            span: Span::default(),
        });

        let func = create_test_function(params, body, env);
        let args = vec![Value::Number(DecimalNumber::from_i64(7))];

        let result = call_closure_simple(&func, args).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(14)));
    }
}
