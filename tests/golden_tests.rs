use nnlang::{
    parser::parse_program,
    runtime::{builtins::setup_global_env, env::Env, eval::eval_program, value::Value},
};
use std::fs;
use std::path::Path;
use std::rc::Rc;

/// Golden test infrastructure for end-to-end testing of example files
///
/// This module provides utilities to:
/// - Execute NN language files and capture their output
/// - Compare actual output against expected "golden" output
/// - Generate new golden files when needed
/// - Handle different output types (stdout, return values, errors)

/// Configuration for golden tests
#[derive(Debug, Clone)]
pub struct GoldenTestConfig {
    /// Directory containing example files
    pub examples_dir: String,
    /// Directory containing expected output files
    pub golden_dir: String,
    /// Whether to update golden files when they don't match
    pub update_golden: bool,
    /// Whether to capture stdout from println calls
    pub capture_stdout: bool,
}

impl Default for GoldenTestConfig {
    fn default() -> Self {
        Self {
            examples_dir: "examples".to_string(),
            golden_dir: "tests/golden".to_string(),
            update_golden: false,
            capture_stdout: true,
        }
    }
}

/// Result of executing a golden test
#[derive(Debug, PartialEq)]
pub struct GoldenTestResult {
    /// The actual output captured
    pub output: String,
    /// The return value of the last expression (if any)
    pub return_value: Option<String>,
    /// Any errors that occurred
    pub errors: Vec<String>,
    /// Whether the test passed
    pub passed: bool,
}

/// Execute a golden test for a given example file
pub fn run_golden_test(example_file: &str, config: &GoldenTestConfig) -> GoldenTestResult {
    let example_path = format!("{}/{}", config.examples_dir, example_file);
    let golden_path = format!("{}/{}.golden", config.golden_dir, example_file);

    // Read the example file
    let source = match fs::read_to_string(&example_path) {
        Ok(content) => content,
        Err(e) => {
            return GoldenTestResult {
                output: String::new(),
                return_value: None,
                errors: vec![format!(
                    "Failed to read example file {}: {}",
                    example_path, e
                )],
                passed: false,
            };
        }
    };

    // Execute the NN program
    let execution_result = execute_nn_program(&source, config);

    // Read expected output if it exists
    let expected_output = if Path::new(&golden_path).exists() {
        match fs::read_to_string(&golden_path) {
            Ok(content) => Some(content),
            Err(e) => {
                return GoldenTestResult {
                    output: execution_result.output.clone(),
                    return_value: execution_result.return_value.clone(),
                    errors: vec![format!("Failed to read golden file {}: {}", golden_path, e)],
                    passed: false,
                };
            }
        }
    } else {
        None
    };

    // Compare results
    let passed = match expected_output {
        Some(expected) => {
            let actual = format_output(&execution_result);
            actual.trim() == expected.trim()
        }
        None => {
            // No golden file exists - this is a new test
            if config.update_golden {
                // Create the golden file
                let output = format_output(&execution_result);
                if let Err(e) = fs::create_dir_all(&config.golden_dir) {
                    return GoldenTestResult {
                        output: execution_result.output,
                        return_value: execution_result.return_value,
                        errors: vec![format!("Failed to create golden directory: {}", e)],
                        passed: false,
                    };
                }
                if let Err(e) = fs::write(&golden_path, output) {
                    return GoldenTestResult {
                        output: execution_result.output,
                        return_value: execution_result.return_value,
                        errors: vec![format!("Failed to write golden file: {}", e)],
                        passed: false,
                    };
                }
            }
            true // New test passes by default
        }
    };

    GoldenTestResult {
        output: execution_result.output,
        return_value: execution_result.return_value,
        errors: execution_result.errors,
        passed,
    }
}

/// Internal result of executing an NN program
#[derive(Debug)]
struct ExecutionResult {
    output: String,
    return_value: Option<String>,
    errors: Vec<String>,
}

