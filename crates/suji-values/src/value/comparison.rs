use super::types::{FunctionValue, Value};

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Regex(a), Value::Regex(b)) => a.as_str() == b.as_str(),
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::Stream(_), Value::Stream(_)) => false, // Streams are never equal
            (Value::StreamProxy(a), Value::StreamProxy(b)) => a == b, // Proxies are equal if same kind
            (Value::Module(a), Value::Module(b)) => a.module_path == b.module_path,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl PartialEq for FunctionValue {
    fn eq(&self, other: &Self) -> bool {
        // Functions are equal if they have the same parameters and body
        // Environment comparison is intentionally omitted
        self.params == other.params && self.body == other.body
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            // Lists, maps, tuples, functions, regex, streams, modules, and nil are not comparable
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{DecimalNumber, FunctionBody, MapKey};
    use super::*;
    use indexmap::IndexMap;
    use std::rc::Rc;
    use suji_ast::{Expr, Literal, Stmt};
    use suji_lexer::Span;

    #[test]
    fn test_value_equality() {
        // Numbers
        assert_eq!(
            Value::Number(DecimalNumber::from_i64(42)),
            Value::Number(DecimalNumber::from_i64(42))
        );
        assert_ne!(
            Value::Number(DecimalNumber::from_i64(42)),
            Value::Number(DecimalNumber::from_i64(43))
        );

        // Booleans
        assert_eq!(Value::Boolean(true), Value::Boolean(true));
        assert_ne!(Value::Boolean(true), Value::Boolean(false));

        // Strings
        assert_eq!(
            Value::String("hello".to_string()),
            Value::String("hello".to_string())
        );
        assert_ne!(
            Value::String("hello".to_string()),
            Value::String("world".to_string())
        );

        // Lists
        assert_eq!(
            Value::List(vec![
                Value::Number(DecimalNumber::from_i64(1)),
                Value::Number(DecimalNumber::from_i64(2))
            ]),
            Value::List(vec![
                Value::Number(DecimalNumber::from_i64(1)),
                Value::Number(DecimalNumber::from_i64(2))
            ])
        );
        assert_ne!(
            Value::List(vec![Value::Number(DecimalNumber::from_i64(1))]),
            Value::List(vec![Value::Number(DecimalNumber::from_i64(2))])
        );

        // Maps
        let mut map1 = IndexMap::new();
        map1.insert(
            MapKey::String("key".to_string()),
            Value::Number(DecimalNumber::from_i64(42)),
        );
        let mut map2 = IndexMap::new();
        map2.insert(
            MapKey::String("key".to_string()),
            Value::Number(DecimalNumber::from_i64(42)),
        );
        assert_eq!(Value::Map(map1), Value::Map(map2));

        // Tuples
        assert_eq!(
            Value::Tuple(vec![
                Value::Number(DecimalNumber::from_i64(1)),
                Value::String("test".to_string())
            ]),
            Value::Tuple(vec![
                Value::Number(DecimalNumber::from_i64(1)),
                Value::String("test".to_string())
            ])
        );

        // Nil
        assert_eq!(Value::Nil, Value::Nil);

        // Cross-type comparisons should be false
        assert_ne!(
            Value::Number(DecimalNumber::from_i64(42)),
            Value::String("42".to_string())
        );
        assert_ne!(
            Value::Boolean(true),
            Value::Number(DecimalNumber::from_i64(1))
        );
        assert_ne!(Value::Nil, Value::Number(DecimalNumber::from_i64(0)));
    }

    #[test]
    fn test_function_value_equality() {
        use super::super::types::ParamSpec;
        use crate::env::Env;

        let env1 = Rc::new(Env::new());
        let env2 = Rc::new(Env::new());

        let params = vec![ParamSpec {
            name: "x".to_string(),
            default: None,
        }];

        let body = Stmt::Expr(Expr::Return {
            values: vec![Expr::Literal(Literal::Number(
                "42".to_string(),
                Span::default(),
            ))],
            span: Span::default(),
        });

        let func1 = FunctionValue {
            params: params.clone(),
            body: FunctionBody::Ast(body.clone()),
            env: env1.clone(),
        };

        let func2 = FunctionValue {
            params: params.clone(),
            body: FunctionBody::Ast(body.clone()),
            env: env2, // Different environment
        };

        // Functions should be equal even with different environments
        assert_eq!(func1, func2);

        // Different parameters should make functions unequal
        let func3 = FunctionValue {
            params: vec![ParamSpec {
                name: "y".to_string(),
                default: None,
            }],
            body: FunctionBody::Ast(body.clone()),
            env: env1,
        };

        assert_ne!(func1, func3);
    }

    #[test]
    fn test_value_ordering() {
        use std::cmp::Ordering;

        // Numbers
        assert_eq!(
            Value::Number(DecimalNumber::from_i64(1))
                .partial_cmp(&Value::Number(DecimalNumber::from_i64(2))),
            Some(Ordering::Less)
        );
        assert_eq!(
            Value::Number(DecimalNumber::from_i64(2))
                .partial_cmp(&Value::Number(DecimalNumber::from_i64(1))),
            Some(Ordering::Greater)
        );
        assert_eq!(
            Value::Number(DecimalNumber::from_i64(1))
                .partial_cmp(&Value::Number(DecimalNumber::from_i64(1))),
            Some(Ordering::Equal)
        );

        // Strings
        assert_eq!(
            Value::String("apple".to_string()).partial_cmp(&Value::String("banana".to_string())),
            Some(Ordering::Less)
        );
        assert_eq!(
            Value::String("banana".to_string()).partial_cmp(&Value::String("apple".to_string())),
            Some(Ordering::Greater)
        );
        assert_eq!(
            Value::String("apple".to_string()).partial_cmp(&Value::String("apple".to_string())),
            Some(Ordering::Equal)
        );

        // Booleans
        assert_eq!(
            Value::Boolean(false).partial_cmp(&Value::Boolean(true)),
            Some(Ordering::Less)
        );
        assert_eq!(
            Value::Boolean(true).partial_cmp(&Value::Boolean(false)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            Value::Boolean(true).partial_cmp(&Value::Boolean(true)),
            Some(Ordering::Equal)
        );

        // Non-comparable types should return None
        assert_eq!(
            Value::Number(DecimalNumber::from_i64(1)).partial_cmp(&Value::String("1".to_string())),
            None
        );
        assert_eq!(Value::List(vec![]).partial_cmp(&Value::List(vec![])), None);
        assert_eq!(Value::Nil.partial_cmp(&Value::Nil), None);
    }
}
