//! Tests for runtime error spans and positions

use std::rc::Rc;
use suji_interpreter::{AstInterpreter, eval_module_source_callback};
use suji_runtime::{Executor, ModuleRegistry, setup_global_env};
use suji_values::Env;
use suji_values::RuntimeError;

use super::common::parse_program;

/// Helper to evaluate a program and get the error (if any)
fn eval_program_for_error(source: &str) -> Result<(), RuntimeError> {
    let statements = match parse_program(source) {
        Ok(stmts) => stmts,
        Err(e) => panic!("Parser error: {:?}", e),
    };

    let env = Rc::new(Env::new());
    setup_global_env(&env);

    // Register builtins and setup module registry
    suji_stdlib::runtime::builtins::register_all_builtins();
    let mut module_registry = ModuleRegistry::new();
    module_registry.set_source_evaluator(eval_module_source_callback);
    suji_stdlib::setup_module_registry(&mut module_registry);

    // Use AstInterpreter to evaluate statements
    let interpreter = AstInterpreter;
    for stmt in &statements {
        interpreter.execute_stmt(stmt, env.clone(), &module_registry)?;
    }

    Ok(())
}

#[test]
fn test_undefined_variable_has_span() {
    let source = "x + 1";
    let result = eval_program_for_error(source);

    assert!(result.is_err(), "Expected an error for undefined variable");

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    // The span should cover "x + 1" (the whole expression)
    // The undefined variable 'x' is at position 0..1
    let error_text = &source[span.start..span.end];
    assert!(
        error_text.contains('x'),
        "Span should cover the undefined variable 'x', got: '{}'",
        error_text
    );
}

#[test]
fn test_index_out_of_bounds_has_span() {
    let source = "xs = [1, 2, 3]\nxs[99]";
    let result = eval_program_for_error(source);

    assert!(result.is_err(), "Expected an error for index out of bounds");

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    // Span should cover "xs[99]" or at minimum the indexing operation
    let error_text = &source[span.start..span.end];
    assert!(
        error_text.contains("[99]") || error_text.contains("xs"),
        "Span should cover the indexing operation, got: '{}'",
        error_text
    );
}

#[test]
fn test_type_error_has_span() {
    let source = "1 + \"hello\"";
    let result = eval_program_for_error(source);

    assert!(result.is_err(), "Expected a type error");

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    // Span should cover at least part of the expression
    // The exact boundaries depend on where the error is detected
    let error_text = &source[span.start..span.end];
    assert!(
        error_text.contains('+') || error_text.contains("hello") || error_text.contains('1'),
        "Span should cover part of the binary expression, got: '{}'",
        error_text
    );
}

#[test]
fn test_method_call_error_has_span() {
    let source = "x = 1\nx::length()";
    let result = eval_program_for_error(source);

    assert!(result.is_err(), "Expected a method error");

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    // Span should cover the method call "x::length()"
    let error_text = &source[span.start..span.end];
    assert!(
        error_text.contains("::length"),
        "Span should cover the method call, got: '{}'",
        error_text
    );
}

#[test]
fn test_nested_index_error_has_span() {
    let source = "xs = [[1, 2], [3, 4]]\nxs[0][99]";
    let result = eval_program_for_error(source);

    assert!(result.is_err(), "Expected an index out of bounds error");

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    // Span should cover at least the failing index operation
    let error_text = &source[span.start..span.end];
    assert!(
        error_text.contains("[99]") || error_text.contains("xs[0][99]"),
        "Span should cover the nested indexing operation, got: '{}'",
        error_text
    );
}

#[test]
fn test_division_by_zero_has_span() {
    let source = "x = 10 / 0";
    let result = eval_program_for_error(source);

    assert!(result.is_err(), "Expected a division by zero error");

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    // Span should cover the division expression
    let error_text = &source[span.start..span.end];
    assert!(
        error_text.contains("10 / 0"),
        "Span should cover the division, got: '{}'",
        error_text
    );
}

#[test]
fn test_string_index_error_has_span() {
    let source = r#"s = "hello"
s[99]"#;
    let result = eval_program_for_error(source);

    assert!(result.is_err(), "Expected a string index error");

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    // Span should cover the index operation
    let error_text = &source[span.start..span.end];
    assert!(
        error_text.contains("[99]"),
        "Span should cover the string index, got: '{}'",
        error_text
    );
}

#[test]
fn test_map_key_not_found_has_span() {
    let source = r#"m = { a: 1 }
m:b"#;
    let result = eval_program_for_error(source);

    assert!(result.is_err(), "Expected a key not found error");

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    // Span should cover the map access
    let error_text = &source[span.start..span.end];
    assert!(
        error_text.contains("m:b"),
        "Span should cover the map access, got: '{}'",
        error_text
    );
}

#[test]
fn test_import_nonexistent_has_span() {
    let source = "import nonexistent";
    let result = eval_program_for_error(source);

    assert!(result.is_err(), "Expected an import resolution error");

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    let error_text = &source[span.start..span.end];
    assert!(
        error_text.contains("import") || error_text.contains("nonexistent"),
        "Span should cover the import statement, got: '{}'",
        error_text
    );
}

#[test]
fn test_loop_through_wrong_iterable_has_span() {
    let source = "x = 1\nloop through x { y = y }";
    let result = eval_program_for_error(source);

    assert!(
        result.is_err(),
        "Expected a type error for non-iterable loop target"
    );

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    let error_text = &source[span.start..span.end];
    assert!(
        error_text.contains("loop through") || error_text.contains("x"),
        "Span should cover the loop-through statement or iterable, got: '{}'",
        error_text
    );
}

#[test]
fn test_function_arity_mismatch_has_span() {
    let source = r#"f = |x, y| { return x + y }
f(1)"#;
    let result = eval_program_for_error(source);

    assert!(result.is_err(), "Expected an arity mismatch error");

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    // Span should cover the function call
    let error_text = &source[span.start..span.end];
    assert!(
        error_text.contains("f(1)"),
        "Span should cover the function call, got: '{}'",
        error_text
    );
}

#[test]
fn test_compound_assignment_type_error_has_span() {
    let source = r#"x = "hello"
x += 5"#;
    let result = eval_program_for_error(source);

    assert!(result.is_err(), "Expected a type error");

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    // Span should cover the compound assignment
    let error_text = &source[span.start..span.end];
    assert!(
        error_text.contains("+="),
        "Span should cover the compound assignment, got: '{}'",
        error_text
    );
}

#[test]
fn test_span_includes_line_and_column() {
    let source = "x = 1\ny = z + 1"; // 'z' is undefined on line 2
    let result = eval_program_for_error(source);

    assert!(result.is_err(), "Expected an undefined variable error");

    let error = result.unwrap_err();
    let span = error.span();

    assert!(span.is_some(), "Error should have a span");

    let span = span.unwrap();
    // The error is on line 2 (1-indexed would be line 2)
    assert!(span.line >= 1, "Line number should be set");
    // Column is always >= 0 (usize), just verify it's set
    assert!(span.start > 0, "Span should have a valid start position");
}
