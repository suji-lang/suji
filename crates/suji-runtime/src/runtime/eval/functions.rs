use super::{EvalResult, eval_expr};
use crate::runtime::builtins::call_builtin;
use crate::runtime::env::Env;
use crate::runtime::shell::execute_shell_template;
use crate::runtime::string_template::evaluate_string_template;
use crate::runtime::value::{FunctionValue, ParamSpec, RuntimeError, Value};
use std::rc::Rc;
use suji_ast::ast::{Expr, Stmt, StringPart};

/// Evaluate a function literal
pub fn eval_function_literal(
    params: &[suji_ast::ast::Param],
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

            // Use the unified function call implementation with full validation
            super::call_function(&func, arg_values, Some(env))
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
    use crate::runtime::builtins::setup_global_env;
    use crate::runtime::env::Env;
    use crate::runtime::value::DecimalNumber;
    use suji_ast::Span;
    use suji_ast::ast::{Expr, Literal, Param, Stmt};

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        setup_global_env(&env);
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

        let body = Stmt::Expr(Expr::Literal(Literal::Number(
            "42".to_string(),
            Span::default(),
        )));

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
            op: suji_ast::ast::BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Number(
                "1".to_string(),
                Span::default(),
            ))),
            span: Span::default(),
        });

        let func = eval_function_literal(&params, &body, env.clone()).unwrap();
        let callee = Expr::Literal(Literal::Identifier("func".to_string(), Span::default()));
        env.define_or_set("func", func);

        let args = vec![Expr::Literal(Literal::Number(
            "5".to_string(),
            Span::default(),
        ))];
        let result = eval_function_call(&callee, &args, env.clone()).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(6)));

        // Test block with last statement as expression (implicit return)
        let block_body = Stmt::Block {
            statements: vec![
                Stmt::Expr(Expr::Literal(Literal::Number(
                    "10".to_string(),
                    Span::default(),
                ))),
                Stmt::Expr(Expr::Binary {
                    left: Box::new(Expr::Literal(Literal::Identifier(
                        "x".to_string(),
                        Span::default(),
                    ))),
                    op: suji_ast::ast::BinaryOp::Multiply,
                    right: Box::new(Expr::Literal(Literal::Number(
                        "2".to_string(),
                        Span::default(),
                    ))),
                    span: Span::default(),
                }),
            ],
            span: Span::default(),
        };

        let func2 = eval_function_literal(&params, &block_body, env.clone()).unwrap();
        env.define_or_set("func2", func2);

        let callee2 = Expr::Literal(Literal::Identifier("func2".to_string(), Span::default()));
        let result2 = eval_function_call(&callee2, &args, env).unwrap();
        assert_eq!(result2, Value::Number(DecimalNumber::from_i64(10))); // 5 * 2 = 10
    }
}
