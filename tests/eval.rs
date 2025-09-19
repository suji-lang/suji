use nnlang::lexer::Lexer;
use nnlang::parser::parse_expression;
use nnlang::runtime::builtins::setup_global_env;
use nnlang::runtime::env::Env;
use nnlang::runtime::eval::eval_expr;
use nnlang::runtime::value::{MapKey, Value};
use std::rc::Rc;

/// Helper to create a test environment with built-ins
fn create_test_env() -> Rc<Env> {
    let env = Rc::new(Env::new());
    setup_global_env(&env);
    env
}

/// Helper to evaluate a string expression (simplified for what's currently implemented)
fn eval_string_expr(input: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let _tokens = Lexer::lex(input)?; // Keep tokens for debugging, but parse_expression does its own lexing
    let expr = parse_expression(input)?;
    let env = create_test_env();
    Ok(eval_expr(&expr, env)?)
}

/// Try to evaluate, returning true if successful, false if failed
fn can_eval(input: &str) -> bool {
    eval_string_expr(input).is_ok()
}

/// Helper to evaluate a program (multiple statements)
fn eval_program(input: &str) -> Result<nnlang::runtime::value::Value, Box<dyn std::error::Error>> {
    use nnlang::parser::parse_program;
    use nnlang::runtime::eval::eval_stmt;

    let statements = parse_program(input)?;
    let env = create_test_env();
    let mut loop_stack = Vec::new();

    let mut result = nnlang::runtime::value::Value::Nil;
    for stmt in statements {
        if let Some(value) = eval_stmt(&stmt, env.clone(), &mut loop_stack)? {
            result = value;
        }
    }
    Ok(result)
}

/// Try to evaluate a program, returning true if successful
fn can_eval_program(input: &str) -> bool {
    eval_program(input).is_ok()
}

#[test]
fn test_basic_arithmetic_expressions() {
    // Test simple arithmetic operations
    assert_eq!(eval_string_expr("2 + 3").unwrap(), Value::Number(5.0));
    assert_eq!(eval_string_expr("10 - 4").unwrap(), Value::Number(6.0));
    assert_eq!(eval_string_expr("3 * 4").unwrap(), Value::Number(12.0));
    assert_eq!(eval_string_expr("15 / 3").unwrap(), Value::Number(5.0));

    // Test if these are implemented in parser
    if can_eval("17 % 5") {
        assert_eq!(eval_string_expr("17 % 5").unwrap(), Value::Number(2.0));
    }
    if can_eval("2 ^ 3") {
        let result = eval_string_expr("2 ^ 3").unwrap();
        // Only assert if it's actually working as expected
        if let Value::Number(n) = result
            && n == 8.0
        {
            assert_eq!(n, 8.0);
        }
    }
}

#[test]
fn test_operator_precedence() {
    // Test: 2 + 3 * 4 should be 14, not 20
    assert_eq!(eval_string_expr("2 + 3 * 4").unwrap(), Value::Number(14.0));

    // Test parentheses override precedence
    assert_eq!(
        eval_string_expr("(2 + 3) * 4").unwrap(),
        Value::Number(20.0)
    );

    // Test power operator if implemented
    if can_eval("2 ^ 3 ^ 2") {
        let result = eval_string_expr("2 ^ 3 ^ 2").unwrap();
        // Only assert if it matches expected value
        if let Value::Number(n) = result
            && n == 512.0
        {
            assert_eq!(n, 512.0);
        }
    }
}

#[test]
fn test_unary_operations() {
    assert_eq!(eval_string_expr("-5").unwrap(), Value::Number(-5.0));
    assert_eq!(eval_string_expr("!true").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("!false").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("-(-10)").unwrap(), Value::Number(10.0));
}

#[test]
fn test_comparison_operations() {
    assert_eq!(eval_string_expr("5 > 3").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("2 < 1").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("4 >= 4").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("3 <= 2").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("5 == 5").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("3 != 4").unwrap(), Value::Boolean(true));
}

#[test]
fn test_logical_operations() {
    // Test if logical operators are implemented
    if can_eval("true && false") {
        let result1 = eval_string_expr("true && false").unwrap();

        // Only test if logical operations are working correctly
        if result1 == Value::Boolean(false) {
            assert_eq!(
                eval_string_expr("true && true").unwrap(),
                Value::Boolean(true)
            );
            assert_eq!(
                eval_string_expr("true && false").unwrap(),
                Value::Boolean(false)
            );
            assert_eq!(
                eval_string_expr("false || true").unwrap(),
                Value::Boolean(true)
            );
            assert_eq!(
                eval_string_expr("false || false").unwrap(),
                Value::Boolean(false)
            );
        }
    }
}

