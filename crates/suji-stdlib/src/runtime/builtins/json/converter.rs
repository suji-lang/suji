//! JSON conversion functions.

use super::types::JsonError;
use indexmap::IndexMap;
use rust_decimal::prelude::ToPrimitive;
use serde_json::{Map as JsonMap, Value as JsonValue};
use suji_runtime::value::{DecimalNumber, MapKey, RuntimeError, Value};

/// Convert JSON value to SUJI value
pub fn json_to_suji_value(json_value: JsonValue) -> Result<Value, RuntimeError> {
    match json_value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Boolean(b)),
        JsonValue::Number(n) => {
            // Convert JSON number to string and then parse as decimal
            let number_str = n.to_string();
            match DecimalNumber::parse(&number_str) {
                Ok(decimal) => Ok(Value::Number(decimal)),
                Err(_) => Err(JsonError::ParseError {
                    message: format!(
                        "JSON number '{}' cannot be converted to decimal",
                        number_str
                    ),
                    json_input: None,
                }
                .into()),
            }
        }
        JsonValue::String(s) => Ok(Value::String(s)),
        JsonValue::Array(arr) => {
            let mut suji_array = Vec::new();
            for item in arr {
                suji_array.push(json_to_suji_value(item)?);
            }
            Ok(Value::List(suji_array))
        }
        JsonValue::Object(obj) => {
            let mut suji_map = IndexMap::new();
            for (key, value) in obj {
                let suji_key = MapKey::String(key);
                let suji_value = json_to_suji_value(value)?;
                suji_map.insert(suji_key, suji_value);
            }
            Ok(Value::Map(suji_map))
        }
    }
}

/// Convert SUJI value to JSON value
pub fn suji_to_json_value(suji_value: &Value) -> Result<JsonValue, RuntimeError> {
    match suji_value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
        Value::Number(n) => {
            if n.is_integer() {
                n.to_i64_checked()
                    .map(|int| JsonValue::Number(int.into()))
                    .ok_or_else(|| {
                        JsonError::GenerateError {
                            message: "Integer out of JSON range".to_string(),
                            value_type: "number".to_string(),
                        }
                        .into()
                    })
            } else {
                n.inner()
                    .to_f64()
                    .and_then(serde_json::Number::from_f64)
                    .map(JsonValue::Number)
                    .ok_or_else(|| {
                        JsonError::GenerateError {
                            message: "Decimal cannot be represented in JSON".to_string(),
                            value_type: "number".to_string(),
                        }
                        .into()
                    })
            }
        }
        Value::String(s) => Ok(JsonValue::String(s.clone())),
        Value::List(items) => {
            let mut json_array = Vec::new();
            for item in items {
                json_array.push(suji_to_json_value(item)?);
            }
            Ok(JsonValue::Array(json_array))
        }
        Value::Map(map) => {
            let mut json_obj = JsonMap::new();
            for (key, value) in map {
                let key_str = match key {
                    MapKey::String(s) => s.clone(),
                    MapKey::Number(n) => n.0.to_string(),
                    MapKey::Boolean(b) => b.to_string(),
                    MapKey::Tuple(_) => {
                        return Err(JsonError::GenerateError {
                            message: "Map keys with tuples cannot be converted to JSON".to_string(),
                            value_type: "map with tuple keys".to_string(),
                        }
                        .into());
                    }
                };
                json_obj.insert(key_str, suji_to_json_value(value)?);
            }
            Ok(JsonValue::Object(json_obj))
        }
        Value::Tuple(items) => {
            let mut json_array = Vec::new();
            for item in items {
                json_array.push(suji_to_json_value(item)?);
            }
            Ok(JsonValue::Array(json_array))
        }
        Value::Regex(_) => Err(JsonError::GenerateError {
            message: "Regex values cannot be converted to JSON".to_string(),
            value_type: "regex".to_string(),
        }
        .into()),
        Value::Function(_) => Err(JsonError::GenerateError {
            message: "Function values cannot be converted to JSON".to_string(),
            value_type: "function".to_string(),
        }
        .into()),
        Value::Stream(_) | Value::StreamProxy(_) => Err(JsonError::GenerateError {
            message: "Stream values cannot be converted to JSON".to_string(),
            value_type: "stream".to_string(),
        }
        .into()),
        Value::EnvMap(_) => Err(JsonError::GenerateError {
            message: "Environment map values cannot be converted to JSON".to_string(),
            value_type: "env".to_string(),
        }
        .into()),
    }
}
