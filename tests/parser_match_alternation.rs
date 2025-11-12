use suji_ast::{Expr, Stmt};

mod common;
use common::parse_statement;

#[test]
fn test_match_alternation_expands_to_multiple_arms() {
    let src = "match x { 1 => \"ok\", 2 | 3 | 4 => \"ok\", _ => \"no\", }";
    let result = parse_statement(src);
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match {
        scrutinee, arms, ..
    })) = result
    {
        assert!(scrutinee.is_some());
        // 1 arm for 1, three for 2|3|4, one fallback => total 5 arms
        assert_eq!(arms.len(), 5);
    } else {
        panic!("Expected match expression statement");
    }
}

#[test]
fn test_regex_not_split_by_alternation_bar() {
    // Ensure /a|b/ remains a single regex pattern, not alternation split
    let src = "match s { /a|b/ => 1, _ => 0, }";
    let result = parse_statement(src);
    assert!(result.is_ok());
}
