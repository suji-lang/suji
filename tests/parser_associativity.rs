use nnlang::ast::{BinaryOp, Expr, Literal};

mod common;
use common::parse_expression;

#[test]
fn power_is_right_associative() {
    let result = parse_expression("a ^ b ^ c");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary {
        left, op, right, ..
    } = expr
    {
        assert_eq!(op, BinaryOp::Power);
        if let Expr::Binary { op: right_op, .. } = right.as_ref() {
            assert_eq!(*right_op, BinaryOp::Power);
        } else {
            panic!("Expected right to be a power expression");
        }
        if let Expr::Literal(Literal::Identifier(name, _)) = left.as_ref() {
            assert_eq!(name, "a");
        } else {
            panic!("Expected left to be identifier a");
        }
    } else {
        panic!("Expected top-level power expression");
    }
}

#[test]
fn backward_apply_is_right_associative() {
    let result = parse_expression("a <| b <| c");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary {
        left, op, right, ..
    } = expr
    {
        assert_eq!(op, BinaryOp::PipeApplyBwd);
        if let Expr::Binary { op: right_op, .. } = right.as_ref() {
            assert_eq!(*right_op, BinaryOp::PipeApplyBwd);
        } else {
            panic!("Expected right to be a backward-apply expression");
        }
        if let Expr::Literal(Literal::Identifier(name, _)) = left.as_ref() {
            assert_eq!(name, "a");
        } else {
            panic!("Expected left to be identifier a");
        }
    } else {
        panic!("Expected top-level backward-apply expression");
    }
}

#[test]
fn subtraction_is_left_associative() {
    let result = parse_expression("a - b - c");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary {
        left, op, right, ..
    } = expr
    {
        assert_eq!(op, BinaryOp::Subtract);
        if let Expr::Binary { op: left_op, .. } = left.as_ref() {
            assert_eq!(*left_op, BinaryOp::Subtract);
        } else {
            panic!("Expected left to be a subtraction expression");
        }
        if let Expr::Literal(Literal::Identifier(name, _)) = right.as_ref() {
            assert_eq!(name, "c");
        } else {
            panic!("Expected right to be identifier c");
        }
    } else {
        panic!("Expected top-level subtraction expression");
    }
}

#[test]
fn stream_pipe_is_left_associative() {
    let result = parse_expression("a | b | c");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary {
        left, op, right, ..
    } = expr
    {
        assert_eq!(op, BinaryOp::Pipe);
        if let Expr::Binary { op: left_op, .. } = left.as_ref() {
            assert_eq!(*left_op, BinaryOp::Pipe);
        } else {
            panic!("Expected left to be a stream pipe expression");
        }
        if let Expr::Literal(Literal::Identifier(name, _)) = right.as_ref() {
            assert_eq!(name, "c");
        } else {
            panic!("Expected right to be identifier c");
        }
    } else {
        panic!("Expected top-level stream pipe expression");
    }
}

#[test]
fn forward_apply_is_left_associative() {
    let result = parse_expression("a |> b |> c");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary {
        left, op, right, ..
    } = expr
    {
        assert_eq!(op, BinaryOp::PipeApplyFwd);
        if let Expr::Binary { op: left_op, .. } = left.as_ref() {
            assert_eq!(*left_op, BinaryOp::PipeApplyFwd);
        } else {
            panic!("Expected left to be a forward-apply expression");
        }
        if let Expr::Literal(Literal::Identifier(name, _)) = right.as_ref() {
            assert_eq!(name, "c");
        } else {
            panic!("Expected right to be identifier c");
        }
    } else {
        panic!("Expected top-level forward-apply expression");
    }
}
