use super::common::parse_expression;
use suji_ast::{BinaryOp, Expr};

#[test]
fn stream_pipe_binds_tighter_than_apply_forward() {
    let src = "producer() | `grep ba` |> format |> println";
    let expr = parse_expression(src).expect("parse ok");

    // Expect: ((producer() | `grep ba`) |> format) |> println
    match expr {
        Expr::Binary {
            op: top_op,
            left: top_left,
            ..
        } => {
            assert_eq!(top_op, BinaryOp::PipeApplyFwd);
            match *top_left {
                Expr::Binary {
                    op: mid_op,
                    left: mid_left,
                    ..
                } => {
                    assert_eq!(mid_op, BinaryOp::PipeApplyFwd);
                    match *mid_left {
                        Expr::Binary { op: inner_op, .. } => {
                            assert_eq!(inner_op, BinaryOp::Pipe);
                        }
                        _ => panic!("Expected inner left to be a Pipe operator"),
                    }
                }
                _ => panic!("Expected left to be PipeApplyFwd"),
            }
        }
        _ => panic!("Expected top-level PipeApplyFwd"),
    }
}

#[test]
fn stream_pipe_binds_tighter_than_apply_backward() {
    let src = "println <| format <| (producer() | `grep ba`)";
    let expr = parse_expression(src).expect("parse ok");

    // Expect: println <| (format <| (producer() | `grep ba`))
    match expr {
        Expr::Binary {
            op: top_op,
            right: top_right,
            ..
        } => {
            assert_eq!(top_op, BinaryOp::PipeApplyBwd);
            match *top_right {
                Expr::Binary {
                    op: mid_op,
                    right: mid_right,
                    ..
                } => {
                    assert_eq!(mid_op, BinaryOp::PipeApplyBwd);
                    match *mid_right {
                        Expr::Binary { op: inner_op, .. } => {
                            assert_eq!(inner_op, BinaryOp::Pipe);
                        }
                        Expr::Grouping { expr, .. } => match *expr {
                            Expr::Binary { op: inner_op, .. } => {
                                assert_eq!(inner_op, BinaryOp::Pipe);
                            }
                            _ => panic!("Expected grouping to contain a Pipe operator"),
                        },
                        _ => panic!("Expected inner right to be a Pipe operator or grouped Pipe"),
                    }
                }
                _ => panic!("Expected right to be PipeApplyBwd"),
            }
        }
        _ => panic!("Expected top-level PipeApplyBwd"),
    }
}
