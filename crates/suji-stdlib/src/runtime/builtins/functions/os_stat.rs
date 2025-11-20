//! Built-in: os:stat(path[, follow_symlinks]) -> map (file/directory metadata).

use indexmap::IndexMap;
use std::fs;
use std::time::SystemTime;
use suji_values::value::{DecimalNumber, MapKey, RuntimeError, Value};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

/// Returns file/directory metadata as a map.
pub fn builtin_os_stat(args: &[Value]) -> Result<Value, RuntimeError> {
    // Validate arity (1-2 args)
    if args.is_empty() || args.len() > 2 {
        return Err(RuntimeError::ArityMismatch {
            message: "os:stat(path[, follow_symlinks]) expects 1 or 2 arguments".to_string(),
        });
    }

    // Extract path
    let path_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "os:stat expects path to be a string".to_string(),
            });
        }
    };

    // Extract follow_symlinks (default false)
    let follow_symlinks = if args.len() == 2 {
        match &args[1] {
            Value::Boolean(b) => *b,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "os:stat expects follow_symlinks to be a boolean".to_string(),
                });
            }
        }
    } else {
        false
    };

    // Get metadata
    let metadata = if follow_symlinks {
        fs::metadata(path_str)
    } else {
        fs::symlink_metadata(path_str)
    }
    .map_err(|e| RuntimeError::InvalidOperation {
        message: format!("Failed to stat '{}': {}", path_str, e),
    })?;

    // Build result map
    let mut result = IndexMap::new();

    // Platform-independent fields
    result.insert(
        MapKey::String("size".to_string()),
        Value::Number(DecimalNumber::from_u64(metadata.len())),
    );

    result.insert(
        MapKey::String("is_directory".to_string()),
        Value::Boolean(metadata.is_dir()),
    );

    result.insert(
        MapKey::String("is_symlink".to_string()),
        Value::Boolean(metadata.is_symlink()),
    );

    // Timestamps (convert to milliseconds since epoch)
    if let Ok(modified) = metadata.modified() {
        let duration = modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        result.insert(
            MapKey::String("mtime".to_string()),
            Value::Number(DecimalNumber::from_u64(duration.as_millis() as u64)),
        );
    }

    if let Ok(accessed) = metadata.accessed() {
        let duration = accessed
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        result.insert(
            MapKey::String("atime".to_string()),
            Value::Number(DecimalNumber::from_u64(duration.as_millis() as u64)),
        );
    }

    if let Ok(created) = metadata.created() {
        let duration = created
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        result.insert(
            MapKey::String("ctime".to_string()),
            Value::Number(DecimalNumber::from_u64(duration.as_millis() as u64)),
        );
    }

    // Symlink target
    let link_target = if metadata.is_symlink() {
        fs::read_link(path_str)
            .ok()
            .and_then(|p| p.to_str().map(|s| Value::String(s.to_string())))
            .unwrap_or(Value::Nil)
    } else {
        Value::Nil
    };
    result.insert(MapKey::String("link".to_string()), link_target);

    // Platform-specific fields
    #[cfg(unix)]
    {
        result.insert(
            MapKey::String("inode".to_string()),
            Value::Number(DecimalNumber::from_u64(metadata.ino())),
        );
        result.insert(
            MapKey::String("mode".to_string()),
            Value::Number(DecimalNumber::from_u64(metadata.mode() as u64)),
        );
        result.insert(
            MapKey::String("uid".to_string()),
            Value::Number(DecimalNumber::from_u64(metadata.uid() as u64)),
        );
        result.insert(
            MapKey::String("gid".to_string()),
            Value::Number(DecimalNumber::from_u64(metadata.gid() as u64)),
        );
    }

    #[cfg(windows)]
    {
        // Windows: provide placeholder values for Unix-specific fields
        result.insert(
            MapKey::String("inode".to_string()),
            Value::Number(DecimalNumber::from_i64(0)),
        );
        result.insert(
            MapKey::String("mode".to_string()),
            Value::Number(DecimalNumber::from_u64(metadata.file_attributes() as u64)),
        );
        result.insert(
            MapKey::String("uid".to_string()),
            Value::Number(DecimalNumber::from_i64(0)),
        );
        result.insert(
            MapKey::String("gid".to_string()),
            Value::Number(DecimalNumber::from_i64(0)),
        );
    }

    Ok(Value::Map(result))
}
