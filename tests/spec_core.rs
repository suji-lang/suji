use suji_values::DecimalNumber;
mod common;

use common::{eval_program, eval_string_expr};
use suji_values::Value;

#[test]
fn test_postfix_operators() {
    assert_eq!(
        eval_program("x = 5\nx++").unwrap(),
        Value::Number(DecimalNumber::from_i64(6))
    );
    assert_eq!(
        eval_program("y = 10\ny++\ny").unwrap(),
        Value::Number(DecimalNumber::from_i64(11))
    );
    assert_eq!(
        eval_program("x = 5\nx--").unwrap(),
        Value::Number(DecimalNumber::from_i64(4))
    );
    assert_eq!(
        eval_program("y = 10\ny--\ny").unwrap(),
        Value::Number(DecimalNumber::from_i64(9))
    );
    assert_eq!(
        eval_program("z = 7\nz++\nz").unwrap(),
        Value::Number(DecimalNumber::from_i64(8))
    );
    assert_eq!(
        eval_program("w = 12\nw--\nw").unwrap(),
        Value::Number(DecimalNumber::from_i64(11))
    );
    assert_eq!(
        eval_program("a = 1\na++\na++\na").unwrap(),
        Value::Number(DecimalNumber::from_i64(3))
    );
}

#[test]
fn test_semicolon_statement_separators() {
    let result = eval_program("import std:println; x = 10; y = 20; result = x + y").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(30)));

    let result = eval_program("a = 5; b = 7; c = a * b; result = c").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(35)));
}

#[test]
fn test_function_implicit_return_and_match_expression() {
    // Implicit return from last expression
    let result = eval_program("mul = |x, y| x * y\nresult = mul(3, 4)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(12)));

    // Match as an expression yielding a value
    let result =
        eval_program("x = 3\ny = match x { 3 => { 5 + 5 }, 4 => { 2 * 3 }, }\nresult = y").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(10)));
}

#[test]
fn test_optional_braces_for_functions_and_match_arms() {
    // Function without braces
    let result = eval_program("square = |x| x * x\nresult = square(6)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(36)));

    // Match arms without braces
    let result =
        eval_program("x = 5\nresult = match x { 5 => \"five\", 6 => \"six\", _ => \"other\", }")
            .unwrap();
    assert_eq!(result, Value::String("five".to_string()));
}

#[test]
fn test_imports_and_renaming() {
    // Import a specific item and call it
    let result = eval_program("import std:println as say\nsay(\"hi\")\nresult = 1").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));

    // Import a module and then a function
    let result = eval_program("import std:println\nprintln(\"x\")\nresult = 2").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));
}

#[test]
fn test_env_map_basic_usage() {
    // Basic import and contains check; avoid asserting real env values
    let result = eval_program(
        "import std:env:var\nresult = var::contains(\"PATH\") || var::contains(\"HOME\")",
    )
    .unwrap();
    assert!(matches!(result, Value::Boolean(_)), "bool expected");
}

#[test]
fn test_comparison_operators() {
    assert_eq!(eval_string_expr("5 > 3").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("2 < 1").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("4 >= 4").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("3 <= 2").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("5 == 5").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("3 != 4").unwrap(), Value::Boolean(true));

    assert_eq!(
        eval_string_expr(r#""abc" == "abc""#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""abc" != "def""#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""abc" < "def""#).unwrap(),
        Value::Boolean(true)
    );

    assert_eq!(
        eval_string_expr("true == true").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("true != false").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("false == false").unwrap(),
        Value::Boolean(true)
    );
}

#[test]
fn test_logical_operators() {
    assert_eq!(
        eval_string_expr("true && true").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("true && false").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr("false && true").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr("false && false").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr("true || true").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("true || false").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("false || true").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("false || false").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(eval_string_expr("!true").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("!false").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("!!true").unwrap(), Value::Boolean(true));
    assert_eq!(
        eval_string_expr("false && (5 / 0)").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr("true || (5 / 0)").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("(true && false) || (false || true)").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("!(true && false)").unwrap(),
        Value::Boolean(true)
    );
}

#[test]
fn test_ranges() {
    let result = eval_string_expr("0..3").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(0)));
        assert_eq!(items[1], Value::Number(DecimalNumber::from_i64(1)));
        assert_eq!(items[2], Value::Number(DecimalNumber::from_i64(2)));
    } else {
        panic!("Expected list from range");
    }

    let result = eval_string_expr("-2..2").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(-2)));
        assert_eq!(items[1], Value::Number(DecimalNumber::from_i64(-1)));
        assert_eq!(items[2], Value::Number(DecimalNumber::from_i64(0)));
        assert_eq!(items[3], Value::Number(DecimalNumber::from_i64(1)));
    } else {
        panic!("Expected list from negative range");
    }

    let result = eval_string_expr("5..3").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(5)));
        assert_eq!(items[1], Value::Number(DecimalNumber::from_i64(4)));
    } else {
        panic!("Expected descending range");
    }

    let result = eval_string_expr("3..3").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 0);
    } else {
        panic!("Expected empty range");
    }

    let result = eval_string_expr("1..2+2").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(1)));
        assert_eq!(items[1], Value::Number(DecimalNumber::from_i64(2)));
        assert_eq!(items[2], Value::Number(DecimalNumber::from_i64(3)));
    } else {
        panic!("Expected list from range expression");
    }
}
