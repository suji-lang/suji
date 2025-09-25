#![allow(dead_code)]

//! Common test utilities for NN Language tests.

use nnlang::ast::{Expr, Stmt};
use nnlang::lexer::Lexer;
use nnlang::parser::{ParseResult, Parser, parse_program};
use nnlang::runtime::builtins::setup_global_env;
use nnlang::runtime::env::Env;
use nnlang::runtime::eval::{
    eval_expr, eval_program_with_modules as eval_program_with_modules_impl, eval_stmt,
};
use nnlang::runtime::module::ModuleRegistry;
use nnlang::runtime::value::Value;
use std::rc::Rc;

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
    Ok(eval_expr(&expr, env)?)
}

/// Helper to evaluate a program (multiple statements)
pub fn eval_program(input: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let statements = parse_program(input)?;
    let env = create_test_env();
    let mut loop_stack = Vec::new();

    let mut result = Value::Nil;
    for stmt in statements {
        if let Some(value) = eval_stmt(&stmt, env.clone(), &mut loop_stack)? {
            result = value;
        }
    }
    Ok(result)
}

/// Helper to evaluate a program with module support
pub fn eval_program_with_modules(input: &str) -> Result<Option<Value>, Box<dyn std::error::Error>> {
    let statements = parse_program(input)?;
    let env = create_test_env();
    let module_registry = ModuleRegistry::new();
    Ok(eval_program_with_modules_impl(
        &statements,
        env,
        &module_registry,
    )?)
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
        Err(e) => assert!(
            e.to_string().contains(expected_error_fragment),
            "Error '{}' should contain '{}'",
            e,
            expected_error_fragment
        ),
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
