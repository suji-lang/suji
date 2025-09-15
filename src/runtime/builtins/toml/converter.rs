//! TOML conversion functions
//!
//! This module handles conversion between NN values and TOML values.

use super::types::TomlError;
use crate::runtime::value::{MapKey, RuntimeError, Value};
use indexmap::IndexMap;
use toml::{Value as TomlValue, map::Map as TomlMap};

/// Convert TOML value to nn value
pub fn toml_to_nn_value(toml_value: TomlValue) -> Result<Value, RuntimeError> {
    match toml_value {
        TomlValue::Boolean(b) => Ok(Value::Boolean(b)),
        TomlValue::Integer(i) => Ok(Value::Number(i as f64)),
        TomlValue::Float(f) => Ok(Value::Number(f)),
        TomlValue::String(s) => Ok(Value::String(s)),
        TomlValue::Array(arr) => {
            let mut nn_array = Vec::new();
            for item in arr {
                nn_array.push(toml_to_nn_value(item)?);
            }
            Ok(Value::List(nn_array))
        }
        TomlValue::Table(table) => {
            let mut nn_map = IndexMap::new();
            for (key, value) in table {
                let nn_key = MapKey::String(key);
                let nn_value = toml_to_nn_value(value)?;
                nn_map.insert(nn_key, nn_value);
            }
            Ok(Value::Map(nn_map))
        }
        TomlValue::Datetime(dt) => {
            // Convert datetime to string
            Ok(Value::String(dt.to_string()))
        }
    }
}

/// Convert nn value to TOML value
pub fn nn_to_toml_value(nn_value: &Value) -> Result<TomlValue, RuntimeError> {
    match nn_value {
        Value::Nil => Err(TomlError::ConversionError {
            message: "TOML does not support nil values".to_string(),
        }
        .into()),
        Value::Boolean(b) => Ok(TomlValue::Boolean(*b)),
        Value::Number(n) => {
            if n.fract() == 0.0 {
                Ok(TomlValue::Integer(*n as i64))
            } else {
                Ok(TomlValue::Float(*n))
            }
        }
        Value::String(s) => Ok(TomlValue::String(s.clone())),
        Value::List(items) => {
            let mut toml_array = Vec::new();
            for item in items {
                toml_array.push(nn_to_toml_value(item)?);
            }
            Ok(TomlValue::Array(toml_array))
        }
        Value::Map(map) => {
            let mut toml_table = TomlMap::new();
            for (key, value) in map {
                let key_str = match key {
                    MapKey::String(s) => s.clone(),
                    _ => {
                        return Err(TomlError::ConversionError {
                            message: "TOML keys must be strings".to_string(),
                        }
                        .into());
                    }
                };
                let toml_value = nn_to_toml_value(value)?;
                toml_table.insert(key_str, toml_value);
            }
            Ok(TomlValue::Table(toml_table))
        }
        Value::Tuple(items) => {
            // Convert tuple to array for TOML
            let mut toml_array = Vec::new();
            for item in items {
                toml_array.push(nn_to_toml_value(item)?);
            }
            Ok(TomlValue::Array(toml_array))
        }
        Value::Regex(_) => Err(TomlError::ConversionError {
            message: "Regex values cannot be converted to TOML".to_string(),
        }
        .into()),
        Value::Function(_) => Err(TomlError::ConversionError {
            message: "Function values cannot be converted to TOML".to_string(),
        }
        .into()),
    }
}
