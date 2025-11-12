use super::types::MapKey;
use std::hash::{Hash, Hasher};

impl Hash for MapKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            MapKey::Number(ordered_decimal) => {
                0u8.hash(state); // discriminant
                ordered_decimal.hash(state);
            }
            MapKey::Boolean(b) => {
                1u8.hash(state); // discriminant
                b.hash(state);
            }
            MapKey::String(s) => {
                2u8.hash(state); // discriminant
                s.hash(state);
            }
            MapKey::Tuple(items) => {
                3u8.hash(state); // discriminant
                items.hash(state);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::OrderedDecimal;
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::str::FromStr;

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn test_map_key_hash() {
        // Same values should have same hash
        let key1 = MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(42)));
        let key2 = MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(42)));
        assert_eq!(hash(&key1), hash(&key2));

        let key3 = MapKey::String("hello".to_string());
        let key4 = MapKey::String("hello".to_string());
        assert_eq!(hash(&key3), hash(&key4));

        let key5 = MapKey::Boolean(true);
        let key6 = MapKey::Boolean(true);
        assert_eq!(hash(&key5), hash(&key6));

        // Different values should have different hashes
        let key7 = MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(42)));
        let key8 = MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(43)));
        assert_ne!(hash(&key7), hash(&key8));

        let key9 = MapKey::String("hello".to_string());
        let key10 = MapKey::String("world".to_string());
        assert_ne!(hash(&key9), hash(&key10));

        // Different types should have different hashes
        let key11 = MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(42)));
        let key12 = MapKey::String("42".to_string());
        assert_ne!(hash(&key11), hash(&key12));
    }

    #[test]
    fn test_ordered_decimal_hash() {
        // Same values should have same hash
        let d1 = OrderedDecimal::new(rust_decimal::Decimal::from(42));
        let d2 = OrderedDecimal::new(rust_decimal::Decimal::from(42));
        assert_eq!(hash(&d1), hash(&d2));

        // Different values should have different hashes
        let d3 = OrderedDecimal::new(rust_decimal::Decimal::from(42));
        let d4 = OrderedDecimal::new(rust_decimal::Decimal::from(43));
        assert_ne!(hash(&d3), hash(&d4));

        // Normalized decimals should have same hash
        let d5 = OrderedDecimal::new(rust_decimal::Decimal::from_str("1.0").unwrap());
        let d6 = OrderedDecimal::new(rust_decimal::Decimal::from_str("1.00").unwrap());
        assert_eq!(hash(&d5), hash(&d6));
    }

    #[test]
    fn test_map_key_hash_with_tuples() {
        let tuple1 = MapKey::Tuple(vec![
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(1))),
            MapKey::String("test".to_string()),
        ]);
        let tuple2 = MapKey::Tuple(vec![
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(1))),
            MapKey::String("test".to_string()),
        ]);
        assert_eq!(hash(&tuple1), hash(&tuple2));

        let tuple3 = MapKey::Tuple(vec![
            MapKey::Number(OrderedDecimal::new(rust_decimal::Decimal::from(2))),
            MapKey::String("test".to_string()),
        ]);
        assert_ne!(hash(&tuple1), hash(&tuple3));
    }

    #[test]
    fn test_hash_consistency() {
        // Hash should be consistent across multiple calls
        let key = MapKey::String("consistent".to_string());
        let hash1 = hash(&key);
        let hash2 = hash(&key);
        assert_eq!(hash1, hash2);

        let decimal = OrderedDecimal::new(rust_decimal::Decimal::from_str("3.14159").unwrap());
        let hash3 = hash(&decimal);
        let hash4 = hash(&decimal);
        assert_eq!(hash3, hash4);
    }
}
