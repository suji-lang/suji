//! TOML conversion functions.

use super::types::TomlError;
use indexmap::IndexMap;
use suji_values::value::{DecimalNumber, MapKey, RuntimeError, Value};
use toml::{Value as TomlValue, map::Map as TomlMap};

/// Convert TOML value to SUJI value
pub fn toml_to_suji_value(toml_value: TomlValue) -> Result<Value, RuntimeError> {
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
            let mut suji_array = Vec::new();
            for item in arr {
                suji_array.push(toml_to_suji_value(item)?);
            }
            Ok(Value::List(suji_array))
        }
        TomlValue::Table(table) => {
            let mut suji_map = IndexMap::new();
            for (key, value) in table {
                let suji_key = MapKey::String(key);
                let suji_value = toml_to_suji_value(value)?;
                suji_map.insert(suji_key, suji_value);
            }
            Ok(Value::Map(suji_map))
        }
        TomlValue::Datetime(dt) => {
            // Convert datetime to string
            Ok(Value::String(dt.to_string()))
        }
    }
}

/// Convert SUJI value to TOML value
pub fn suji_to_toml_value(suji_value: &Value) -> Result<TomlValue, RuntimeError> {
    match suji_value {
        Value::Nil => Err(TomlError::Conversion {
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
                        Err(_) => Err(TomlError::Conversion {
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
                    Err(_) => Err(TomlError::Conversion {
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
                toml_array.push(suji_to_toml_value(item)?);
            }
            Ok(TomlValue::Array(toml_array))
        }
        Value::Map(map) => {
            let mut toml_table = TomlMap::new();
            for (key, value) in map {
                let key_str = match key {
                    MapKey::String(s) => s.clone(),
                    _ => {
                        return Err(TomlError::Conversion {
                            message: "TOML keys must be strings".to_string(),
                        }
                        .into());
                    }
                };
                let toml_value = suji_to_toml_value(value)?;
                toml_table.insert(key_str, toml_value);
            }
            Ok(TomlValue::Table(toml_table))
        }
        Value::Tuple(items) => {
            // Convert tuple to array for TOML
            let mut toml_array = Vec::new();
            for item in items {
                toml_array.push(suji_to_toml_value(item)?);
            }
            Ok(TomlValue::Array(toml_array))
        }
        Value::Regex(_) => Err(TomlError::Conversion {
            message: "Regex values cannot be converted to TOML".to_string(),
        }
        .into()),
        Value::Function(_) => Err(TomlError::Conversion {
            message: "Function values cannot be converted to TOML".to_string(),
        }
        .into()),
        Value::Stream(_) | Value::StreamProxy(_) => Err(TomlError::Conversion {
            message: "Stream values cannot be converted to TOML".to_string(),
        }
        .into()),
        Value::EnvMap(_) => Err(TomlError::Conversion {
            message: "Environment map values cannot be converted to TOML".to_string(),
        }
        .into()),
        Value::Module(handle) => Err(TomlError::Conversion {
            message: format!(
                "Cannot serialize module '{}'. Import and use module members instead.",
                handle.module_path
            ),
        }
        .into()),
    }
}
