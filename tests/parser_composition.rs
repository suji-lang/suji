use nnlang::ast::{BinaryOp, Expr, Literal};

mod common;
use common::parse_expression;

#[test]
fn test_parse_compose_right_and_left() {
    // f >> g
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = parse_expression("f >> g")
    {
        assert_eq!(op, BinaryOp::ComposeRight);
        match (left.as_ref(), right.as_ref()) {
            (
                Expr::Literal(Literal::Identifier(l, _)),
                Expr::Literal(Literal::Identifier(r, _)),
            ) => {
                assert_eq!(l, "f");
                assert_eq!(r, "g");
            }
            _ => panic!("Expected identifier operands"),
        }
    } else {
        panic!("Expected composition expression");
    }

    // f << g
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = parse_expression("f << g")
    {
        assert_eq!(op, BinaryOp::ComposeLeft);
        match (left.as_ref(), right.as_ref()) {
            (
                Expr::Literal(Literal::Identifier(l, _)),
                Expr::Literal(Literal::Identifier(r, _)),
            ) => {
                assert_eq!(l, "f");
                assert_eq!(r, "g");
            }
            _ => panic!("Expected identifier operands"),
        }
    } else {
        panic!("Expected composition expression");
    }
}

#[test]
fn test_composition_precedence_with_pipe_apply() {
    // x |> f >> g  => top-level is |> with right side as (f >> g)
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = parse_expression("x |> f >> g")
    {
        assert_eq!(op, BinaryOp::PipeApplyFwd);

        if let Expr::Literal(Literal::Identifier(name, _)) = left.as_ref() {
            assert_eq!(name, "x");
        } else {
            panic!("Expected left to be identifier x");
        }

        if let Expr::Binary {
            op: right_op,
            left: r_left,
            right: r_right,
            ..
        } = right.as_ref()
        {
            assert_eq!(*right_op, BinaryOp::ComposeRight);
            if let (
                Expr::Literal(Literal::Identifier(l, _)),
                Expr::Literal(Literal::Identifier(r, _)),
            ) = (r_left.as_ref(), r_right.as_ref())
            {
                assert_eq!(l, "f");
                assert_eq!(r, "g");
            } else {
                panic!("Expected identifiers f and g in composition");
            }
        } else {
            panic!("Expected right to be a composition expression");
        }
    } else {
        panic!("Expected pipe-apply expression");
    }

    // f << g |> h  => top-level |> with left side as (f << g)
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = parse_expression("f << g |> h")
    {
        assert_eq!(op, BinaryOp::PipeApplyFwd);

        if let Expr::Binary {
            op: left_op,
            left: l_left,
            right: l_right,
            ..
        } = left.as_ref()
        {
            assert_eq!(*left_op, BinaryOp::ComposeLeft);
            if let (
                Expr::Literal(Literal::Identifier(l, _)),
                Expr::Literal(Literal::Identifier(r, _)),
            ) = (l_left.as_ref(), l_right.as_ref())
            {
                assert_eq!(l, "f");
                assert_eq!(r, "g");
            } else {
                panic!("Expected identifiers f and g in composition");
            }
        } else {
            panic!("Expected left to be a composition expression");
        }

        if let Expr::Literal(Literal::Identifier(name, _)) = right.as_ref() {
            assert_eq!(name, "h");
        } else {
            panic!("Expected right to be identifier h");
        }
    } else {
        panic!("Expected pipe-apply expression");
    }
}

#[test]
fn test_composition_associativity_left() {
    // a >> b >> c  => (a >> b) >> c
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = parse_expression("a >> b >> c")
    {
        assert_eq!(op, BinaryOp::ComposeRight);
        // left should be a composition
        if let Expr::Binary {
            op: left_op,
            left: ll,
            right: lr,
            ..
        } = left.as_ref()
        {
            assert_eq!(*left_op, BinaryOp::ComposeRight);
            if let (
                Expr::Literal(Literal::Identifier(a, _)),
                Expr::Literal(Literal::Identifier(b, _)),
            ) = (ll.as_ref(), lr.as_ref())
            {
                assert_eq!(a, "a");
                assert_eq!(b, "b");
            } else {
                panic!("Expected identifiers a and b");
            }
        } else {
            panic!("Expected left to be a composition expression");
        }
        if let Expr::Literal(Literal::Identifier(c, _)) = right.as_ref() {
            assert_eq!(c, "c");
        } else {
            panic!("Expected right to be identifier c");
        }
    } else {
        panic!("Expected composition expression");
    }

    // a << b << c  => (a << b) << c
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = parse_expression("a << b << c")
    {
        assert_eq!(op, BinaryOp::ComposeLeft);
        if let Expr::Binary {
            op: left_op,
            left: ll,
            right: lr,
            ..
        } = left.as_ref()
        {
            assert_eq!(*left_op, BinaryOp::ComposeLeft);
            if let (
                Expr::Literal(Literal::Identifier(a, _)),
                Expr::Literal(Literal::Identifier(b, _)),
            ) = (ll.as_ref(), lr.as_ref())
            {
                assert_eq!(a, "a");
                assert_eq!(b, "b");
            } else {
                panic!("Expected identifiers a and b");
            }
        } else {
            panic!("Expected left to be a composition expression");
        }
        if let Expr::Literal(Literal::Identifier(c, _)) = right.as_ref() {
            assert_eq!(c, "c");
        } else {
            panic!("Expected right to be identifier c");
        }
    } else {
        panic!("Expected composition expression");
    }
}
