use super::types::{MapKey, OrderedFloat};
use std::hash::{Hash, Hasher};

impl Hash for MapKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            MapKey::Number(OrderedFloat(n)) => {
                0u8.hash(state); // discriminant
                n.to_bits().hash(state);
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

impl Hash for OrderedFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn test_map_key_hash() {
        // Same values should have same hash
        let key1 = MapKey::Number(OrderedFloat(42.0));
        let key2 = MapKey::Number(OrderedFloat(42.0));
        assert_eq!(hash(&key1), hash(&key2));

        let key3 = MapKey::String("hello".to_string());
        let key4 = MapKey::String("hello".to_string());
        assert_eq!(hash(&key3), hash(&key4));

        let key5 = MapKey::Boolean(true);
        let key6 = MapKey::Boolean(true);
        assert_eq!(hash(&key5), hash(&key6));

        // Different values should have different hashes
        let key7 = MapKey::Number(OrderedFloat(42.0));
        let key8 = MapKey::Number(OrderedFloat(43.0));
        assert_ne!(hash(&key7), hash(&key8));

        let key9 = MapKey::String("hello".to_string());
        let key10 = MapKey::String("world".to_string());
        assert_ne!(hash(&key9), hash(&key10));

        // Different types should have different hashes
        let key11 = MapKey::Number(OrderedFloat(42.0));
        let key12 = MapKey::String("42".to_string());
        assert_ne!(hash(&key11), hash(&key12));
    }

    #[test]
    fn test_ordered_float_hash() {
        // Same values should have same hash
        let f1 = OrderedFloat(42.0);
        let f2 = OrderedFloat(42.0);
        assert_eq!(hash(&f1), hash(&f2));

        // Different values should have different hashes
        let f3 = OrderedFloat(42.0);
        let f4 = OrderedFloat(43.0);
        assert_ne!(hash(&f3), hash(&f4));

        // NaN should have consistent hash
        let nan1 = OrderedFloat(f64::NAN);
        let nan2 = OrderedFloat(f64::NAN);
        assert_eq!(hash(&nan1), hash(&nan2));
    }

    #[test]
    fn test_map_key_hash_with_tuples() {
        let tuple1 = MapKey::Tuple(vec![
            MapKey::Number(OrderedFloat(1.0)),
            MapKey::String("test".to_string()),
        ]);
        let tuple2 = MapKey::Tuple(vec![
            MapKey::Number(OrderedFloat(1.0)),
            MapKey::String("test".to_string()),
        ]);
        assert_eq!(hash(&tuple1), hash(&tuple2));

        let tuple3 = MapKey::Tuple(vec![
            MapKey::Number(OrderedFloat(2.0)),
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

        let float = OrderedFloat(std::f64::consts::PI);
        let hash3 = hash(&float);
        let hash4 = hash(&float);
        assert_eq!(hash3, hash4);
    }
}
