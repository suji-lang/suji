use crate::runtime::builtins::common::one_string_arg;
use md5::Md5;
use sha2::Digest;
use suji_values::value::{RuntimeError, Value};

pub fn builtin_crypto_md5(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = one_string_arg(args, "crypto:md5")?;
    let digest = Md5::digest(s.as_bytes());
    Ok(Value::String(hex::encode(digest)))
}
