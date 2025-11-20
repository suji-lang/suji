use super::common::{eval_program, eval_string_expr};
use suji_values::DecimalNumber;
use suji_values::Value;

#[test]
fn test_arithmetic_operations() {
    assert_eq!(
        eval_string_expr("2 + 3").unwrap(),
        Value::Number(DecimalNumber::from_i64(5))
    );
    assert_eq!(
        eval_string_expr("10 - 4").unwrap(),
        Value::Number(DecimalNumber::from_i64(6))
    );
    assert_eq!(
        eval_string_expr("3 * 4").unwrap(),
        Value::Number(DecimalNumber::from_i64(12))
    );
    assert_eq!(
        eval_string_expr("15 / 3").unwrap(),
        Value::Number(DecimalNumber::from_i64(5))
    );
    assert_eq!(
        eval_string_expr("17 % 5").unwrap(),
        Value::Number(DecimalNumber::from_i64(2))
    );
    assert_eq!(
        eval_string_expr("2 ^ 3").unwrap(),
        Value::Number(DecimalNumber::from_i64(8))
    );

    // Precedence
    assert_eq!(
        eval_string_expr("2 + 3 * 4").unwrap(),
        Value::Number(DecimalNumber::from_i64(14))
    );
    assert_eq!(
        eval_string_expr("(2 + 3) * 4").unwrap(),
        Value::Number(DecimalNumber::from_i64(20))
    );
    assert_eq!(
        eval_string_expr("2 ^ 3 ^ 2").unwrap(),
        Value::Number(DecimalNumber::from_i64(512))
    );
}

#[test]
fn test_unary_operations() {
    assert_eq!(
        eval_string_expr("-5").unwrap(),
        Value::Number(DecimalNumber::from_i64(-5))
    );
    assert_eq!(eval_string_expr("!true").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("!false").unwrap(), Value::Boolean(true));
    assert_eq!(
        eval_string_expr("-(-10)").unwrap(),
        Value::Number(DecimalNumber::from_i64(10))
    );
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

#[test]
fn test_basic_literals() {
    assert_eq!(
        eval_string_expr("42").unwrap(),
        Value::Number(DecimalNumber::from_i64(42))
    );
    assert_eq!(
        eval_string_expr("2.5").unwrap(),
        Value::Number(DecimalNumber::parse("2.5").unwrap())
    );
    assert_eq!(eval_string_expr("true").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("false").unwrap(), Value::Boolean(false));
}

#[test]
fn test_error_cases() {
    assert!(eval_string_expr("5 / 0").is_err());
    assert!(eval_string_expr("5 % 0").is_err());
    assert!(eval_string_expr(r#"5 + \"hello\""#).is_err());
    assert!(eval_string_expr("true * 3").is_err());
    assert!(eval_string_expr("!5").is_err());
    assert!(eval_string_expr("-true").is_err());
}

#[test]
fn test_short_circuit_evaluation() {
    assert_eq!(
        eval_string_expr("false && (5 / 0)").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr("true || (5 / 0)").unwrap(),
        Value::Boolean(true)
    );
}

#[test]
fn test_complex_literals() {
    let Value::String(s) = eval_string_expr(r#""hello""#).unwrap() else {
        panic!("Expected string")
    };
    assert_eq!(s, "hello");

    let Value::List(items) = eval_string_expr("[1, 2, 3]").unwrap() else {
        panic!("Expected list")
    };
    assert_eq!(
        items,
        vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3))
        ]
    );

    let Value::Tuple(items) = eval_string_expr("(1, 2)").unwrap() else {
        panic!("Expected tuple")
    };
    assert_eq!(
        items,
        vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2))
        ]
    );

    let Value::Map(_) = eval_string_expr(r#"{ "a": 1 }"#).unwrap() else {
        panic!("Expected map")
    };

    let Value::List(items) = eval_string_expr("1..4").unwrap() else {
        panic!("Expected range list")
    };
    assert_eq!(
        items,
        vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3))
        ]
    );
}

#[test]
fn test_single_element_tuple_syntax() {
    // Test single-element tuple syntax (x,) vs grouping (x)

    // Single-element tuple with trailing comma should create a tuple
    let Value::Tuple(items) = eval_string_expr("(42,)").unwrap() else {
        panic!("Expected single-element tuple from (42,)")
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(42)));

    // Grouping without trailing comma should just return the value
    let result = eval_string_expr("(42)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(42)));
    assert!(
        !matches!(result, Value::Tuple(_)),
        "Expected number, not tuple from (42)"
    );

    // Test with string values too
    let Value::Tuple(items) = eval_string_expr("(\"hello\",)").unwrap() else {
        panic!("Expected single-element tuple from (\"hello\",)")
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], Value::String("hello".to_string()));

    // Grouping with string
    let result = eval_string_expr("(\"hello\")").unwrap();
    assert_eq!(result, Value::String("hello".to_string()));
    assert!(
        !matches!(result, Value::Tuple(_)),
        "Expected string, not tuple from (\"hello\")"
    );

    // Test with expressions
    let Value::Tuple(items) = eval_string_expr("(2 + 3,)").unwrap() else {
        panic!("Expected single-element tuple from (2 + 3,)")
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(5)));

    // Grouping with expression
    let result = eval_string_expr("(2 + 3)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));
    assert!(
        !matches!(result, Value::Tuple(_)),
        "Expected number, not tuple from (2 + 3)"
    );
}

#[test]
fn test_program_evaluation() {
    assert_eq!(
        eval_program("42").unwrap(),
        Value::Number(DecimalNumber::from_i64(42))
    );
    assert_eq!(
        eval_program("2 + 3").unwrap(),
        Value::Number(DecimalNumber::from_i64(5))
    );
    assert_eq!(
        eval_program("x = 5").unwrap(),
        Value::Number(DecimalNumber::from_i64(5))
    );
}

// ============================================================================
// Postfix Operators Tests
// ============================================================================

#[test]
fn test_postfix_operators() {
    assert_eq!(
        eval_program("x = 5\nx++").unwrap(),
        Value::Number(DecimalNumber::from_i64(6))
    );
    assert_eq!(
        eval_program("y = 10\ny--").unwrap(),
        Value::Number(DecimalNumber::from_i64(9))
    );
    assert_eq!(
        eval_program("z = 7\nz++\nz").unwrap(),
        Value::Number(DecimalNumber::from_i64(8))
    );
}
