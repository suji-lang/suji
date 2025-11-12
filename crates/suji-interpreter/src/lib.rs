//! AST interpreter

mod interpreter;
pub use interpreter::AstInterpreter;

mod eval;
pub use eval::eval_module_source_callback;
