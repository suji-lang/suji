//! std:random module creation and wiring of helpers.

use crate::runtime::builtins::suji_loader::load_random_helpers;
use indexmap::IndexMap;
use suji_runtime::value::{MapKey, Value};

/// Create a special built-in function value that can be called normally
fn create_builtin_function_value(name: &str) -> Value {
    use std::rc::Rc;
    use suji_ast::ast::Stmt;
    use suji_lexer::token::Span;
    use suji_runtime::env::Env;
    use suji_runtime::value::{FunctionValue, ParamSpec};

    // Create a function value with a special marker that identifies it as built-in
    Value::Function(FunctionValue {
        params: vec![ParamSpec {
            name: format!("__builtin_{}__", name), // Special marker with function name
            default: None,
        }],
        body: Stmt::Block {
            statements: vec![],
            span: Span::default(),
        },
        env: Rc::new(Env::new()),
    })
}

/// Create the random module under std
pub fn create_random_module() -> Value {
    let mut random_map = IndexMap::new();

    // Builtin functions implemented in Rust
    let random_fn = create_builtin_function_value("random_random");
    let seed_fn = create_builtin_function_value("random_seed");

    // Insert base functions
    random_map.insert(MapKey::String("random".to_string()), random_fn.clone());
    random_map.insert(MapKey::String("seed".to_string()), seed_fn);

    // Load SUJI helpers and merge into the module
    if let Ok(helpers) = load_random_helpers(random_fn) {
        for (k, v) in helpers.into_iter() {
            random_map.insert(k, v);
        }
    }

    Value::Map(random_map)
}
