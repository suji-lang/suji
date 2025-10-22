use crate::runtime::builtins::common::one_string_arg;
use sha2::Digest;
use sha2::Sha256;
use suji_runtime::value::{RuntimeError, Value};

pub fn builtin_crypto_sha256(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = one_string_arg(args, "crypto:sha256")?;
    let digest = Sha256::digest(s.as_bytes());
    Ok(Value::String(hex::encode(digest)))
}
