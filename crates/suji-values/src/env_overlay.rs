//! Environment overlay for variables used by the runtime and shell.

use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::{OnceLock, RwLock};

use super::value::RuntimeError;

/// Proxy for environment variable operations (map-like interface).
#[derive(Debug)]
pub struct EnvProxy;

impl Default for EnvProxy {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvProxy {
    /// Create a new environment proxy
    pub fn new() -> Self {
        Self
    }

    /// Get an environment variable value (overlay takes precedence)
    pub fn get(&self, key: &str) -> Option<String> {
        get_effective_env_var(key)
    }

    /// Set an environment variable
    pub fn set(&self, key: &str, value: &str) -> Result<(), RuntimeError> {
        if key.is_empty() {
            return Err(RuntimeError::InvalidOperation {
                message: "Environment variable key cannot be empty".to_string(),
            });
        }
        // Validate key/value to avoid platform panics (e.g., NULs or '=' in key)
        if key.contains('=') {
            return Err(RuntimeError::InvalidOperation {
                message: "Environment variable key cannot contain '='".to_string(),
            });
        }
        if key.as_bytes().contains(&0) {
            return Err(RuntimeError::InvalidOperation {
                message: "Environment variable key cannot contain NUL (\\0)".to_string(),
            });
        }
        if value.as_bytes().contains(&0) {
            return Err(RuntimeError::InvalidOperation {
                message: "Environment variable value cannot contain NUL (\\0)".to_string(),
            });
        }
        // Record in overlay instead of mutating the process environment
        let overlay = env_overlay();
        let mut map = overlay
            .write()
            .map_err(|_| RuntimeError::InvalidOperation {
                message: "Environment overlay lock poisoned".to_string(),
            })?;
        map.insert(key.to_string(), Some(value.to_string()));
        Ok(())
    }

    /// Delete an environment variable
    pub fn delete(&self, key: &str) -> bool {
        let existed = get_effective_env_var(key).is_some();
        let overlay = env_overlay();
        if let Ok(mut map) = overlay.write() {
            map.insert(key.to_string(), None);
        }
        existed
    }

    /// Check if an environment variable exists
    pub fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Get all environment variable keys
    pub fn keys(&self) -> Vec<String> {
        let mut keys: HashSet<String> = std::env::vars().map(|(k, _)| k).collect();
        let overlay = env_overlay();
        if let Ok(map) = overlay.read() {
            for (k, v) in map.iter() {
                if v.is_some() {
                    keys.insert(k.clone());
                } else {
                    keys.remove(k);
                }
            }
        }
        keys.into_iter().collect()
    }

    /// Get all environment variable values
    pub fn values(&self) -> Vec<String> {
        self.to_list().into_iter().map(|(_, v)| v).collect()
    }

    /// Get all environment variables as key-value pairs
    pub fn to_list(&self) -> Vec<(String, String)> {
        let mut map: HashMap<String, String> = std::env::vars().collect();
        let overlay = env_overlay();
        if let Ok(ov) = overlay.read() {
            for (k, v) in ov.iter() {
                match v {
                    Some(val) => {
                        map.insert(k.clone(), val.clone());
                    }
                    None => {
                        map.remove(k);
                    }
                }
            }
        }
        map.into_iter().collect()
    }

    /// Get the number of environment variables
    pub fn length(&self) -> usize {
        self.keys().len()
    }
}

// Global environment overlay (safe, no unsafe)
static ENV_OVERLAY: OnceLock<RwLock<HashMap<String, Option<String>>>> = OnceLock::new();

/// Get the global environment overlay
fn env_overlay() -> &'static RwLock<HashMap<String, Option<String>>> {
    ENV_OVERLAY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Get effective variable value (overlay first, then process env)
pub fn get_effective_env_var(key: &str) -> Option<String> {
    if let Ok(overlay) = env_overlay().read()
        && let Some(entry) = overlay.get(key)
    {
        return entry.clone();
    }
    std::env::var(key).ok()
}

/// Apply overlay to a Command (set/remove variables)
pub fn apply_env_overlay_to_command(cmd: &mut Command) -> Result<(), RuntimeError> {
    let overlay = env_overlay()
        .read()
        .map_err(|_| RuntimeError::InvalidOperation {
            message: "Environment overlay lock poisoned".to_string(),
        })?;

    for (k, v) in overlay.iter() {
        match v {
            Some(val) => {
                cmd.env(k, val);
            }
            None => {
                cmd.env_remove(k);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_proxy_basic_operations() {
        let env = EnvProxy::new();

        // Test set and get
        env.set("TEST_VAR", "test_value").unwrap();
        assert_eq!(env.get("TEST_VAR"), Some("test_value".to_string()));

        // Test contains
        assert!(env.contains("TEST_VAR"));
        assert!(!env.contains("NONEXISTENT_VAR"));

        // Test delete
        assert!(env.delete("TEST_VAR"));
        assert_eq!(env.get("TEST_VAR"), None);
        assert!(!env.delete("TEST_VAR")); // Should return false for non-existent
    }

    #[test]
    fn test_env_proxy_validation() {
        let env = EnvProxy::new();

        // Test empty key
        assert!(env.set("", "value").is_err());

        // Test key with equals
        assert!(env.set("KEY=BAD", "value").is_err());

        // Test key with NUL
        assert!(env.set("KEY\0BAD", "value").is_err());

        // Test value with NUL
        assert!(env.set("KEY", "value\0bad").is_err());
    }

    #[test]
    fn test_overlay_precedence() {
        let env = EnvProxy::new();

        // Set a process env var (if it doesn't exist)
        let test_key = "NNLANG_TEST_OVERLAY";
        unsafe {
            std::env::set_var(test_key, "process_value");
        }

        // Overlay should take precedence
        env.set(test_key, "overlay_value").unwrap();
        assert_eq!(
            get_effective_env_var(test_key),
            Some("overlay_value".to_string())
        );

        // Clean up
        env.delete(test_key);
        unsafe {
            std::env::remove_var(test_key);
        }
    }

    #[test]
    fn test_keys_and_values() {
        let env = EnvProxy::new();

        env.set("TEST_KEY1", "value1").unwrap();
        env.set("TEST_KEY2", "value2").unwrap();

        let keys = env.keys();
        assert!(keys.contains(&"TEST_KEY1".to_string()));
        assert!(keys.contains(&"TEST_KEY2".to_string()));

        let values = env.values();
        assert!(values.contains(&"value1".to_string()));
        assert!(values.contains(&"value2".to_string()));

        let list = env.to_list();
        assert!(list.contains(&("TEST_KEY1".to_string(), "value1".to_string())));
        assert!(list.contains(&("TEST_KEY2".to_string(), "value2".to_string())));

        // Clean up
        env.delete("TEST_KEY1");
        env.delete("TEST_KEY2");
    }

    #[test]
    fn test_apply_overlay_to_command() {
        let env = EnvProxy::new();
        env.set("TEST_CMD_VAR", "cmd_value").unwrap();

        let mut cmd = Command::new("echo");
        apply_env_overlay_to_command(&mut cmd).unwrap();

        // We can't easily test the actual environment application without running the command,
        // but we can verify the function doesn't panic or error

        // Clean up
        env.delete("TEST_CMD_VAR");
    }
}
