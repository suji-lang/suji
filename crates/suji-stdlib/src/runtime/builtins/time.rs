use indexmap::IndexMap;
use suji_values::value::{DecimalNumber, MapKey, Value};

pub fn map_now(epoch_ms: i64, iso: String, tz: String) -> Value {
    let mut map = IndexMap::new();
    map.insert(
        MapKey::String("epoch_ms".to_string()),
        Value::Number(DecimalNumber::from_i64(epoch_ms)),
    );
    map.insert(MapKey::String("iso".to_string()), Value::String(iso));
    map.insert(MapKey::String("tz".to_string()), Value::String(tz));
    Value::Map(map)
}

pub fn map_epoch_tz(epoch_ms: i64, tz: String) -> Value {
    let mut map = IndexMap::new();
    map.insert(
        MapKey::String("epoch_ms".to_string()),
        Value::Number(DecimalNumber::from_i64(epoch_ms)),
    );
    map.insert(MapKey::String("tz".to_string()), Value::String(tz));
    Value::Map(map)
}
