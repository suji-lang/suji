use crate::eval::{eval_expr, eval_stmt, implicit_return::eval_implicit_return};
use std::rc::Rc;
use suji_runtime::{ModuleRegistry, call_builtin};
use suji_values::{ControlFlow, Env, FunctionBody, FunctionValue, RuntimeError, Value};

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

/// Unified function call implementation with optional module registry
pub fn call_function(
    func: &FunctionValue,
    args: Vec<Value>,
    caller_env: Option<Rc<Env>>,
    registry: Option<&ModuleRegistry>,
    env_overrides: Option<Vec<(String, Value)>>,
) -> Result<Value, RuntimeError> {
    let mut context = CallContext::new(func.clone(), args, caller_env);
    // Delegate to internal executor with optional module registry and env overrides
    execute_function(&mut context, registry, env_overrides)
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

                    let default_value = eval_expr(default_expr, eval_env, None)?;
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

/// Internal unified executor used by both public call paths
fn execute_function(
    context: &mut CallContext,
    module_registry: Option<&ModuleRegistry>,
    env_overrides: Option<Vec<(String, Value)>>,
) -> Result<Value, RuntimeError> {
    // Check for builtins
    if let FunctionBody::Builtin(builtin_name) = &context.func.body {
        return call_builtin(builtin_name, &context.args);
    }

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

    // Get AST body (will fail if bytecode has been passed)
    let body_stmt = context.func.as_ast_body()?;

    // Phase 3: Body execution
    let mut loop_stack = Vec::new();
    match eval_stmt(
        body_stmt,
        context.call_env.clone(),
        &mut loop_stack,
        module_registry,
    ) {
        Ok(result) => match result {
            Some(value) => Ok(value),
            None => eval_implicit_return(body_stmt, context.call_env.clone(), module_registry),
        },
        Err(e) => {
            // Check if it's a Return control flow (may be wrapped in WithSpan)
            match e.without_span() {
                RuntimeError::ControlFlow {
                    flow: ControlFlow::Return(value),
                } => Ok((**value).clone()),
                _ => Err(e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_ast::{Expr, Literal, Stmt};
    use suji_lexer::Span;
    use suji_runtime::setup_global_env;
    use suji_values::DecimalNumber;
    use suji_values::Env;
    use suji_values::ParamSpec;

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        setup_global_env(&env);
        env
    }

    fn create_test_function(params: Vec<ParamSpec>, body: Stmt, env: Rc<Env>) -> FunctionValue {
        FunctionValue {
            params,
            body: FunctionBody::Ast(body),
            env,
        }
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
            op: suji_ast::BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Number(
                "1".to_string(),
                Span::default(),
            ))),
            span: Span::default(),
        });

        let func = create_test_function(params, body, env.clone());
        let args = vec![Value::Number(DecimalNumber::from_i64(5))];

        let result = call_function(&func, args, Some(env), None, None).unwrap();
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
            op: suji_ast::BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Identifier(
                "y".to_string(),
                Span::default(),
            ))),
            span: Span::default(),
        });

        let func = create_test_function(params, body, env.clone());

        // Test with one argument (should use default for y)
        let args = vec![Value::Number(DecimalNumber::from_i64(5))];
        let result = call_function(&func, args, Some(env.clone()), None, None).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(15))); // 5 + 10

        // Test with two arguments (should override default)
        let args = vec![
            Value::Number(DecimalNumber::from_i64(5)),
            Value::Number(DecimalNumber::from_i64(3)),
        ];
        let result = call_function(&func, args, Some(env), None, None).unwrap();
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
        let result = call_function(&func, args, Some(env.clone()), None, None);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));

        // Test with too many arguments (should fail)
        let args = vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
        ];
        let result = call_function(&func, args, Some(env), None, None);
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
        let result = call_function(&func, args, Some(env.clone()), None, None).unwrap();
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
        let result = call_function(&func, args, Some(env), None, None).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(99)));
    }
}
