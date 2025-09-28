//! TOML conversion functions.

use super::types::TomlError;
use crate::runtime::value::{DecimalNumber, MapKey, RuntimeError, Value};
use indexmap::IndexMap;
use toml::{Value as TomlValue, map::Map as TomlMap};

/// Convert TOML value to nn value
pub fn toml_to_nn_value(toml_value: TomlValue) -> Result<Value, RuntimeError> {
    match toml_value {
        TomlValue::Boolean(b) => Ok(Value::Boolean(b)),
        TomlValue::Integer(i) => Ok(Value::Number(DecimalNumber::from_i64(i))),
        TomlValue::Float(f) => {
            // Convert float to string and then parse as decimal for precision
            let float_str = f.to_string();
            match DecimalNumber::parse(&float_str) {
                Ok(decimal) => Ok(Value::Number(decimal)),
                Err(_) => Err(TomlError::ParseError {
                    message: format!("TOML float '{}' cannot be converted to decimal", float_str),
                    toml_input: None,
                }
                .into()),
            }
        }
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
            if n.is_integer() {
                if let Some(i) = n.to_i64_checked() {
                    Ok(TomlValue::Integer(i))
                } else {
                    // Integer too large, convert to float
                    let float_str = n.to_string();
                    match float_str.parse::<f64>() {
                        Ok(f) => Ok(TomlValue::Float(f)),
                        Err(_) => Err(TomlError::ConversionError {
                            message: format!("Number '{}' cannot be converted to TOML", float_str),
                        }
                        .into()),
                    }
                }
            } else {
                // Convert decimal to float for TOML
                let float_str = n.to_string();
                match float_str.parse::<f64>() {
                    Ok(f) => Ok(TomlValue::Float(f)),
                    Err(_) => Err(TomlError::ConversionError {
                        message: format!("Number '{}' cannot be converted to TOML", float_str),
                    }
                    .into()),
                }
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
        Value::Stream(_) => Err(TomlError::ConversionError {
            message: "Stream values cannot be converted to TOML".to_string(),
        }
        .into()),
        Value::EnvMap(_) => Err(TomlError::ConversionError {
            message: "Environment map values cannot be converted to TOML".to_string(),
        }
        .into()),
    }
}