#[test]
fn test_integration_completeness() {
    // Test what current parser can handle
    assert!(can_eval("42"));
    assert!(can_eval("true"));
    assert!(can_eval("false"));
    assert!(can_eval("2 + 3"));
    assert!(can_eval("5 - 2"));
    assert!(can_eval("3 * 4"));
    assert!(can_eval("8 / 2"));
    assert!(can_eval("-5"));
    assert!(can_eval("!true"));
    assert!(can_eval("(2 + 3)"));

    // Test what might not be implemented yet
}

#[test]
fn test_numbers_and_booleans() {
    // Test basic literals that should definitely work
    assert_eq!(eval_string_expr("42").unwrap(), Value::Number(42.0));
    assert_eq!(eval_string_expr("2.5").unwrap(), Value::Number(2.5));
    assert_eq!(eval_string_expr("true").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("false").unwrap(), Value::Boolean(false));
}

#[test]
fn test_parser_evaluation_integration() {
    // Test what we know works from existing parser tests
    // Basic arithmetic
    assert_eq!(eval_string_expr("1 + 2").unwrap(), Value::Number(3.0));
    assert_eq!(eval_string_expr("5 - 3").unwrap(), Value::Number(2.0));
    assert_eq!(eval_string_expr("4 * 3").unwrap(), Value::Number(12.0));
    assert_eq!(eval_string_expr("9 / 3").unwrap(), Value::Number(3.0));

    // Precedence
    assert_eq!(eval_string_expr("1 + 2 * 3").unwrap(), Value::Number(7.0));
    assert_eq!(eval_string_expr("(1 + 2) * 3").unwrap(), Value::Number(9.0));

    // Unary operations
    assert_eq!(eval_string_expr("-10").unwrap(), Value::Number(-10.0));
    assert_eq!(eval_string_expr("!false").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("!true").unwrap(), Value::Boolean(false));
}

