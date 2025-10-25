use super::super::value::{RuntimeError, Value};
use super::common::ValueRef;

/// Function methods: to_string()
pub fn call_function_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::Function(_) => match method {
            "to_string" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "to_string() takes no arguments".to_string(),
                    });
                }
                Ok(Value::String("<function>".to_string()))
            }
            _ => Err(RuntimeError::MethodError {
                message: format!("Function has no method '{}'", method),
            }),
        },
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::value::FunctionValue;
    use super::super::common::ValueRef;
    use super::*;
    use crate::runtime::value::DecimalNumber;
    use std::rc::Rc;
    use suji_ast::Span;
    use suji_ast::ast::Expr;
    use suji_ast::ast::Stmt;

    #[test]
    fn test_function_to_string() {
        let func_val = Value::Function(FunctionValue {
            params: vec![],
            body: Stmt::Expr(Expr::Return {
                values: Vec::new(),
                span: Span::default(),
            }),
            env: Rc::new(crate::runtime::env::Env::new()),
        });
        let receiver = ValueRef::Immutable(&func_val);
        let result = call_function_method(receiver, "to_string", vec![]).unwrap();
        assert_eq!(result, Value::String("<function>".to_string()));
    }

    #[test]
    fn test_function_to_string_arity_mismatch() {
        let func_val = Value::Function(FunctionValue {
            params: vec![],
            body: Stmt::Expr(Expr::Return {
                values: Vec::new(),
                span: Span::default(),
            }),
            env: Rc::new(crate::runtime::env::Env::new()),
        });
        let receiver = ValueRef::Immutable(&func_val);
        let result = call_function_method(
            receiver,
            "to_string",
            vec![Value::Number(DecimalNumber::from_i64(1))],
        );
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_function_invalid_method() {
        let func_val = Value::Function(FunctionValue {
            params: vec![],
            body: Stmt::Expr(Expr::Return {
                values: Vec::new(),
                span: Span::default(),
            }),
            env: Rc::new(crate::runtime::env::Env::new()),
        });
        let receiver = ValueRef::Immutable(&func_val);
        let result = call_function_method(receiver, "invalid_method", vec![]);
        assert!(matches!(result, Err(RuntimeError::MethodError { .. })));
    }
}
