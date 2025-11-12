// Core types
pub use types::{
    BytecodeFunction, ControlFlow, DecimalNumber, FunctionBody, FunctionValue, MapKey,
    ModuleHandle, OrderedDecimal, ParamSpec, StreamBackend, StreamHandle, StreamProxyKind, Value,
};

// Environment overlay types and functions
pub use super::env_overlay::{EnvProxy, apply_env_overlay_to_command, get_effective_env_var};

// Error types
pub use errors::RuntimeError;

mod comparison;
mod conversion;
mod display;
mod errors;
mod serialization;
mod types;
