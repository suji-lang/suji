use super::StringPart;
use crate::span::Span;

/// Literal expression values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Numeric literal: 42, 3.14
    Number(String, Span),

    /// Boolean literal: true, false
    Boolean(bool, Span),

    /// Identifier: variable_name
    Identifier(String, Span),

    /// String template with interpolation: "Hello ${name}!"
    StringTemplate(Vec<StringPart>, Span),

    /// List literal: [1, 2, 3]
    List(Vec<super::Expr>, Span),

    /// Map literal: { key: value, ... }
    Map(Vec<(super::Expr, super::Expr)>, Span),

    /// Tuple literal: (a, b, c)
    Tuple(Vec<super::Expr>, Span),

    /// Regex literal: /pattern/
    RegexLiteral(String, Span),

    /// Null literal: nil
    Nil(Span),
}

impl Literal {
    /// Get the span of this literal
    pub fn span(&self) -> &Span {
        match self {
            Literal::Number(_, span) => span,
            Literal::Boolean(_, span) => span,
            Literal::Identifier(_, span) => span,
            Literal::StringTemplate(_, span) => span,
            Literal::List(_, span) => span,
            Literal::Map(_, span) => span,
            Literal::Tuple(_, span) => span,
            Literal::RegexLiteral(_, span) => span,
            Literal::Nil(span) => span,
        }
    }
}
