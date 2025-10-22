use crate::runtime::builtins::common::one_string_arg;
use sha2::Digest;
use sha2::Sha512;
use suji_runtime::value::{RuntimeError, Value};

pub fn builtin_crypto_sha512(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = one_string_arg(args, "crypto:sha512")?;
    let digest = Sha512::digest(s.as_bytes());
    Ok(Value::String(hex::encode(digest)))
}
