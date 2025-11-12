use hmac::{Hmac, Mac};
use sha2::Sha256;
use suji_values::value::{RuntimeError, Value};

pub fn builtin_crypto_hmac_sha256(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            message: "crypto:hmac_sha256 expects 2 arguments".to_string(),
        });
    }
    let key = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "first argument must be string".to_string(),
            });
        }
    };
    let text = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "second argument must be string".to_string(),
            });
        }
    };
    let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes()).map_err(|_| {
        RuntimeError::InvalidOperation {
            message: "invalid HMAC key".to_string(),
        }
    })?;
    mac.update(text.as_bytes());
    let out = mac.finalize().into_bytes();
    Ok(Value::String(hex::encode(out)))
}
