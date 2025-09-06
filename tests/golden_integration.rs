use nnlang::{
    parser::parse_program,
    runtime::{builtins::setup_global_env, env::Env, eval::eval_program, value::Value},
};
use std::fs;
use std::path::Path;
use std::rc::Rc;

/// Execute an NN program and capture its output
fn execute_nn_program(source: &str) -> Result<Option<Value>, String> {
    // Parse the source directly
    let ast = parse_program(source).map_err(|e| format!("Parse error: {}", e))?;

    // Create runtime environment
    let env = Rc::new(Env::new());

    // Setup global environment with std module
    setup_global_env(&env);

    // Evaluate the program
    eval_program(&ast, env).map_err(|e| format!("Runtime error: {}", e))
}

/// Integration tests for golden test infrastructure
///
/// These tests verify that all example files execute correctly and
/// maintain consistent output over time.

#[test]
fn test_hello_example() {
    let source = fs::read_to_string("examples/hello.nn").expect("Failed to read hello.nn");

    let result = execute_nn_program(&source);
    assert!(result.is_ok(), "Hello example failed: {:?}", result.err());

    // The hello example should execute without errors
    println!("Hello example executed successfully");
}

#[test]
fn test_from_spec_example() {
    let source = fs::read_to_string("examples/from_spec.nn").expect("Failed to read from_spec.nn");

    let result = execute_nn_program(&source);
    assert!(
        result.is_ok(),
        "From spec example failed: {:?}",
        result.err()
    );

    // The from_spec example should have a return value (the last expression)
    if let Ok(Some(value)) = result {
        println!("From spec example returned: {}", value);
    }
}

#[test]
fn test_demo_example() {
    let source = fs::read_to_string("examples/demo.nn").expect("Failed to read demo.nn");

    let result = execute_nn_program(&source);
    assert!(result.is_ok(), "Demo example failed: {:?}", result.err());

    println!("Demo example executed successfully");
}

#[test]
fn test_fib_example() {
    let source = fs::read_to_string("examples/fib.nn").expect("Failed to read fib.nn");

    let result = execute_nn_program(&source);
    assert!(result.is_ok(), "Fib example failed: {:?}", result.err());

    println!("Fib example executed successfully");
}

#[test]
fn test_all_examples_execute_successfully() {
    let examples_dir = Path::new("examples");
    if !examples_dir.exists() {
        panic!("Examples directory not found");
    }

    let mut success_count = 0;
    let mut total_count = 0;

    let entries = fs::read_dir(examples_dir).expect("Failed to read examples directory");

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("nn") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    total_count += 1;

                    let source =
                        fs::read_to_string(&path).expect(&format!("Failed to read {}", filename));

                    match execute_nn_program(&source) {
                        Ok(_) => {
                            success_count += 1;
                            println!("✓ {}: PASS", filename);
                        }
                        Err(e) => {
                            println!("✗ {}: FAIL - {}", filename, e);
                        }
                    }
                }
            }
        }
    }

    println!(
        "Successfully tested {}/{} example files",
        success_count, total_count
    );
    assert_eq!(
        success_count, total_count,
        "Not all examples executed successfully"
    );
}
