use crate::eval::{EvalResult, call_function, eval_expr};
use std::rc::Rc;
use suji_ast::ast::{Expr, Stmt, StringPart};
use suji_runtime::ModuleRegistry;
use suji_values::Env;
use suji_values::evaluate_string_template;
use suji_values::execute_shell_template;
use suji_values::{FunctionBody, FunctionValue, ParamSpec, RuntimeError, Value};

/// Evaluate a function literal
pub fn eval_function_literal(
    params: &[suji_ast::ast::Param],
    body: &Stmt,
    env: Rc<Env>,
) -> EvalResult<Value> {
    let param_specs: Vec<ParamSpec> = params.iter().map(|p| p.clone().into()).collect();
    let function = FunctionValue {
        params: param_specs,
        body: FunctionBody::Ast(body.clone()),
        env: env.clone(), // Capture current environment
    };
    Ok(Value::Function(function))
}

/// Evaluate a function call
pub fn eval_function_call(
    callee: &Expr,
    args: &[Expr],
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Value> {
    let function_value = eval_expr(callee, env.clone(), registry)?;

    match function_value {
        Value::Function(func) => {
            // Evaluate arguments
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(eval_expr(arg, env.clone(), registry)?);
            }

            // Delegate to call_function
            call_function(&func, arg_values, Some(env), registry, None)
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot call {}", function_value.type_name()),
        }),
    }
}

/// Evaluate a shell command template
pub fn eval_shell_command_template(
    parts: &[StringPart],
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Value> {
    let command = evaluate_string_template(parts, |expr| eval_expr(expr, env.clone(), registry))?;
    execute_shell_template(&command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_ast::Span;
    use suji_ast::ast::{Expr, Literal, Param, Stmt};
    use suji_runtime::setup_global_env;
    use suji_values::DecimalNumber;
    use suji_values::Env;

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
        let result = eval_function_call(&callee, &args, env.clone(), None).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(6)));
    }
}
