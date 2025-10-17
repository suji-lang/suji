//! Adapter to connect virtual_std to suji-runtime's ModuleRegistry

use super::virtual_std::{self, StdResolution};
use suji_runtime::module::VirtualStdResult;

/// Resolver function that adapts virtual_std API to ModuleRegistry's callback type
pub fn virtual_std_resolver(segments: &[&str]) -> Option<VirtualStdResult> {
    match virtual_std::resolve_std_path(segments) {
        Some(StdResolution::File(source)) => Some(VirtualStdResult::File(source)),
        Some(StdResolution::Directory(children)) => {
            // Extract file basenames (without .si extension) for the module map
            let names: Vec<String> = children
                .iter()
                .filter_map(|path| {
                    path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .map(|s| s.to_string())
                })
                .collect();
            Some(VirtualStdResult::Directory(names))
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_std_resolver_file() {
        // Test resolving a file
        let result = virtual_std_resolver(&["io"]);
        assert!(result.is_some());
        match result.unwrap() {
            VirtualStdResult::File(source) => {
                assert!(source.contains("io_stdin"));
            }
            _ => panic!("Expected File result"),
        }
    }

    #[test]
    fn test_virtual_std_resolver_nonexistent() {
        // Test resolving a nonexistent path
        let result = virtual_std_resolver(&["nonexistent"]);
        assert!(result.is_none());
    }
}
