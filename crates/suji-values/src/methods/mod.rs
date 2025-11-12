//! Method dispatch for runtime values.
use super::value::{RuntimeError, Value};

pub use common::{ClosureEvaluator, ValueRef};

mod boolean_methods;
mod common;
mod env_map_methods;
mod function_methods;
mod list_methods;
mod map_methods;
mod nil_methods;
mod number_methods;
mod regex_methods;
mod stream_methods;
mod string_methods;
mod tuple_methods;

/// Call a method on a value with the given arguments
///
/// For methods that need to evaluate closures (like list.filter, list.map, list.fold),
/// a callback function must be provided. For other methods, pass `None`.
///
/// # Parameters
/// - `call_closure_fn`: Optional callback to evaluate closures (needed for higher-order methods)
/// - `receiver`: The value to call the method on
/// - `method`: The method name
/// - `args`: Arguments to pass to the method
pub fn call_method<'a>(
    call_closure_fn: Option<ClosureEvaluator<'a>>,
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::String(_) => string_methods::call_string_method(receiver, method, args),
        Value::Number(_) => number_methods::call_number_method(receiver, method, args),
        Value::Boolean(_) => boolean_methods::call_boolean_method(receiver, method, args),
        Value::List(_) => list_methods::call_list_method(call_closure_fn, receiver, method, args),
        Value::Map(_) => map_methods::call_map_method(receiver, method, args),
        Value::EnvMap(_) => env_map_methods::call_env_map_method(receiver, method, args),
        Value::Tuple(_) => tuple_methods::call_tuple_method(receiver, method, args),
        Value::Regex(_) => regex_methods::call_regex_method(receiver, method, args),
        Value::Function(_) => function_methods::call_function_method(receiver, method, args),
        Value::Stream(_) => stream_methods::call_stream_method(receiver, method, args),
        Value::StreamProxy(_) => stream_methods::call_stream_method(receiver, method, args),
        Value::Module(handle) => Err(RuntimeError::InvalidOperation {
            message: format!("Cannot call methods on module '{}'.", handle.module_path),
        }),
        Value::Nil => nil_methods::call_nil_method(receiver, method, args),
    }
}
