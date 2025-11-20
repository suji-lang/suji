#![allow(dead_code)]

//! Common test utilities for SUJI Language tests.

use std::rc::Rc;
use suji_ast::{Expr, Stmt};
use suji_interpreter::{AstInterpreter, eval_module_source_callback};
use suji_lexer::Lexer;
use suji_parser::{ParseResult, Parser};
use suji_runtime::{Executor, ModuleRegistry, setup_global_env};
use suji_values::Env;
use suji_values::Value;

/// Parse a single expression from source code (test utility)
pub fn parse_expression(input: &str) -> ParseResult<Expr> {
    let tokens = Lexer::lex(input)?;
    let mut parser = Parser::new(tokens);
    parser.expression()
}

/// Parse a single statement (test utility)
pub fn parse_statement(input: &str) -> ParseResult<Stmt> {
    let tokens = Lexer::lex(input)?;
    let mut parser = Parser::new(tokens);
    parser.statement()
}

/// Parse a complete program (test utility)
pub fn parse_program(input: &str) -> ParseResult<Vec<Stmt>> {
    let tokens = Lexer::lex(input)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

/// Helper to create a test environment with built-ins
pub fn create_test_env() -> Rc<Env> {
    let env = Rc::new(Env::new());
    setup_global_env(&env);
    env
}

/// Helper to evaluate a string expression
pub fn eval_string_expr(input: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let expr = parse_expression(input)?;
    let env = create_test_env();

    // Create interpreter and module registry
    let interpreter = AstInterpreter;
    let mut module_registry = ModuleRegistry::new();
    module_registry.set_source_evaluator(eval_module_source_callback);
    suji_stdlib::setup_module_registry(&mut module_registry);

    Ok(interpreter.execute_expr(&expr, env, &module_registry)?)
}

/// Helper to evaluate a program (multiple statements)
pub fn eval_program(input: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let statements = parse_program(input)?;
    let env = create_test_env();

    // Register builtins BEFORE creating the module registry
    // so that __builtins__ module is populated correctly
    suji_stdlib::runtime::builtins::register_all_builtins();
    let mut module_registry = ModuleRegistry::new();
    module_registry.set_source_evaluator(eval_module_source_callback);
    suji_stdlib::setup_module_registry(&mut module_registry);

    // Use AstInterpreter to evaluate statements
    let interpreter = AstInterpreter;
    let mut last_value = None;
    for stmt in &statements {
        match interpreter.execute_stmt(stmt, env.clone(), &module_registry) {
            Ok(Some(v)) => last_value = Some(v),
            Ok(None) => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(last_value.unwrap_or(Value::Nil))
}

/// Helper to evaluate a program with module support
pub fn eval_program_with_modules(input: &str) -> Result<Option<Value>, Box<dyn std::error::Error>> {
    let statements = parse_program(input)?;
    let env = create_test_env();

    // Register builtins BEFORE creating the module registry
    // so that __builtins__ module is populated correctly
    suji_stdlib::runtime::builtins::register_all_builtins();
    let mut module_registry = ModuleRegistry::new();
    module_registry.set_source_evaluator(eval_module_source_callback);
    suji_stdlib::setup_module_registry(&mut module_registry);

    // Use AstInterpreter to evaluate statements
    let interpreter = AstInterpreter;
    let mut last_value = None;
    for stmt in &statements {
        match interpreter.execute_stmt(stmt, env.clone(), &module_registry) {
            Ok(Some(v)) => last_value = Some(v),
            Ok(None) => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(last_value)
}

/// Try to evaluate, returning true if successful, false if failed
pub fn can_eval(input: &str) -> bool {
    eval_string_expr(input).is_ok()
}

/// Try to evaluate a program, returning true if successful
pub fn can_eval_program(input: &str) -> bool {
    eval_program(input).is_ok()
}

/// Test that import evaluation succeeds
pub fn assert_import_works(input: &str) {
    match eval_program_with_modules(input) {
        Ok(_) => {}
        Err(e) => panic!("Import test failed: {}", e),
    }
}

/// Test that parsing fails with expected error
pub fn assert_parse_fails(input: &str, expected_error_fragment: &str) {
    match parse_program(input) {
        Ok(_) => panic!("Expected parsing to fail for: {}", input),
        Err(e) => {
            let msg = e.to_string();
            // Support multiple acceptable fragments separated by '||' to ease message transitions
            let options: Vec<&str> = expected_error_fragment.split("||").collect();
            // Support general matching (substring) or exact error enum message text
            let matched = options
                .iter()
                .map(|frag| frag.trim())
                .any(|frag| msg.contains(frag) || msg == frag);
            assert!(
                matched,
                "Error '{}' should contain one of '{}'",
                msg, expected_error_fragment
            );
        }
    }
}

/// Test that evaluation fails with expected error
pub fn assert_eval_fails(input: &str, expected_error_fragment: &str) {
    match eval_program_with_modules(input) {
        Ok(_) => panic!("Expected evaluation to fail for: {}", input),
        Err(e) => assert!(
            e.to_string().contains(expected_error_fragment),
            "Error '{}' should contain '{}'",
            e,
            expected_error_fragment
        ),
    }
}
