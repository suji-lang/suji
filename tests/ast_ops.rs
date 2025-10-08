use suji_lang::ast::*;

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
        let _ = op;
        let _ = op == BinaryOp::Add;
    }
}

#[test]
fn test_unary_op_variants() {
    let ops = vec![UnaryOp::Negate, UnaryOp::Not];

    for op in ops {
        let _ = format!("{:?}", op);
        let _ = op;
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
