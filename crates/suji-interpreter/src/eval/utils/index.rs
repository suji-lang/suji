use suji_values::RuntimeError;

/// Normalize an index with bounds checking
pub fn normalize_index(idx: i64, len: usize) -> Result<usize, RuntimeError> {
    // Handle negative indices: convert to positive index from end
    let normalized_idx = if idx < 0 { len as i64 + idx } else { idx };

    // Bounds checking
    if normalized_idx < 0 || normalized_idx >= len as i64 {
        return Err(RuntimeError::IndexOutOfBounds {
            message: format!("Index {} out of bounds for length {}", idx, len),
        });
    }

    Ok(normalized_idx as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_indices() {
        assert_eq!(normalize_index(0, 5).unwrap(), 0);
        assert_eq!(normalize_index(1, 5).unwrap(), 1);
        assert_eq!(normalize_index(4, 5).unwrap(), 4);
    }

    #[test]
    fn test_negative_indices() {
        assert_eq!(normalize_index(-1, 5).unwrap(), 4);
        assert_eq!(normalize_index(-2, 5).unwrap(), 3);
        assert_eq!(normalize_index(-5, 5).unwrap(), 0);
    }

    #[test]
    fn test_out_of_bounds_positive() {
        assert!(normalize_index(5, 5).is_err());
        assert!(normalize_index(10, 5).is_err());
    }

    #[test]
    fn test_out_of_bounds_negative() {
        assert!(normalize_index(-6, 5).is_err());
        assert!(normalize_index(-10, 5).is_err());
    }

    #[test]
    fn test_empty_collection() {
        assert!(normalize_index(0, 0).is_err());
        assert!(normalize_index(-1, 0).is_err());
    }

    #[test]
    fn test_single_element() {
        assert_eq!(normalize_index(0, 1).unwrap(), 0);
        assert_eq!(normalize_index(-1, 1).unwrap(), 0);
        assert!(normalize_index(1, 1).is_err());
        assert!(normalize_index(-2, 1).is_err());
    }
}
