//! NN-language builtin loader for std:print and std:println
//!
//! Evaluates embedded NN source files by pre-binding required names in an
//! evaluation environment and directly evaluating the export statement.

use crate::parser::parse_program;
use crate::runtime::env::Env;
use crate::runtime::eval::eval_export;
use crate::runtime::value::{MapKey, RuntimeError, Value};
use std::rc::Rc;

pub fn load_print() -> Result<Value, RuntimeError> {
    let source = include_str!("nn_sources/print.nn");
    let program = parse_program(source).map_err(|e| RuntimeError::InvalidOperation {
        message: format!("Failed to parse NN builtin 'print': {e}"),
    })?;

    let env = Rc::new(Env::new());

    // Find export spec and evaluate
    let export_spec = program
        .iter()
        .find_map(|stmt| match stmt {
            crate::ast::Stmt::Export { spec, .. } => Some(spec.clone()),
            _ => None,
        })
        .ok_or_else(|| RuntimeError::InvalidOperation {
            message: "Builtin 'print' did not contain an export statement".to_string(),
        })?;

    let module = eval_export(&export_spec, env)?.module;
    if let Value::Map(map) = module {
        map.get(&MapKey::String("print".to_string()))
            .cloned()
            .ok_or_else(|| RuntimeError::InvalidOperation {
                message: "Builtin 'print' did not export a function named 'print'".to_string(),
            })
    } else {
        Err(RuntimeError::InvalidOperation {
            message: "Builtin 'print' did not produce an export map".to_string(),
        })
    }
}

pub fn load_println() -> Result<Value, RuntimeError> {
    let source = include_str!("nn_sources/println.nn");
    let program = parse_program(source).map_err(|e| RuntimeError::InvalidOperation {
        message: format!("Failed to parse NN builtin 'println': {e}"),
    })?;

    let env = Rc::new(Env::new());

    // Find export spec and evaluate
    let export_spec = program
        .iter()
        .find_map(|stmt| match stmt {
            crate::ast::Stmt::Export { spec, .. } => Some(spec.clone()),
            _ => None,
        })
        .ok_or_else(|| RuntimeError::InvalidOperation {
            message: "Builtin 'println' did not contain an export statement".to_string(),
        })?;

    let module = eval_export(&export_spec, env)?.module;
    if let Value::Map(map) = module {
        map.get(&MapKey::String("println".to_string()))
            .cloned()
            .ok_or_else(|| RuntimeError::InvalidOperation {
                message: "Builtin 'println' did not export a function named 'println'".to_string(),
            })
    } else {
        Err(RuntimeError::InvalidOperation {
            message: "Builtin 'println' did not produce an export map".to_string(),
        })
    }
}
