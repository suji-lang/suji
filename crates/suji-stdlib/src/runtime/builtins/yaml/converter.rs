//! YAML conversion functions.

use super::types::YamlError;
use indexmap::IndexMap;
use suji_runtime::value::{DecimalNumber, MapKey, RuntimeError, Value};
use yaml_rust2::Yaml;

/// Convert YAML value to SUJI value
pub fn yaml_to_suji_value(yaml_value: Yaml) -> Result<Value, RuntimeError> {
    match yaml_value {
        Yaml::Null => Ok(Value::Nil),
        Yaml::Boolean(b) => Ok(Value::Boolean(b)),
        Yaml::Integer(i) => Ok(Value::Number(DecimalNumber::from_i64(i))),
        Yaml::Real(s) => match DecimalNumber::parse(&s) {
            Ok(decimal) => Ok(Value::Number(decimal)),
            Err(_) => Err(YamlError::ParseError {
                message: "Invalid YAML number format".to_string(),
                yaml_input: Some(s),
            }
            .into()),
        },
        Yaml::String(s) => Ok(Value::String(s)),
        Yaml::Array(seq) => {
            let mut suji_list = Vec::new();
            for item in seq {
                suji_list.push(yaml_to_suji_value(item)?);
            }
            Ok(Value::List(suji_list))
        }
        Yaml::Hash(map) => {
            let mut suji_map = IndexMap::new();
            for (key, value) in map {
                let suji_key = yaml_to_suji_value(key)?;
                let suji_value = yaml_to_suji_value(value)?;
                suji_map.insert(MapKey::from_value(suji_key)?, suji_value);
            }
            Ok(Value::Map(suji_map))
        }
        Yaml::Alias(_) | Yaml::BadValue => Err(YamlError::ParseError {
            message: "Unsupported YAML type (alias or bad value)".to_string(),
            yaml_input: None,
        }
        .into()),
    }
}

/// Convert SUJI value to YAML value
pub fn suji_to_yaml_value(suji_value: &Value) -> Result<Yaml, RuntimeError> {
    match suji_value {
        Value::Nil => Ok(Yaml::Null),
        Value::Boolean(b) => Ok(Yaml::Boolean(*b)),
        Value::Number(n) => {
            if n.is_integer() {
                if let Some(i) = n.to_i64_checked() {
                    Ok(Yaml::Integer(i))
                } else {
                    Ok(Yaml::Real(n.to_string()))
                }
            } else {
                Ok(Yaml::Real(n.to_string()))
            }
        }
        Value::String(s) => Ok(Yaml::String(s.clone())),
        Value::List(list) => {
            let mut yaml_seq = Vec::new();
            for item in list {
                yaml_seq.push(suji_to_yaml_value(item)?);
            }
            Ok(Yaml::Array(yaml_seq))
        }
        Value::Map(map) => {
            let mut yaml_map = yaml_rust2::yaml::Hash::new();
            for (key, value) in map {
                let yaml_key = suji_to_yaml_value(&key.to_value())?;
                let yaml_value = suji_to_yaml_value(value)?;
                yaml_map.insert(yaml_key, yaml_value);
            }
            Ok(Yaml::Hash(yaml_map))
        }
        Value::Tuple(tuple) => {
            // Convert tuple to array for YAML
            let mut yaml_seq = Vec::new();
            for item in tuple {
                yaml_seq.push(suji_to_yaml_value(item)?);
            }
            Ok(Yaml::Array(yaml_seq))
        }
        Value::Regex(_) => Err(YamlError::GenerateError {
            message: "Regex values cannot be converted to YAML".to_string(),
            value_type: "regex".to_string(),
        }
        .into()),
        Value::Function(_) => Err(YamlError::GenerateError {
            message: "Function values cannot be converted to YAML".to_string(),
            value_type: "function".to_string(),
        }
        .into()),
        Value::Stream(_) => Err(YamlError::GenerateError {
            message: "Stream values cannot be converted to YAML".to_string(),
            value_type: "stream".to_string(),
        }
        .into()),
        Value::EnvMap(_) => Err(YamlError::GenerateError {
            message: "Environment map values cannot be converted to YAML".to_string(),
            value_type: "env".to_string(),
        }
        .into()),
    }
}
