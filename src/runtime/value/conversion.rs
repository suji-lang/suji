use super::errors::RuntimeError;
use super::types::{MapKey, OrderedFloat, ParamSpec, Value};
use crate::ast::Param;

impl Value {
    /// Get the type name of this value for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::Boolean(_) => "boolean",
            Value::String(_) => "string",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Tuple(_) => "tuple",
            Value::Regex(_) => "regex",
            Value::Function(_) => "function",
            Value::Nil => "nil",
        }
    }

    /// Check if this value is truthy (only true for Boolean(true))
    pub fn is_truthy(&self) -> bool {
        matches!(self, Value::Boolean(true))
    }

    /// Try to convert this value to a MapKey
    pub fn try_into_map_key(self) -> Result<MapKey, RuntimeError> {
        match self {
            Value::Number(n) => Ok(MapKey::Number(OrderedFloat(n))),
            Value::Boolean(b) => Ok(MapKey::Boolean(b)),
            Value::String(s) => Ok(MapKey::String(s)),
            Value::Tuple(items) => {
                let key_items: Result<Vec<MapKey>, RuntimeError> = items
                    .into_iter()
                    .map(|item| item.try_into_map_key())
                    .collect();
                Ok(MapKey::Tuple(key_items?))
            }
            _ => Err(RuntimeError::InvalidKeyType {
                message: format!("{} cannot be used as a map key", self.type_name()),
            }),
        }
    }
}

impl MapKey {
    /// Convert MapKey back to a Value
    pub fn to_value(&self) -> Value {
        match self {
            MapKey::Number(OrderedFloat(n)) => Value::Number(*n),
            MapKey::Boolean(b) => Value::Boolean(*b),
            MapKey::String(s) => Value::String(s.clone()),
            MapKey::Tuple(items) => Value::Tuple(items.iter().map(|k| k.to_value()).collect()),
        }
    }

    /// Convert a Value to a MapKey
    pub fn from_value(value: Value) -> Result<MapKey, RuntimeError> {
        value.try_into_map_key()
    }
}

impl From<Param> for ParamSpec {
    fn from(param: Param) -> Self {
        ParamSpec {
            name: param.name,
            default: param.default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal, Param};
    use crate::token::Span;

    #[test]
    fn test_type_name() {
        assert_eq!(Value::Number(42.0).type_name(), "number");
        assert_eq!(Value::Boolean(true).type_name(), "boolean");
        assert_eq!(Value::String("hello".to_string()).type_name(), "string");
        assert_eq!(Value::List(vec![]).type_name(), "list");
        assert_eq!(Value::Map(indexmap::IndexMap::new()).type_name(), "map");
        assert_eq!(Value::Tuple(vec![]).type_name(), "tuple");
        assert_eq!(Value::Nil.type_name(), "nil");
    }

    #[test]
    fn test_is_truthy() {
        // Only Boolean(true) is truthy
        assert!(Value::Boolean(true).is_truthy());

        // Everything else is falsy
        assert!(!Value::Boolean(false).is_truthy());
        assert!(!Value::Number(42.0).is_truthy());
        assert!(!Value::Number(0.0).is_truthy());
        assert!(!Value::String("hello".to_string()).is_truthy());
        assert!(!Value::String("".to_string()).is_truthy());
        assert!(!Value::List(vec![]).is_truthy());
        assert!(!Value::List(vec![Value::Number(1.0)]).is_truthy());
        assert!(!Value::Map(indexmap::IndexMap::new()).is_truthy());
        assert!(!Value::Tuple(vec![]).is_truthy());
        assert!(!Value::Nil.is_truthy());
    }

    #[test]
    fn test_try_into_map_key_success() {
        // Number
        let key = Value::Number(42.0).try_into_map_key().unwrap();
        assert!(matches!(key, MapKey::Number(OrderedFloat(42.0))));

        // Boolean
        let key = Value::Boolean(true).try_into_map_key().unwrap();
        assert!(matches!(key, MapKey::Boolean(true)));

        // String
        let key = Value::String("hello".to_string())
            .try_into_map_key()
            .unwrap();
        assert!(matches!(key, MapKey::String(s) if s == "hello"));

        // Tuple
        let tuple = Value::Tuple(vec![
            Value::Number(1.0),
            Value::String("test".to_string()),
            Value::Boolean(false),
        ]);
        let key = tuple.try_into_map_key().unwrap();
        assert!(matches!(key, MapKey::Tuple(items) if items.len() == 3));
    }

    #[test]
    fn test_try_into_map_key_failure() {
        // List cannot be used as map key
        let result = Value::List(vec![Value::Number(1.0)]).try_into_map_key();
        assert!(matches!(result, Err(RuntimeError::InvalidKeyType { .. })));

        // Map cannot be used as map key
        let result = Value::Map(indexmap::IndexMap::new()).try_into_map_key();
        assert!(matches!(result, Err(RuntimeError::InvalidKeyType { .. })));

        // Nil cannot be used as map key
        let result = Value::Nil.try_into_map_key();
        assert!(matches!(result, Err(RuntimeError::InvalidKeyType { .. })));
    }

    #[test]
    fn test_map_key_to_value() {
        // Number
        let key = MapKey::Number(OrderedFloat(42.0));
        let value = key.to_value();
        assert_eq!(value, Value::Number(42.0));

        // Boolean
        let key = MapKey::Boolean(true);
        let value = key.to_value();
        assert_eq!(value, Value::Boolean(true));

        // String
        let key = MapKey::String("hello".to_string());
        let value = key.to_value();
        assert_eq!(value, Value::String("hello".to_string()));

        // Tuple
        let key = MapKey::Tuple(vec![
            MapKey::Number(OrderedFloat(1.0)),
            MapKey::String("test".to_string()),
        ]);
        let value = key.to_value();
        assert_eq!(
            value,
            Value::Tuple(vec![Value::Number(1.0), Value::String("test".to_string()),])
        );
    }

    #[test]
    fn test_param_to_param_spec() {
        let param = Param {
            name: "test_param".to_string(),
            default: Some(Expr::Literal(Literal::Number(42.0, Span::default()))),
            span: Span::default(),
        };

        let param_spec = ParamSpec::from(param);
        assert_eq!(param_spec.name, "test_param");
        assert!(param_spec.default.is_some());
    }
}
