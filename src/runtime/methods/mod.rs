use super::value::{RuntimeError, Value};

pub use common::ValueRef;

mod common;
mod list_methods;
mod map_methods;
mod number_methods;
mod string_methods;
mod tuple_methods;

/// Call a method on a value with the given arguments
pub fn call_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::String(_) => string_methods::call_string_method(receiver, method, args),
        Value::Number(_) => number_methods::call_number_method(receiver, method, args),
        Value::List(_) => list_methods::call_list_method(receiver, method, args),
        Value::Map(_) => map_methods::call_map_method(receiver, method, args),
        Value::Tuple(_) => tuple_methods::call_tuple_method(receiver, method, args),
        _ => Err(RuntimeError::MethodError {
            message: format!("{} has no method '{}'", receiver.get().type_name(), method),
        }),
    }
}
