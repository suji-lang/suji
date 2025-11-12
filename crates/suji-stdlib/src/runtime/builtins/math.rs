use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use suji_values::value::{DecimalNumber, RuntimeError, Value};

pub fn to_f64(arg: &Value, name: &str) -> Result<f64, RuntimeError> {
    match arg {
        Value::Number(n) => {
            n.inner()
                .to_f64()
                .ok_or_else(|| RuntimeError::InvalidNumberConversion {
                    message: format!("{} cannot be represented as f64", name),
                })
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("{} must be a number", name),
        }),
    }
}

pub fn from_f64(v: f64) -> Result<Value, RuntimeError> {
    if !v.is_finite() {
        return Err(RuntimeError::InvalidOperation {
            message: "math result not finite".to_string(),
        });
    }
    let dec = rust_decimal::Decimal::from_f64(v).ok_or_else(|| RuntimeError::InvalidOperation {
        message: "cannot convert result to decimal".to_string(),
    })?;
    Ok(Value::Number(DecimalNumber(dec)))
}

pub fn to_decimal(arg: &Value, name: &str) -> Result<Decimal, RuntimeError> {
    match arg {
        Value::Number(n) => Ok(n.inner()),
        _ => Err(RuntimeError::TypeError {
            message: format!("{} must be a number", name),
        }),
    }
}

pub fn from_decimal(v: Decimal) -> Value {
    Value::Number(DecimalNumber(v))
}

pub fn ensure_in_unit_interval(x: f64, fname: &str) -> Result<(), RuntimeError> {
    if !(-1.0..=1.0).contains(&x) {
        return Err(RuntimeError::InvalidOperation {
            message: format!("{} domain is [-1,1]", fname),
        });
    }
    Ok(())
}

pub fn ensure_positive_decimal(x: Decimal, fname: &str) -> Result<(), RuntimeError> {
    if x <= Decimal::ZERO {
        return Err(RuntimeError::InvalidOperation {
            message: format!("{} domain is (0, +inf)", fname),
        });
    }
    Ok(())
}
