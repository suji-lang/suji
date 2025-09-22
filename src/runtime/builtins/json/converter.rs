//! JSON conversion functions.

use super::types::JsonError;
use crate::runtime::value::{MapKey, RuntimeError, Value};
use indexmap::IndexMap;
use serde_json::{Map as JsonMap, Value as JsonValue};

/// Convert JSON value to nn value
pub fn json_to_nn_value(json_value: JsonValue) -> Result<Value, RuntimeError> {
    match json_value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Boolean(b)),
        JsonValue::Number(n) => {
            if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Err(JsonError::ParseError {
                    message: "JSON number too large for nn number type".to_string(),
                    json_input: None,
                }
                .into())
            }
        }
        JsonValue::String(s) => Ok(Value::String(s)),
        JsonValue::Array(arr) => {
            let mut nn_array = Vec::new();
            for item in arr {
                nn_array.push(json_to_nn_value(item)?);
            }
            Ok(Value::List(nn_array))
        }
        JsonValue::Object(obj) => {
            let mut nn_map = IndexMap::new();
            for (key, value) in obj {
                let nn_key = MapKey::String(key);
                let nn_value = json_to_nn_value(value)?;
                nn_map.insert(nn_key, nn_value);
            }
            Ok(Value::Map(nn_map))
        }
    }
}

/// Convert nn value to JSON value
pub fn nn_to_json_value(nn_value: &Value) -> Result<JsonValue, RuntimeError> {
    match nn_value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
        Value::Number(n) => Ok(serde_json::Number::from_f64(*n)
            .ok_or_else(|| JsonError::GenerateError {
                message: "Number cannot be represented in JSON".to_string(),
                value_type: "number".to_string(),
            })?
            .into()),
        Value::String(s) => Ok(JsonValue::String(s.clone())),
        Value::List(items) => {
            let mut json_array = Vec::new();
            for item in items {
                json_array.push(nn_to_json_value(item)?);
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
                json_obj.insert(key_str, nn_to_json_value(value)?);
            }
            Ok(JsonValue::Object(json_obj))
        }
        Value::Tuple(items) => {
            let mut json_array = Vec::new();
            for item in items {
                json_array.push(nn_to_json_value(item)?);
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
        Value::Stream(_) => Err(JsonError::GenerateError {
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
