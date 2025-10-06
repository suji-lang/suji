//! Loader for SUJI-language builtins (print, println, random helpers).

use std::rc::Rc;
use suji_parser::parse_program;
use suji_runtime::env::Env;
use suji_runtime::eval::eval_export;
use suji_runtime::value::{MapKey, RuntimeError, Value};

pub fn load_print() -> Result<Value, RuntimeError> {
    let source = include_str!("si_sources/print.si");
    let program = parse_program(source).map_err(|e| RuntimeError::InvalidOperation {
        message: format!("Failed to parse SUJI builtin 'print': {e}"),
    })?;

    let env = Rc::new(Env::new());

    // Find export spec and evaluate
    let export_spec = program
        .iter()
        .find_map(|stmt| match stmt {
            suji_ast::ast::Stmt::Export { spec, .. } => Some(spec.clone()),
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
    let source = include_str!("si_sources/println.si");
    let program = parse_program(source).map_err(|e| RuntimeError::InvalidOperation {
        message: format!("Failed to parse SUJI builtin 'println': {e}"),
    })?;

    let env = Rc::new(Env::new());

    // Find export spec and evaluate
    let export_spec = program
        .iter()
        .find_map(|stmt| match stmt {
            suji_ast::ast::Stmt::Export { spec, .. } => Some(spec.clone()),
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

/// Load SUJI helpers for the random module: integer, pick, shuffle, sample.
/// The provided `random_fn` will be bound into the environment as `random` so
/// that helpers can call `random()` without importing.
pub fn load_random_helpers(
    random_fn: Value,
) -> Result<indexmap::IndexMap<MapKey, Value>, RuntimeError> {
    let source = include_str!("si_sources/random_helpers.si");
    let program = parse_program(source).map_err(|e| RuntimeError::InvalidOperation {
        message: format!("Failed to parse SUJI builtin 'random_helpers': {e}"),
    })?;

    let env = Rc::new(Env::new());
    // Pre-bind `random` name to provided function value
    env.define_or_set("random", random_fn);

    // Find export spec and evaluate
    let export_spec = program
        .iter()
        .find_map(|stmt| match stmt {
            suji_ast::ast::Stmt::Export { spec, .. } => Some(spec.clone()),
            _ => None,
        })
        .ok_or_else(|| RuntimeError::InvalidOperation {
            message: "Builtin 'random_helpers' did not contain an export statement".to_string(),
        })?;

    let module = eval_export(&export_spec, env)?.module;
    if let Value::Map(map) = module {
        Ok(map)
    } else {
        Err(RuntimeError::InvalidOperation {
            message: "Builtin 'random_helpers' did not produce an export map".to_string(),
        })
    }
}
