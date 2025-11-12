// Suji Values Crate

// Value types and errors
pub mod value;
pub use value::*;

// Environment for variable bindings
pub mod env;
pub use env::Env;

// Environment variable overlays for shell commands
pub mod env_overlay;
pub use env_overlay::*;

// IO context for stream redirection
pub mod io_context;
pub use io_context::IoContext;

// Range type and operations
pub mod range;
pub use range::*;

// Regex compilation and caching
pub mod regex;
pub use regex::*;

// Shell command execution
pub mod shell;
pub use shell::*;

// String interpolation
pub mod string_template;
pub use string_template::*;

// Methods on values
pub mod methods;
pub use methods::{ValueRef, call_method};
