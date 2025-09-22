use nnlang::ast::*;

#[test]
fn test_binary_op_variants() {
    // Test that all binary operators are represented
    let ops = vec![
        BinaryOp::Add,
        BinaryOp::Subtract,
        BinaryOp::Multiply,
        BinaryOp::Divide,
        BinaryOp::Modulo,
        BinaryOp::Power,
        BinaryOp::Equal,
        BinaryOp::NotEqual,
        BinaryOp::Less,
        BinaryOp::LessEqual,
        BinaryOp::Greater,
        BinaryOp::GreaterEqual,
        BinaryOp::And,
        BinaryOp::Or,
        BinaryOp::Range,
        BinaryOp::RegexMatch,
        BinaryOp::RegexNotMatch,
    ];

    // Ensure all operators implement required traits
    for op in ops {
        let _ = format!("{:?}", op);
        let _ = op.clone();
        let _ = op == BinaryOp::Add;
    }
}

#[test]
fn test_unary_op_variants() {
    let ops = vec![UnaryOp::Negate, UnaryOp::Not];

    for op in ops {
        let _ = format!("{:?}", op);
        let _ = op.clone();
        let _ = op == UnaryOp::Negate;
    }
}

#[test]
fn test_compound_op_variants() {
    // Test that all compound assignment operators are represented
    let ops = vec![
        CompoundOp::PlusAssign,
        CompoundOp::MinusAssign,
        CompoundOp::MultiplyAssign,
        CompoundOp::DivideAssign,
        CompoundOp::ModuloAssign,
    ];

    // Ensure all operators implement required traits
    for op in ops {
        let _ = format!("{:?}", op);
        let _ = op;
        let _ = op == CompoundOp::PlusAssign;
    }
}

#[test]
fn test_compound_op_from_token() {
    use nnlang::token::Token;

    // Test valid token conversions
    assert_eq!(
        CompoundOp::from_token(&Token::PlusAssign),
        Some(CompoundOp::PlusAssign)
    );
    assert_eq!(
        CompoundOp::from_token(&Token::MinusAssign),
        Some(CompoundOp::MinusAssign)
    );
    assert_eq!(
        CompoundOp::from_token(&Token::MultiplyAssign),
        Some(CompoundOp::MultiplyAssign)
    );
    assert_eq!(
        CompoundOp::from_token(&Token::DivideAssign),
        Some(CompoundOp::DivideAssign)
    );
    assert_eq!(
        CompoundOp::from_token(&Token::ModuloAssign),
        Some(CompoundOp::ModuloAssign)
    );

    // Test invalid token conversions
    assert_eq!(CompoundOp::from_token(&Token::Assign), None);
    assert_eq!(CompoundOp::from_token(&Token::Plus), None);
    assert_eq!(
        CompoundOp::from_token(&Token::Identifier("test".to_string())),
        None
    );
}
