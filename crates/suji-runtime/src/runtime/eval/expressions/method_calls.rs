use super::{EvalResult, eval_expr};
use crate::runtime::env::Env;
use crate::runtime::methods::{ValueRef, call_method};
use crate::runtime::value::Value;
use std::rc::Rc;
use suji_ast::ast::{Expr, Literal};

#[cfg(test)]
use crate::runtime::builtins::setup_global_env;

/// Evaluate method call
pub fn eval_method_call(
    target: &Expr,
    method: &str,
    args: &[Expr],
    env: Rc<Env>,
) -> EvalResult<Value> {
    // Evaluate arguments first
    let mut arg_values = Vec::new();
    for arg in args {
        arg_values.push(eval_expr(arg, env.clone())?);
    }

    // For method calls, we need to determine if we have a mutable or immutable receiver
    match target {
        Expr::Literal(Literal::Identifier(name, _)) => {
            // Variable - can be mutable
            let mut target_value = env.get(name)?;
            let receiver = ValueRef::Mutable(&mut target_value);
            let result = call_method(receiver, method, arg_values)?;
            // Update the variable in case it was mutated
            env.set_existing(name, target_value)?;
            Ok(result)
        }
        _ => {
            // Expression result - immutable
            let target_value = eval_expr(target, env)?;
            let receiver = ValueRef::Immutable(&target_value);
            call_method(receiver, method, arg_values)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::env::Env;
    use crate::runtime::value::DecimalNumber;
    use suji_ast::Span;
    use suji_ast::ast::{Expr, Literal};

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

        let result = eval_method_call(&target, "length", &args, env).unwrap();
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

        let result = eval_method_call(&target, "length", &args, env).unwrap();
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(3)));
    }
}
