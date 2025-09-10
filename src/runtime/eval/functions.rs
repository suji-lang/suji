use super::{EvalResult, eval_expr, eval_stmt};
use crate::ast::{Expr, Stmt, StringPart};
use crate::runtime::builtins::call_builtin;
use crate::runtime::env::Env;
use crate::runtime::shell::execute_shell_template;
use crate::runtime::string_template::evaluate_string_template;
use crate::runtime::value::{ControlFlow, FunctionValue, ParamSpec, RuntimeError, Value};
use std::rc::Rc;

/// Evaluate a function literal
pub fn eval_function_literal(
    params: &[crate::ast::Param],
    body: &Stmt,
    env: Rc<Env>,
) -> EvalResult<Value> {
    let param_specs: Vec<ParamSpec> = params.iter().map(|p| p.clone().into()).collect();
    let function = FunctionValue {
        params: param_specs,
        body: body.clone(),
        env: env.clone(), // Capture current environment
    };
    Ok(Value::Function(function))
}

/// Evaluate a function call
pub fn eval_function_call(callee: &Expr, args: &[Expr], env: Rc<Env>) -> EvalResult<Value> {
    // Note: Direct builtin checking removed to enforce module system
    // All builtins should now be accessed through imports (e.g., import std:println)

    let function_value = eval_expr(callee, env.clone())?;

    match function_value {
        Value::Function(func) => {
            // Check if this is a special builtin function
            if func.params.len() == 1
                && func.params[0].name.starts_with("__builtin_")
                && func.params[0].name.ends_with("__")
            {
                // Extract the builtin function name
                let builtin_name = func.params[0]
                    .name
                    .strip_prefix("__builtin_")
                    .and_then(|s| s.strip_suffix("__"))
                    .unwrap_or("");

                // Evaluate arguments for builtin call
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(eval_expr(arg, env.clone())?);
                }

                // Call the builtin function
                return call_builtin(builtin_name, &arg_values);
            }

            // Evaluate arguments
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(eval_expr(arg, env.clone())?);
            }

            // Handle default parameters
            let mut final_args = arg_values;
            if final_args.len() < func.params.len() {
                // Fill missing arguments with defaults
                for i in final_args.len()..func.params.len() {
                    match &func.params[i].default {
                        Some(default_expr) => {
                            // Evaluate default in function's closure environment
                            let default_value = eval_expr(default_expr, func.env.clone())?;
                            final_args.push(default_value);
                        }
                        None => {
                            return Err(RuntimeError::ArityMismatch {
                                message: format!(
                                    "Function expects {} arguments, got {}",
                                    func.params.len(),
                                    final_args.len()
                                ),
                            });
                        }
                    }
                }
            } else if final_args.len() > func.params.len() {
                return Err(RuntimeError::ArityMismatch {
                    message: format!(
                        "Function expects {} arguments, got {}",
                        func.params.len(),
                        final_args.len()
                    ),
                });
            }

            // Create new environment for function execution
            let call_env = Rc::new(Env::new_child(func.env.clone()));

            // Bind parameters
            for (param, arg_value) in func.params.iter().zip(final_args) {
                call_env.define_or_set(&param.name, arg_value);
            }

            // Execute function body
            let mut loop_stack = Vec::new();
            match eval_stmt(&func.body, call_env.clone(), &mut loop_stack) {
                Ok(result) => {
                    // Handle implicit returns
                    match result {
                        Some(value) => Ok(value), // Statement returned a value
                        None => {
                            // No explicit return, check if function body was a single expression
                            match &func.body {
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
                    }
                }
                Err(RuntimeError::ControlFlow {
                    flow: ControlFlow::Return(value),
                }) => Ok(*value),
                Err(other_error) => Err(other_error),
            }
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot call {}", function_value.type_name()),
        }),
    }
}

/// Evaluate a shell command template
pub fn eval_shell_command_template(parts: &[StringPart], env: Rc<Env>) -> EvalResult<Value> {
    let command = evaluate_string_template(parts, |expr| eval_expr(expr, env.clone()))?;
    execute_shell_template(&command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal, Param, Stmt};
    use crate::runtime::env::Env;
    use crate::token::Span;

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        crate::runtime::builtins::setup_global_env(&env);
        env
    }

    #[test]
    fn test_function_literal_creation() {
        let env = create_test_env();

        let params = vec![Param {
            name: "x".to_string(),
            default: None,
            span: Span::default(),
        }];

        let body = Stmt::Expr(Expr::Literal(Literal::Number(42.0, Span::default())));

        let result = eval_function_literal(&params, &body, env).unwrap();

        match result {
            Value::Function(func) => {
                assert_eq!(func.params.len(), 1);
                assert_eq!(func.params[0].name, "x");
            }
            _ => panic!("Expected function value"),
        }
    }

    #[test]
    fn test_builtin_function_call() {
        let env = create_test_env();

        // Set up imported println in the environment (simulating import std:println)
        let std_module = crate::runtime::builtins::create_std_module();
        if let Value::Map(map) = std_module {
            env.define_or_set(
                "println",
                map.get(&crate::runtime::value::MapKey::String(
                    "println".to_string(),
                ))
                .unwrap()
                .clone(),
            );
        }

        // Test calling the imported builtin
        let callee = Expr::Literal(Literal::Identifier("println".to_string(), Span::default()));
        let args = vec![Expr::Literal(Literal::StringTemplate(
            vec![crate::ast::StringPart::Text("hello".to_string())],
            Span::default(),
        ))];

        let result = eval_function_call(&callee, &args, env);
        // println returns null
        match &result {
            Ok(value) => assert_eq!(*value, Value::Nil),
            Err(error) => panic!("Test failed with error: {:?}", error),
        }
    }

    #[test]
    fn test_implicit_returns() {
        let env = create_test_env();

        // Test single expression function body (implicit return)
        let params = vec![Param {
            name: "x".to_string(),
            default: None,
            span: Span::default(),
        }];

        let body = Stmt::Expr(Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Identifier(
                "x".to_string(),
                Span::default(),
            ))),
            op: crate::ast::BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Number(1.0, Span::default()))),
            span: Span::default(),
        });

        let func = eval_function_literal(&params, &body, env.clone()).unwrap();
        let callee = Expr::Literal(Literal::Identifier("func".to_string(), Span::default()));
        env.define_or_set("func", func);

        let args = vec![Expr::Literal(Literal::Number(5.0, Span::default()))];
        let result = eval_function_call(&callee, &args, env.clone()).unwrap();
        assert_eq!(result, Value::Number(6.0));

        // Test block with last statement as expression (implicit return)
        let block_body = Stmt::Block {
            statements: vec![
                Stmt::Expr(Expr::Literal(Literal::Number(10.0, Span::default()))),
                Stmt::Expr(Expr::Binary {
                    left: Box::new(Expr::Literal(Literal::Identifier(
                        "x".to_string(),
                        Span::default(),
                    ))),
                    op: crate::ast::BinaryOp::Multiply,
                    right: Box::new(Expr::Literal(Literal::Number(2.0, Span::default()))),
                    span: Span::default(),
                }),
            ],
            span: Span::default(),
        };

        let func2 = eval_function_literal(&params, &block_body, env.clone()).unwrap();
        env.define_or_set("func2", func2);

        let callee2 = Expr::Literal(Literal::Identifier("func2".to_string(), Span::default()));
        let result2 = eval_function_call(&callee2, &args, env).unwrap();
        assert_eq!(result2, Value::Number(10.0)); // 5 * 2 = 10
    }
}
