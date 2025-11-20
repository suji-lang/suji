use suji_ast::*;

use super::common::{parse_expression, parse_program};

#[test]
fn test_parse_range_exclusive() {
    let result = parse_expression("0..5");
    assert!(result.is_ok());

    // Verify it parses as BinaryOp::Range
    if let Ok(Expr::Binary {
        op: BinaryOp::Range,
        ..
    }) = result
    {
        // Success
    } else {
        panic!("Expected Range binary operation");
    }
}

#[test]
fn test_parse_range_inclusive() {
    let result = parse_expression("0..=5");
    assert!(result.is_ok());

    // Verify it parses as BinaryOp::RangeInclusive
    if let Ok(Expr::Binary {
        op: BinaryOp::RangeInclusive,
        ..
    }) = result
    {
        // Success
    } else {
        panic!("Expected RangeInclusive binary operation");
    }
}

#[test]
fn test_parse_range_in_expression() {
    let input = "x = 1..=10";
    let ast = parse_program(input).unwrap();

    // Should parse successfully
    assert_eq!(ast.len(), 1);
}

#[test]
fn test_parse_range_negative_numbers() {
    let input = "-5..=5";
    let result = parse_expression(input);

    // Should parse successfully
    assert!(result.is_ok());
}

#[test]
fn test_parse_both_range_types() {
    let input = "a = 0..5\nb = 0..=5";
    let ast = parse_program(input).unwrap();

    // Should parse both assignments successfully
    assert_eq!(ast.len(), 2);

    // First is exclusive range
    if let Stmt::Expr(Expr::Assign { value, .. }) = &ast[0] {
        if let Expr::Binary {
            op: BinaryOp::Range,
            ..
        } = &**value
        {
            // Success
        } else {
            panic!("Expected Range binary operation");
        }
    } else {
        panic!("Expected assignment expression");
    }

    // Second is inclusive range
    if let Stmt::Expr(Expr::Assign { value, .. }) = &ast[1] {
        if let Expr::Binary {
            op: BinaryOp::RangeInclusive,
            ..
        } = &**value
        {
            // Success
        } else {
            panic!("Expected RangeInclusive binary operation");
        }
    } else {
        panic!("Expected assignment expression");
    }
}

#[test]
fn test_parse_range_in_loop() {
    let input = "loop through 0..=10 with i { }";
    let ast = parse_program(input).unwrap();

    // Should parse successfully
    assert_eq!(ast.len(), 1);
}
