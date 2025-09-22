use super::env_overlay::apply_env_overlay_to_command;
use super::value::{RuntimeError, Value};
use std::process::Command;

/// Execute a shell command and return stdout as UTF-8 (trims trailing newline)
pub fn run_shell(command: &str) -> Result<String, RuntimeError> {
    // Determine shell to use
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    // Execute command using shell
    let mut cmd = Command::new(&shell);
    apply_env_overlay_to_command(&mut cmd)?;
    let output = cmd
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|err| RuntimeError::ShellError {
            message: format!("Failed to execute shell command '{}': {}", command, err),
        })?;

    // Check exit status
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let exit_code = output.status.code().unwrap_or(-1);
        return Err(RuntimeError::ShellError {
            message: format!(
                "Shell command '{}' failed with exit code {}: {}",
                command,
                exit_code,
                stderr.trim()
            ),
        });
    }

    // Convert stdout to UTF-8 string
    let stdout = String::from_utf8(output.stdout).map_err(|err| RuntimeError::ShellError {
        message: format!("Shell command output is not valid UTF-8: {}", err),
    })?;

    // Trim trailing newline (common in shell commands)
    let result = if stdout.ends_with('\n') {
        stdout.strip_suffix('\n').unwrap_or(&stdout).to_string()
    } else {
        stdout
    };

    Ok(result)
}

/// Execute a shell command template and return Value::String
pub fn execute_shell_template(command: &str) -> Result<Value, RuntimeError> {
    let output = run_shell(command)?;
    Ok(Value::String(output))
}

/// Basic safety check for commands (non-empty)
pub fn is_safe_command(command: &str) -> bool {
    // Reject empty commands
    if command.trim().is_empty() {
        return false;
    }

    // For now, we allow all non-empty commands
    // In production, you might want to:
    // - Blacklist dangerous commands (rm -rf, etc.)
    // - Whitelist only specific commands
    // - Sandbox execution
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_shell_command() {
        let result = run_shell("echo hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_basic_echo_commands() {
        // Test echo with quoted output
        let result = run_shell("echo 'Hello, World!'");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");

        // Test echo with numbers
        let result = run_shell("echo 42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");

        // Test echo with spaces
        let result = run_shell("echo hello world");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");
    }

    #[test]
    fn test_multiline_output() {
        let result = run_shell("printf 'line1\\nline2\\nline3'");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "line1\nline2\nline3");
    }

    #[test]
    fn test_failing_command() {
        let result = run_shell("false"); // Command that always fails
        assert!(matches!(result, Err(RuntimeError::ShellError { .. })));
    }

    #[test]
    fn test_nonexistent_command() {
        let result = run_shell("nonexistent_command_xyz_123");
        assert!(matches!(result, Err(RuntimeError::ShellError { .. })));
    }

    #[test]
    fn test_execute_shell_template() {
        let result = execute_shell_template("echo test");
        assert!(result.is_ok());
        if let Value::String(s) = result.unwrap() {
            assert_eq!(s, "test");
        } else {
            panic!("Expected string value");
        }
    }

    #[test]
    fn test_empty_output() {
        let result = run_shell("echo -n"); // echo without newline
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_command_with_quotes() {
        let result = run_shell(r#"echo "quoted string""#);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "quoted string");
    }

    #[test]
    fn test_is_safe_command() {
        assert!(is_safe_command("echo hello"));
        assert!(is_safe_command("ls -la"));
        assert!(!is_safe_command(""));
        assert!(!is_safe_command("   "));
    }

    #[test]
    fn test_trailing_newline_removal() {
        // Most shell commands add a trailing newline
        let result = run_shell("printf 'hello\\n'");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");

        // Commands without trailing newline should be preserved
        let result = run_shell("printf 'hello'");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_command_with_pipes() {
        let result = run_shell("echo 'hello world' | grep hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");
    }

    #[test]
    fn test_command_with_variables() {
        let result = run_shell("VAR=test; echo $VAR");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_unicode_output() {
        let result = run_shell("echo 'café 🚀'");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "café 🚀");
    }
}