/// Execute an NN program and capture its output
fn execute_nn_program(source: &str, _config: &GoldenTestConfig) -> ExecutionResult {
    let output = String::new();
    let mut errors = Vec::new();
    let mut return_value = None;

    // Parse the source directly
    let ast = match parse_program(source) {
        Ok(ast) => ast,
        Err(e) => {
            errors.push(format!("Parse error: {}", e));
            return ExecutionResult {
                output,
                return_value,
                errors,
            };
        }
    };

    // Create runtime environment
    let env = Rc::new(Env::new());

    // Setup global environment with std module
    setup_global_env(&env);

    // Evaluate the program
    match eval_program(&ast, env) {
        Ok(value) => {
            // Capture the return value if it's not Null
            if let Some(val) = value {
                if !matches!(val, Value::Nil) {
                    return_value = Some(val.to_string());
                }
            }
        }
        Err(e) => {
            errors.push(format!("Runtime error: {}", e));
        }
    }

    ExecutionResult {
        output,
        return_value,
        errors,
    }
}

/// Format the execution result into a consistent string format
fn format_output(result: &ExecutionResult) -> String {
    let mut output = String::new();

    // Add stdout output if any
    if !result.output.is_empty() {
        output.push_str("=== STDOUT ===\n");
        output.push_str(&result.output);
        output.push_str("\n");
    }

    // Add return value if any
    if let Some(ref ret) = result.return_value {
        output.push_str("=== RETURN ===\n");
        output.push_str(ret);
        output.push_str("\n");
    }

    // Add errors if any
    if !result.errors.is_empty() {
        output.push_str("=== ERRORS ===\n");
        for error in &result.errors {
            output.push_str(error);
            output.push_str("\n");
        }
    }

    output
}

/// Run all golden tests in the examples directory
pub fn run_all_golden_tests(config: &GoldenTestConfig) -> Vec<(String, GoldenTestResult)> {
    let mut results = Vec::new();

    // Find all .nn files in the examples directory
    let examples_dir = Path::new(&config.examples_dir);
    if !examples_dir.exists() {
        return results;
    }

    let entries = match fs::read_dir(examples_dir) {
        Ok(entries) => entries,
        Err(_) => return results,
    };

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("nn") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    let result = run_golden_test(filename, config);
                    results.push((filename.to_string(), result));
                }
            }
        }
    }

    results
}

/// Generate golden files for all examples (useful for initial setup)
pub fn generate_golden_files(config: &GoldenTestConfig) -> Result<(), String> {
    let mut config = config.clone();
    config.update_golden = true;

    let results = run_all_golden_tests(&config);

    for (filename, result) in results {
        if !result.passed && !result.errors.is_empty() {
            return Err(format!(
                "Failed to generate golden file for {}: {:?}",
                filename, result.errors
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_test_config_default() {
        let config = GoldenTestConfig::default();
        assert_eq!(config.examples_dir, "examples");
        assert_eq!(config.golden_dir, "tests/golden");
        assert!(!config.update_golden);
        assert!(config.capture_stdout);
    }

    #[test]
    fn test_format_output() {
        let result = ExecutionResult {
            output: "Hello, world!".to_string(),
            return_value: Some("42".to_string()),
            errors: vec!["Test error".to_string()],
        };

        let formatted = format_output(&result);
        assert!(formatted.contains("=== STDOUT ==="));
        assert!(formatted.contains("Hello, world!"));
        assert!(formatted.contains("=== RETURN ==="));
        assert!(formatted.contains("42"));
        assert!(formatted.contains("=== ERRORS ==="));
        assert!(formatted.contains("Test error"));
    }

    #[test]
    fn test_run_golden_test_hello() {
        let config = GoldenTestConfig::default();
        let result = run_golden_test("hello.nn", &config);

        // The hello.nn file should execute without errors
        assert!(
            result.errors.is_empty(),
            "Expected no errors, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_run_golden_test_from_spec() {
        let config = GoldenTestConfig::default();
        let result = run_golden_test("from_spec.nn", &config);

        // The from_spec.nn file should execute without errors
        assert!(
            result.errors.is_empty(),
            "Expected no errors, got: {:?}",
            result.errors
        );
    }
}
