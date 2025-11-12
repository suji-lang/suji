use crate::runtime::builtins::common::one_string_arg;
use md5::Digest;
use suji_values::value::{RuntimeError, Value};

pub fn builtin_crypto_sha1(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = one_string_arg(args, "crypto:sha1")?;
    let digest = sha1::Sha1::digest(s.as_bytes());
    Ok(Value::String(hex::encode(digest)))
}
