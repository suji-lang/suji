//! Virtual std provider - access embedded SUJI standard library sources

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;

use super::std_sources_map;

/// Resolution result for a std module path
#[derive(Debug, Clone)]
pub enum StdResolution {
    /// A single file source
    File(&'static str),
    /// A directory containing child modules
    Directory(Vec<PathBuf>),
}

/// Lazily initialized map of all std sources
static STD_SOURCES: Lazy<HashMap<PathBuf, &'static str>> =
    Lazy::new(std_sources_map::get_std_sources);

// Internal helpers for adapter/tests can iterate keys directly if needed

/// Resolve a std module path (e.g., ["io"] or ["subdir", "module"])
pub fn resolve_std_path(segments: &[&str]) -> Option<StdResolution> {
    // Empty segments means root directory - list all top-level files
    if segments.is_empty() {
        let mut children = Vec::new();
        for key in STD_SOURCES.keys() {
            let key_str = key.to_string_lossy();
            // Only top-level files (no slashes in path)
            if !key_str.contains('/') {
                children.push(key.clone());
            }
        }
        return if !children.is_empty() {
            Some(StdResolution::Directory(children))
        } else {
            None
        };
    }

    // Try file first: segments ["io"] -> "io.si"
    let file_path = PathBuf::from(segments.join("/") + ".si");
    if let Some(source) = STD_SOURCES.get(&file_path) {
        return Some(StdResolution::File(source));
    }

    // Try directory: find all keys starting with segments prefix
    let dir_prefix = segments.join("/") + "/";

    let mut children = Vec::new();
    for key in STD_SOURCES.keys() {
        let key_str = key.to_string_lossy();

        // Check if this key is under the directory prefix
        if !dir_prefix.is_empty() && !key_str.starts_with(&dir_prefix) {
            continue;
        }

        // For root directory (empty prefix), all files are children
        let relative = if dir_prefix.is_empty() {
            key_str.as_ref()
        } else if let Some(rel) = key_str.strip_prefix(&dir_prefix) {
            rel
        } else {
            continue;
        };

        // Only immediate children (no further slashes)
        if !relative.contains('/') {
            children.push(key.clone());
        }
    }

    if !children.is_empty() {
        Some(StdResolution::Directory(children))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_std_path_file() {
        // Test resolving a file path
        let result = resolve_std_path(&["io"]);
        assert!(result.is_some());
        match result.unwrap() {
            StdResolution::File(source) => {
                assert!(source.contains("io_stdin"));
            }
            _ => panic!("Expected File resolution"),
        }
    }

    #[test]
    fn test_resolve_std_path_nonexistent() {
        // Test resolving a nonexistent path
        let result = resolve_std_path(&["nonexistent"]);
        assert!(result.is_none());
    }
}
