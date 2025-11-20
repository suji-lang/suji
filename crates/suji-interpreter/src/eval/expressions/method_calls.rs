use crate::eval::utils::evaluate_exprs;
use crate::eval::{EvalResult, call_function, eval_expr};
use std::rc::Rc;
use suji_ast::{Expr, Literal};
use suji_runtime::ModuleRegistry;
use suji_values::methods::{ValueRef, call_method};
use suji_values::{Env, FunctionValue, Value};

/// Evaluate method call with optional module registry
pub fn eval_method_call(
    target: &Expr,
    method: &str,
    args: &[Expr],
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Value> {
    // Evaluate arguments first
    let arg_values = evaluate_exprs(args, env.clone(), registry)?;

    // For method calls, we need to determine if we have a mutable or immutable receiver
    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            // Variable - can be mutable
            let mut target_value = env.get(name)?;
            let receiver = ValueRef::Mutable(&mut target_value);

            // Create callback that captures registry for closure evaluation (if registry provided)
            let result = if let Some(reg) = registry {
                let call_closure_fn =
                    &|func: &FunctionValue, args: Vec<Value>, caller_env: Option<Rc<Env>>| {
                        call_function(func, args, caller_env, Some(reg), None)
                    };
                call_method(Some(call_closure_fn), receiver, method, arg_values)?
            } else {
                call_method(None, receiver, method, arg_values)?
            };

            // Update the variable in case it was mutated
            env.set_existing(name, target_value)?;
            Ok(result)
        }
        _ => {
            // Expression result - immutable
            let target_value = eval_expr(target, env, registry)?;
            let receiver = ValueRef::Immutable(&target_value);

            if let Some(reg) = registry {
                let call_closure_fn =
                    &|func: &FunctionValue, args: Vec<Value>, caller_env: Option<Rc<Env>>| {
                        call_function(func, args, caller_env, Some(reg), None)
                    };
                call_method(Some(call_closure_fn), receiver, method, arg_values)
            } else {
                call_method(None, receiver, method, arg_values)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_ast::{Expr, Literal};
    use suji_lexer::Span;
    use suji_runtime::setup_global_env;
    use suji_values::DecimalNumber;
    use suji_values::Env;

    fn create_test_env() -> Rc<Env> {
        let env = Rc::new(Env::new());
        setup_global_env(&env);
        env
    }

    #[test]
    fn test_string_method_call() {
        let env = create_test_env();
        env.define_or_set("s", Value::String("hello world".to_string()));

        let target = Expr::Literal(Literal::Identifier("s".to_string(), Span::default()));
        let args = vec![];

        let result = eval_method_call(&target, "length", &args, env, None).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(11)));
    }

    #[test]
    fn test_list_method_call() {
        let env = create_test_env();
        let list = Value::List(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3)),
        ]);
        env.define_or_set("my_list", list);

        let target = Expr::Literal(Literal::Identifier("my_list".to_string(), Span::default()));
        let args = vec![];

        let result = eval_method_call(&target, "length", &args, env, None).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(3)));
    }
}
