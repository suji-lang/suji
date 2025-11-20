use super::StringPart;
use suji_lexer::Span;

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

    /// Check if this literal contains control flow expressions
    pub fn has_control_flow(&self) -> bool {
        match self {
            Literal::List(exprs, _) => exprs.iter().any(|e| e.has_control_flow()),
            Literal::Map(pairs, _) => pairs
                .iter()
                .any(|(k, v)| k.has_control_flow() || v.has_control_flow()),
            Literal::Tuple(exprs, _) => exprs.iter().any(|e| e.has_control_flow()),
            Literal::StringTemplate(parts, _) => parts
                .iter()
                .any(|part| matches!(part, StringPart::Expr(e) if e.has_control_flow())),
            _ => false,
        }
    }
}