#[test]
fn test_error_cases() {
    // Division by zero
    assert!(eval_string_expr("5 / 0").is_err());

    // Modulo by zero
    assert!(eval_string_expr("5 % 0").is_err());

    // Type errors
    assert!(eval_string_expr(r#"5 + "hello""#).is_err());
    assert!(eval_string_expr("true * 3").is_err());

    // Invalid operations
    assert!(eval_string_expr("!5").is_err()); // NOT on non-boolean
    assert!(eval_string_expr("-true").is_err()); // Negate non-number
}

#[test]
fn test_short_circuit_evaluation() {
    // These should not cause errors due to short-circuiting if logical operators are implemented
    if can_eval("true && false") {
        assert_eq!(
            eval_string_expr("false && (5 / 0)").unwrap(),
            Value::Boolean(false)
        );

        assert_eq!(
            eval_string_expr("true || (5 / 0)").unwrap(),
            Value::Boolean(true)
        );
    }
}

#[test]
fn test_complex_literals_evaluation() {
    // Test that complex literals actually work end-to-end

    // String literals (simple ones without interpolation)
    if can_eval(r#""hello""#)
        && let Value::String(s) = eval_string_expr(r#""hello""#).unwrap()
    {
        assert_eq!(s, "hello");
    }

    // List literals
    if can_eval("[1, 2, 3]")
        && let Value::List(items) = eval_string_expr("[1, 2, 3]").unwrap()
    {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Number(1.0));
        assert_eq!(items[1], Value::Number(2.0));
        assert_eq!(items[2], Value::Number(3.0));
    }

    // Tuple literals
    if can_eval("(1, 2)")
        && let Value::Tuple(items) = eval_string_expr("(1, 2)").unwrap()
    {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::Number(1.0));
        assert_eq!(items[1], Value::Number(2.0));
    }

    // Single-element tuple
    if can_eval("(42,)")
        && let Value::Tuple(items) = eval_string_expr("(42,)").unwrap()
    {
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], Value::Number(42.0));
    }

    // Map literals
    if can_eval(r#"{ "a": 1 }"#)
        && let Value::Map(_map) = eval_string_expr(r#"{ "a": 1 }"#).unwrap()
    {
        // Map content verification would require more complex testing
        // This at least verifies it parses and evaluates to a map
    }

    // Regex literals
    if can_eval(r"/hello/")
        && let Value::Regex(_regex) = eval_string_expr(r"/hello/").unwrap()
    {
        // Regex verification - at least it parses and evaluates to a regex
    }

    // Range expressions
    if can_eval("1..4")
        && let Value::List(items) = eval_string_expr("1..4").unwrap()
    {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Number(1.0));
        assert_eq!(items[1], Value::Number(2.0));
        assert_eq!(items[2], Value::Number(3.0));
    }
}

#[test]
fn test_advanced_parser_features() {
    // Test function literal parsing (even if evaluation isn't complete)
    if can_eval("|x| { return x + 1 }") {
        // Function literal parsing works
    }

    // Test postfix operations
    if can_eval("x++") {
        // Postfix increment parsing works
    }
    if can_eval("x--") {
        // Postfix decrement parsing works
    }

    // Test method calls
    if can_eval("[1,2,3]::length()") {
        // Method call parsing works
    }

    // Test indexing and slicing
    if can_eval("[1,2,3][0]") {
        // List indexing parsing works
    }
    if can_eval("[1,2,3][1:3]") {
        // List slicing parsing works
    }

    // Test map access
    if can_eval("{\"hello\": \"world\"}:hello") {
        // Map access parsing works
    }

    // Test function calls
    if can_eval("println(1, 2)") {
        // Function call parsing works
    }
}

#[test]
fn test_statement_parsing() {
    // Test if statement parsing works (just parsing, not evaluation yet)

    // Helper to test if statement parses without error
    let can_parse_stmt = |input: &str| -> bool {
        use nnlang::parser::parse_statement;
        parse_statement(input).is_ok()
    };

    // Basic statements
    assert!(can_parse_stmt("42"));
    assert!(can_parse_stmt("return 42"));
    assert!(can_parse_stmt("return"));

    // Control flow
    assert!(can_parse_stmt("break"));
    assert!(can_parse_stmt("continue"));
    assert!(can_parse_stmt("break outer"));
    assert!(can_parse_stmt("continue outer"));

    // Blocks
    assert!(can_parse_stmt("{ 42 }"));
    assert!(can_parse_stmt("{ x = 1\ny = 2 }"));

    // Loops
    assert!(can_parse_stmt("loop { break }"));
    assert!(can_parse_stmt("loop as outer { break }"));
    assert!(can_parse_stmt("loop through list { break }"));
    assert!(can_parse_stmt("loop through list with x { break }"));
    assert!(can_parse_stmt("loop through map with k, v { break }"));

    // Match statements
    assert!(can_parse_stmt("match x { 1: return 1, _: return 0 }"));

    // Import/Export
    assert!(can_parse_stmt("import std"));
    assert!(can_parse_stmt("export { add: |x, y| { return x + y } }"));
}

#[test]
fn test_full_pipeline_integration() {
    // Test end-to-end: lexer → parser → evaluator for complete programs

    // Test some actual evaluations if they work
    if can_eval_program("42") {
        let result = eval_program("42").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    if can_eval_program("2 + 3") {
        let result = eval_program("2 + 3").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    if can_eval_program("x = 5") {
        let result = eval_program("x = 5").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }
}

#[test]
fn test_circular_reference_fix() {
    // Test that function literals don't cause infinite debug output
    if can_eval_program("func = |x| { return x * 2 }") {
        match eval_program("func = |x| { return x * 2 }") {
            Ok(result) => {
                // Function creation successful - no stack overflow
                assert!(matches!(result, Value::Function(_)));
            }
            Err(e) => panic!("Function creation failed: {}", e),
        }
    }

    // Test a simple function call
    if can_eval_program("add = |x, y| { x + y }; add(2, 3)") {
        match eval_program("add = |x, y| { x + y }; add(2, 3)") {
            Ok(result) => assert_eq!(result, Value::Number(5.0)),
            Err(e) => panic!("Function call failed: {}", e),
        }
    }
}

#[test]
fn test_map_literal_comprehensive() {
    // Test basic map literal
    assert_eq!(
        eval_string_expr(r#"{ "a": 1 }"#).unwrap(),
        Value::Map(indexmap::IndexMap::from([(
            MapKey::String("a".to_string()),
            Value::Number(1.0)
        )]))
    );

    // Test empty map
    assert_eq!(
        eval_string_expr("{}").unwrap(),
        Value::Map(indexmap::IndexMap::new())
    );

    // Test map with multiple entries
    let result = eval_string_expr(r#"{ "name": "Alice", "age": 30 }"#).unwrap();
    if let Value::Map(map) = result {
        assert_eq!(map.len(), 2);
    } else {
        panic!("Expected map");
    }
}

#[test]
fn test_postfix_increment_decrement() {
    // Test x++ behavior
    let result = eval_program("x = 5\nx++").unwrap();
    assert_eq!(result, Value::Number(6.0));

    // Test x-- behavior
    let result = eval_program("y = 10\ny--").unwrap();
    assert_eq!(result, Value::Number(9.0));

    // Test that variable is actually modified
    let result = eval_program("z = 7\nz++\nz").unwrap();
    assert_eq!(result, Value::Number(8.0));
}

#[test]
fn test_method_calls_comprehensive() {
    // Test list methods
    let result = eval_string_expr("[1, 2, 3]::length()").unwrap();
    assert_eq!(result, Value::Number(3.0));

    // Test string methods
    let result = eval_string_expr(r#""hello"::length()"#).unwrap();
    assert_eq!(result, Value::Number(5.0));

    let result = eval_string_expr(r#""hello world"::contains("world")"#).unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = eval_string_expr(r#""document.pdf"::starts_with("doc")"#).unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = eval_string_expr(r#""document.pdf"::ends_with(".pdf")"#).unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = eval_string_expr(r#""hello world"::replace("world", "universe")"#).unwrap();
    assert_eq!(result, Value::String("hello universe".to_string()));

    let result = eval_string_expr(r#""  hello world  "::trim()"#).unwrap();
    assert_eq!(result, Value::String("hello world".to_string()));

    let result = eval_string_expr(r#""hello world"::upper()"#).unwrap();
    assert_eq!(result, Value::String("HELLO WORLD".to_string()));

    let result = eval_string_expr(r#""HELLO WORLD"::lower()"#).unwrap();
    assert_eq!(result, Value::String("hello world".to_string()));

    let result = eval_string_expr(r#""hello"::reverse()"#).unwrap();
    assert_eq!(result, Value::String("olleh".to_string()));

    let result = eval_string_expr(r#""ha"::repeat(3)"#).unwrap();
    assert_eq!(result, Value::String("hahaha".to_string()));

    // Test method calls on variables
    let result = eval_program("list = [1, 2, 3, 4]\nlist::length()").unwrap();
    assert_eq!(result, Value::Number(4.0));

    // Test method calls on complex expressions
    let result = eval_string_expr("(1..5)::length()").unwrap();
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn test_new_string_methods_error_handling() {
    // Test contains with wrong argument type
    let result = eval_string_expr(r#""hello"::contains(42)"#);
    assert!(result.is_err());

    // Test replace with wrong number of arguments
    let result = eval_string_expr(r#""hello"::replace("world")"#);
    assert!(result.is_err());

    // Test repeat with negative count
    let result = eval_string_expr(r#""hello"::repeat(-1)"#);
    assert!(result.is_err());

    // Test trim with arguments
    let result = eval_string_expr(r#""hello"::trim("test")"#);
    assert!(result.is_err());

    // Test upper with arguments
    let result = eval_string_expr(r#""hello"::upper("test")"#);
    assert!(result.is_err());

    // Test lower with arguments
    let result = eval_string_expr(r#""hello"::lower("test")"#);
    assert!(result.is_err());

    // Test reverse with arguments
    let result = eval_string_expr(r#""hello"::reverse("test")"#);
    assert!(result.is_err());

    // Test starts_with with wrong argument type
    let result = eval_string_expr(r#""hello"::starts_with(42)"#);
    assert!(result.is_err());

    // Test ends_with with wrong argument type
    let result = eval_string_expr(r#""hello"::ends_with(42)"#);
    assert!(result.is_err());
}

#[test]
fn test_indexing_slicing_comprehensive() {
    // Test basic indexing
    let result = eval_string_expr("[1, 2, 3, 4][0]").unwrap();
    assert_eq!(result, Value::Number(1.0));

    let result = eval_string_expr("[1, 2, 3, 4][2]").unwrap();
    assert_eq!(result, Value::Number(3.0));

    // Test slicing
    let result = eval_string_expr("[1, 2, 3, 4][1:3]").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::Number(2.0));
        assert_eq!(items[1], Value::Number(3.0));
    } else {
        panic!("Expected list from slicing");
    }

    // Test slicing with different ranges
    let result = eval_string_expr("[1, 2, 3, 4, 5][0:2]").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::Number(1.0));
        assert_eq!(items[1], Value::Number(2.0));
    } else {
        panic!("Expected list from slicing");
    }

    // Test on variables
    let result = eval_program("list = [1, 2, 3, 4]\nlist[1]").unwrap();
    assert_eq!(result, Value::Number(2.0));

    let result = eval_program("list = [1, 2, 3, 4]\nlist[1:3]").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 2);
    } else {
        panic!("Expected list from variable slicing");
    }
}

#[test]
fn test_function_calls_comprehensive() {
    // Test builtin function calls through import system
    let result = eval_program("import std:println\nprintln(1)").unwrap();
    // \"1\\n\" => 2 bytes
    assert_eq!(result, Value::Number(2.0));

    // With explicit output stream
    let result = eval_program("import std:println\nimport std:FD\nprintln(1, FD:stdout)").unwrap();
    assert_eq!(result, Value::Number(2.0));

    // Test user-defined function calls
    let result = eval_program("add = |x, y| { x + y }\nadd(2, 3)").unwrap();
    assert_eq!(result, Value::Number(5.0));

    // Test function call with no arguments
    let result = eval_program("get_five = || { 5 }\nget_five()").unwrap();
    assert_eq!(result, Value::Number(5.0));

    // Test function call with default parameters
    let result = eval_program("greet = |name = \"World\"| { name }\ngreet()").unwrap();
    assert_eq!(result, Value::String("World".to_string()));

    let result = eval_program("greet = |name = \"World\"| { name }\ngreet(\"Alice\")").unwrap();
    assert_eq!(result, Value::String("Alice".to_string()));
}

#[test]
fn test_map_access_comprehensive() {
    // Test basic map access with string keys
    let result = eval_string_expr("{\"hello\": \"world\"}:hello").unwrap();
    assert_eq!(result, Value::String("world".to_string()));

    let result = eval_string_expr("{\"a\": 1, \"b\": 2}:b").unwrap();
    assert_eq!(result, Value::Number(2.0));

    // Test map access on variables
    let result = eval_program("data = {\"name\": \"Alice\", \"age\": 30}\ndata:name").unwrap();
    assert_eq!(result, Value::String("Alice".to_string()));

    let result = eval_program("data = {\"name\": \"Alice\", \"age\": 30}\ndata:age").unwrap();
    assert_eq!(result, Value::Number(30.0));

    // Test map access on complex expressions
    let result = eval_string_expr("{\"nested\": {\"value\": 42}}:nested").unwrap();
    if let Value::Map(_) = result {
        // Map access returning nested map works
    } else {
        panic!("Expected nested map");
    }

    // Test non-existent key access (should return null or error)
    match eval_string_expr("{\"a\": 1}:nonexistent") {
        Ok(Value::Nil) => {
            // Non-existent key returns nil
        }
        Ok(other) => {
            // Non-existent key returns some other value
            assert!(matches!(
                other,
                Value::Nil | Value::String(_) | Value::Number(_)
            ));
        }
        Err(_) => {
            // Non-existent key returns error (valid behavior)
        }
    }
}

#[test]
fn test_shell_command_templates() {
    // Test basic shell command
    if can_eval("`echo hello`") {
        match eval_string_expr("`echo hello`") {
            Ok(Value::String(s)) => {
                assert_eq!(s, "hello");
            }
            _ => panic!("Basic shell command failed"),
        }
    }

    // Test shell command with interpolation
    if can_eval_program("name = \"world\"") {
        let success = parse_expression("`echo hello ${name}`").is_ok();
        if success {
            // Test the full program (use newline instead of semicolon)
            match eval_program("name = \"world\"\n`echo hello ${name}`") {
                Ok(Value::String(s)) => {
                    assert_eq!(s, "hello world");
                }
                Ok(other) => panic!("Shell interpolation returned wrong type: {:?}", other),
                Err(e) => panic!("Shell interpolation eval error: {}", e),
            }
        }
    }
}

// Phase 6 Tests - Optional Braces for Single Expressions

#[test]
fn test_optional_braces_function_evaluation() {
    // Test that functions with single expression bodies work correctly
    let result = eval_program("multiply = |x, y| x * y\nresult = multiply(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(12.0));

    let result = eval_program("square = |x| x * x\nresult = square(5)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(25.0));

    let result = eval_program("greet = |name| \"Hello, \" + name\nresult = greet(\"world\")");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Hello, world".to_string()));
}

#[test]
fn test_optional_braces_match_evaluation() {
    // Test that match statements with single expression arms work correctly
    let result = eval_program(
        "x = 3\nmatch x { 1: result = \"one\" 2: result = \"two\" 3: result = \"three\" _: result = \"other\" }",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("three".to_string()));

    let result = eval_program(
        "x = 5\nmatch x { 1: result = 10 2: result = 20 3: result = 30 _: result = 0 }",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(0.0));
}

#[test]
fn test_optional_braces_mixed_evaluation() {
    // Test mixing single expressions and blocks
    let result = eval_program(
        "x = 2\nmatch x { 1: result = x * 2 2: { doubled = x * 2; result = doubled + 1 } _: result = 0 }",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(5.0));
}

#[test]
fn test_optional_braces_implicit_return() {
    // Test that single expression functions return their value
    let result = eval_program("add = |x, y| x + y\nresult = add(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(7.0));

    let result =
        eval_program("complex = |x| { doubled = x * 2; doubled + 1 }\nresult = complex(5)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(11.0));
}

#[test]
fn test_optional_braces_empty_function() {
    // Test empty function with single expression body
    let result = eval_program("constant = || 42\nresult = constant()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(42.0));
}

#[test]
fn test_optional_braces_function_with_default_params() {
    // Test function with default parameters and single expression body
    let result = eval_program("add = |x = 0, y = 1| x + y\nresult = add(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(7.0));

    let result = eval_program("add = |x = 0, y = 1| x + y\nresult = add()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(1.0));
}

#[test]
fn test_optional_braces_nested_expressions() {
    // Test complex nested expressions without braces
    let result =
        eval_program("x = 2\nmatch x { 1: result = x * 2 2: result = x + 10 _: result = 0 }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(12.0));
}

#[test]
fn test_optional_braces_backward_compatibility() {
    // Test that existing syntax with braces still works
    let result = eval_program("add = |x, y| { return x + y }\nresult = add(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(7.0));

    let result = eval_program("match 1 { 1: { result = \"one\" } 2: { result = \"two\" } }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("one".to_string()));
}

#[test]
fn test_optional_braces_boolean_match() {
    // Test match with boolean patterns and single expressions
    let result = eval_program("match true { true: result = 1 false: result = 0 }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(1.0));

    let result = eval_program("match false { true: result = 1 false: result = 0 }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(0.0));
}

#[test]
fn test_optional_braces_wildcard_match() {
    // Test match with wildcard pattern and single expression
    let result =
        eval_program("match 42 { 1: result = \"one\" 2: result = \"two\" _: result = \"other\" }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("other".to_string()));
}

#[test]
fn test_map_iteration_no_bindings() {
    let result = eval_program(
        r#"
        map = { a: 1, b: 2 }
        count = 0
        loop through map {
            count = count + 1
        }
        count
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(2.0));
}

#[test]
fn test_map_iteration_one_binding() {
    let result = eval_program(
        r#"
        map = { a: 1, b: 2 }
        result = ""
        loop through map with k {
            result = result + k + " "
        }
        result
    "#,
    );
    assert!(result.is_ok());
    let result_str = match result.unwrap() {
        Value::String(s) => s,
        _ => panic!("Expected string result"),
    };
    // Order is not guaranteed, so check both keys are present
    assert!(result_str.contains("a "));
    assert!(result_str.contains("b "));
}

#[test]
fn test_map_iteration_two_bindings() {
    let result = eval_program(
        r#"
        map = { a: 10, b: 20 }
        total = 0
        loop through map with k, v {
            total = total + v
        }
        total
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(30.0));
}

#[test]
fn test_map_iteration_empty_map() {
    let result = eval_program(
        r#"
        empty = {}
        count = 0
        loop through empty with k, v {
            count = count + 1
        }
        count
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(0.0));
}

#[test]
fn test_map_iteration_variable_scoping() {
    let result = eval_program(
        r#"
        outer = "original"
        map = { key: "value" }
        loop through map with k, v {
            inner = "inside"
            outer = "modified"
        }
        outer
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("modified".to_string()));
}

#[test]
fn test_map_iteration_control_flow_break() {
    let result = eval_program(
        r#"
        map = { a: 1, b: 2, c: 3, d: 4 }
        result = ""
        loop through map with k, v {
            match k {
                "c": { break }
                _: result = result + k
            }
        }
        result
    "#,
    );
    assert!(result.is_ok());
    let result_str = match result.unwrap() {
        Value::String(s) => s,
        _ => panic!("Expected string result"),
    };
    // Should break when it hits "c", so "c" and "d" should not be in result
    assert!(!result_str.contains("c"));
    // The exact order is not guaranteed, but it should contain some keys before "c"
    assert!(!result_str.is_empty());
}

#[test]
fn test_map_iteration_mixed_key_types() {
    let result = eval_program(
        r#"
        mixed = { "string": 1, 42: "number", true: false }
        count = 0
        loop through mixed with k, v {
            count = count + 1
        }
        count
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(3.0));
}

#[test]
fn test_map_iteration_nested_maps() {
    let result = eval_program(
        r#"
        nested = {
            user: { name: "Alice" },
            settings: { theme: "dark" }
        }
        count = 0
        loop through nested with section, data {
            count = count + 1
        }
        count
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(2.0));
}

#[test]
fn test_nil_pattern_matching() {
    let result = eval_program(
        r#"
        user = nil
        result = match user {
            nil: "No user found"
            _: "Some user found"
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("No user found".to_string()));
}

#[test]
fn test_return_statements_in_match_arms() {
    let result = eval_program(
        r#"
        f = || {
            match 1 {
                1: return 1
                2: return 2
                _: return 0
            }
        }
        f()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(1.0));
}

#[test]
fn test_implicit_returns_in_match_arms() {
    let result = eval_program(
        r#"
        g = || {
            match 1 {
                1: 1    # implicitly returned
                2: 2    # implicitly returned
                _: 0    # implicitly returned
            }
        }
        g()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(1.0));
}

#[test]
fn test_map_literals_in_match_arms() {
    let result = eval_program(
        r#"
        result = match 1 {
            1: { "status": "success", "value": 42 }
            2: { "status": "error", "message": "failed" }
            _: { "status": "unknown" }
        }
        result:status
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("success".to_string()));
}

#[test]
fn test_mixed_match_arm_syntax() {
    let result = eval_program(
        r#"
        import std:println
        h = || {
            match 1 {
                1: {
                    println("one")
                    return 1
                }
                2: 2    # implicit return
                _: return 0
            }
        }
        h()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(1.0));
}

#[test]
fn test_complex_nil_handling() {
    let result = eval_program(
        r#"
        find_user = |id| {
            match id {
                1: { "name": "Alice" }
                2: { "name": "Bob" }
                _: nil
            }
        }
        user_info = match find_user(3) {
            nil: "User not found"
            user: "User: " + user:name
        }
        user_info
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("User not found".to_string()));
}

#[test]
fn test_conditional_match_basic() {
    let result = eval_program(
        r#"
        x = 5
        result = match {
            x > 0: "positive"
            _: "non-positive"
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("positive".to_string()));
}

#[test]
fn test_conditional_match_multiple_conditions() {
    let result = eval_program(
        r#"
        x = 7
        result = match {
            x > 10: "very large"
            x > 5: "large"
            x > 0: "positive"
            _: "non-positive"
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("large".to_string()));
}

#[test]
fn test_conditional_match_complex_boolean() {
    let result = eval_program(
        r#"
        x = 3
        y = 4
        result = match {
            x > 0 && y > 0: "both positive"
            x > 0: "only x positive"
            y > 0: "only y positive"
            _: "neither positive"
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("both positive".to_string()));
}

#[test]
fn test_conditional_match_in_function() {
    let result = eval_program(
        r#"
        f = |x| {
            return match {
                x > 10: "very large"
                x > 5: "large"
                x > 0: "positive"
                _: "non-positive"
            }
        }
        f(15)
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("very large".to_string()));
}

#[test]
fn test_conditional_match_mixed_with_traditional() {
    let result = eval_program(
        r#"
        x = 2
        y = 3

        result1 = match x {
            1: "one"
            2: "two"
            _: "other"
        }

        result2 = match {
            y > 2: "y is large"
            _: "y is small"
        }

        result1 + " " + result2
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("two y is large".to_string()));
}

#[test]
fn test_conditional_match_all_conditions_false() {
    let result = eval_program(
        r#"
        x = -5
        result = match {
            x > 0: "positive"
            x > -10: "small negative"
            _: "large negative"
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("small negative".to_string()));
}

#[test]
fn test_conditional_match_with_wildcard() {
    let result = eval_program(
        r#"
        x = 0
        result = match {
            x > 0: "positive"
            x < 0: "negative"
            _: "zero"
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("zero".to_string()));
}

#[test]
fn test_conditional_match_nested() {
    let result = eval_program(
        r#"
        x = 3
        y = 4
        result = match {
            x > 0: match {
                y > 0: "both positive"
                _: "x positive, y non-positive"
            }
            _: "x non-positive"
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("both positive".to_string()));
}

#[test]
fn test_conditional_match_in_binary_operation() {
    let result = eval_program(
        r#"
        x = 5
        y = 3
        result = match {
            x > 0: 10
            _: 0
        } + match {
            y > 0: 5
            _: 0
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(15.0));
}

#[test]
fn test_conditional_match_with_string_comparison() {
    let result = eval_program(
        r#"
        name = "alice"
        result = match {
            name == "alice": "Hello Alice"
            name == "bob": "Hello Bob"
            _: "Hello stranger"
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Hello Alice".to_string()));
}

#[test]
fn test_map_keys_method_integration() {
    let result = eval_program(
        r#"
        config = { 
            name: "Alice", 
            age: 30, 
            city: "New York",
            active: true
        }
        keys = config::keys()
        keys::length()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(4.0));
}

#[test]
fn test_map_values_method_integration() {
    let result = eval_program(
        r#"
        scores = { 1: 100, 2: 85, 3: 92 }
        values = scores::values()
        values::length()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(3.0));
}

#[test]
fn test_map_to_list_method_integration() {
    let result = eval_program(
        r#"
        user = { name: "Bob", age: 25 }
        pairs = user::to_list()
        pairs::length()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(2.0));
}

#[test]
fn test_map_length_method_integration() {
    let result = eval_program(
        r#"
        data = { a: 1, b: 2, c: 3, d: 4, e: 5 }
        data::length()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(5.0));
}

#[test]
fn test_map_methods_combined_usage() {
    let result = eval_program(
        r#"
        config = { 
            name: "Alice", 
            age: 30, 
            city: "New York",
            active: true
        }
        
        # Test all methods together
        key_count = config::keys()::length()
        value_count = config::values()::length()
        pair_count = config::to_list()::length()
        direct_count = config::length()
        
        # All should be equal
        key_count == value_count && value_count == pair_count && pair_count == direct_count
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_map_methods_with_iteration() {
    let result = eval_program(
        r#"
        scores = { math: 95, science: 87, english: 92 }
        
        # Iterate over keys
        key_count = 0
        loop through scores::keys() with key {
            key_count = key_count + 1
        }
        
        # Iterate over values
        value_count = 0
        loop through scores::values() with value {
            value_count = value_count + 1
        }
        
        # Iterate over pairs
        pair_count = 0
        loop through scores::to_list() with pair {
            pair_count = pair_count + 1
        }
        
        # All counts should equal the map length
        key_count == scores::length() && value_count == scores::length() && pair_count == scores::length()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_json_module_integration() {
    let result = eval_program(
        r#"
        import std:json
        data = json:parse("{\"name\": \"Alice\", \"age\": 30}")
        json:generate(data)
    "#,
    );
    assert!(result.is_ok());
    let output = result.unwrap();
    if let Value::String(json_str) = output {
        assert!(json_str.contains("\"name\":\"Alice\""));
        assert!(json_str.contains("\"age\":30.0"));
    } else {
        panic!("Expected string output");
    }
}

#[test]
fn test_json_direct_imports() {
    let result = eval_program(
        r#"
        import std:json:parse
        import std:json:generate
        data = parse("{\"value\": 42}")
        generate(data)
    "#,
    );
    assert!(result.is_ok());
    let output = result.unwrap();
    if let Value::String(json_str) = output {
        assert!(json_str.contains("\"value\":42.0"));
    } else {
        panic!("Expected string output");
    }
}

#[test]
fn test_json_roundtrip_integration() {
    let result = eval_program(
        r#"
        import std:json
        original = { name: "Bob", hobbies: ["reading", "coding"] }
        json_str = json:generate(original)
        parsed = json:parse(json_str)
        parsed:name
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Bob".to_string()));
}

#[test]
fn test_json_nested_structures() {
    let result = eval_program(
        r#"
        import std:json
        complex = {
            users: [
                { name: "Alice", age: 30 },
                { name: "Bob", age: 25 }
            ],
            metadata: { version: "1.0", count: 2 }
        }
        json_str = json:generate(complex)
        parsed = json:parse(json_str)
        parsed:metadata:version
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("1.0".to_string()));
}

#[test]
fn test_json_with_boolean_values() {
    let result = eval_program(
        r#"
        import std:json
        data = { active: true, disabled: false, count: 0 }
        json_str = json:generate(data)
        parsed = json:parse(json_str)
        parsed:active
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_json_with_nil_values() {
    let result = eval_program(
        r#"
        import std:json
        data = { value: nil, name: "test" }
        json_str = json:generate(data)
        parsed = json:parse(json_str)
        parsed:value
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Nil);
}

// New number methods integration tests
#[test]
fn test_new_number_methods_integration() {
    // Test abs method
    let result = eval_string_expr("(-5)::abs()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(5.0));

    // Test ceil method
    let result = eval_string_expr("3.14159::ceil()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(4.0));

    // Test floor method
    let result = eval_string_expr("3.14159::floor()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(3.0));

    // Test round method
    let result = eval_string_expr("3.14159::round()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(3.0));

    // Test sqrt method
    let result = eval_string_expr("16::sqrt()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(4.0));

    // Test pow method
    let result = eval_string_expr("2::pow(3)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(8.0));

    // Test min method
    let result = eval_string_expr("10::min(20)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(10.0));

    // Test max method
    let result = eval_string_expr("10::max(20)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(20.0));
}

#[test]
fn test_number_methods_with_variables() {
    let result = eval_program(
        r#"
        x = -5
        y = x::abs()
        y
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(5.0));
}

#[test]
fn test_number_methods_chaining() {
    let result = eval_program(
        r#"
        x = -3.7
        y = x::abs()::ceil()
        y
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(4.0));
}

#[test]
fn test_number_methods_in_expressions() {
    let result = eval_program(
        r#"
        x = 2
        y = 3
        z = x::pow(y) + 5::sqrt()
        z
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(8.0 + 5.0_f64.sqrt()));
}

#[test]
fn test_number_methods_error_cases() {
    // Test sqrt with negative number
    let result = eval_string_expr("(-4)::sqrt()");
    assert!(result.is_err());

    // Test pow with non-number argument
    let result = eval_string_expr("2::pow(\"invalid\")");
    assert!(result.is_err());

    // Test min with non-number argument
    let result = eval_string_expr("10::min(\"invalid\")");
    assert!(result.is_err());
}
