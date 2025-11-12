use super::types::{FunctionValue, MapKey, Value};
use std::fmt;

impl fmt::Debug for FunctionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Custom Debug implementation to avoid circular reference with env
        f.debug_struct("FunctionValue")
            .field("params", &self.params)
            .field("body", &self.body)
            .field(
                "env",
                &format_args!("Env{{ {} bindings }}", self.env.binding_count()),
            )
            .finish()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::List(items) => {
                let item_strings: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", item_strings.join(", "))
            }
            Value::Map(map) => {
                let entries: Vec<String> =
                    map.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", entries.join(", "))
            }
            Value::Tuple(items) => {
                let item_strings: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                if items.len() == 1 {
                    write!(f, "({},)", item_strings[0])
                } else {
                    write!(f, "({})", item_strings.join(", "))
                }
            }
            Value::Regex(regex) => write!(f, "/{}/", regex.as_str()),
            Value::Function(_) => write!(f, "<function>"),
            Value::Stream(stream) => write!(f, "<stream:{}>", stream.name),
            Value::StreamProxy(kind) => write!(f, "<stream-proxy:{:?}>", kind),
            Value::EnvMap(_) => write!(f, "<env>"),
            Value::Module(handle) => {
                if handle.loaded.borrow().is_some() {
                    write!(f, "<module '{}' (loaded)>", handle.module_path)
                } else {
                    write!(f, "<module '{}' (unloaded)>", handle.module_path)
                }
            }
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl fmt::Display for MapKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_value())
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{DecimalNumber, FunctionBody, MapKey, OrderedDecimal};
    use super::*;
    use indexmap::IndexMap;
    use regex::Regex;
    use std::rc::Rc;
    use suji_ast::Span;
    use suji_ast::ast::{Expr, Literal, Stmt};

    #[test]
    fn test_value_display() {
        // Numbers - integers should display without decimal
        assert_eq!(
            format!("{}", Value::Number(DecimalNumber::from_i64(42))),
            "42"
        );
        assert_eq!(
            format!("{}", Value::Number(DecimalNumber::parse("42.5").unwrap())),
            "42.5"
        );

        // Booleans
        assert_eq!(format!("{}", Value::Boolean(true)), "true");
        assert_eq!(format!("{}", Value::Boolean(false)), "false");

        // Strings
        assert_eq!(format!("{}", Value::String("hello".to_string())), "hello");

        // Lists
        assert_eq!(format!("{}", Value::List(vec![])), "[]");
        assert_eq!(
            format!(
                "{}",
                Value::List(vec![
                    Value::Number(DecimalNumber::from_i64(1)),
                    Value::Number(DecimalNumber::from_i64(2))
                ])
            ),
            "[1, 2]"
        );

        // Maps
        let mut map = IndexMap::new();
        map.insert(
            MapKey::String("key".to_string()),
            Value::Number(DecimalNumber::from_i64(42)),
        );
        assert_eq!(format!("{}", Value::Map(map)), "{key: 42}");

        // Tuples
        assert_eq!(format!("{}", Value::Tuple(vec![])), "()");
        assert_eq!(
            format!(
                "{}",
                Value::Tuple(vec![Value::Number(DecimalNumber::from_i64(1))])
            ),
            "(1,)"
        );
        assert_eq!(
            format!(
                "{}",
                Value::Tuple(vec![
                    Value::Number(DecimalNumber::from_i64(1)),
                    Value::Number(DecimalNumber::from_i64(2))
                ])
            ),
            "(1, 2)"
        );

        // Regex
        let regex = Regex::new("test").unwrap();
        assert_eq!(format!("{}", Value::Regex(regex)), "/test/");

        // Function
        assert_eq!(
            format!(
                "{}",
                Value::Function(FunctionValue {
                    params: vec![],
                    body: FunctionBody::Ast(Stmt::Expr(Expr::Return {
                        values: Vec::new(),
                        span: Span::default()
                    })),
                    env: Rc::new(crate::env::Env::new()),
                })
            ),
            "<function>"
        );

        // Nil
        assert_eq!(format!("{}", Value::Nil), "nil");
    }

    #[test]
    fn test_map_key_display() {
        assert_eq!(
            format!(
                "{}",
                MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(42)))
            ),
            "42"
        );
        assert_eq!(format!("{}", MapKey::Boolean(true)), "true");
        assert_eq!(format!("{}", MapKey::String("hello".to_string())), "hello");

        let tuple_key = MapKey::Tuple(vec![
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(1))),
            MapKey::String("test".to_string()),
        ]);
        assert_eq!(format!("{}", tuple_key), "(1, test)");
    }

    #[test]
    fn test_function_value_debug() {
        use super::super::types::ParamSpec;
        use crate::env::Env;

        let env = Rc::new(Env::new());
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

        let func = FunctionValue {
            params,
            body: FunctionBody::Ast(body),
            env,
        };

        let debug_str = format!("{:?}", func);
        assert!(debug_str.contains("FunctionValue"));
        assert!(debug_str.contains("params"));
        assert!(debug_str.contains("body"));
        assert!(debug_str.contains("Env{"));
        assert!(debug_str.contains("bindings"));
    }
}
