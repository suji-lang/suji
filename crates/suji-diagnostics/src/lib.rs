//! Diagnostics: rich error reporting utilities.
use suji_lexer::LexError;
use suji_parser::ParseError;
use suji_values::RuntimeError;

mod context;
mod emitters;
mod error_builder;
mod error_codes;
mod error_template;
mod lexer_errors;
mod parser_errors;
mod runtime_errors;
mod suggestions;

pub use context::DiagnosticContext;

/// Different kinds of diagnostics that can be reported
#[derive(Debug)]
pub enum DiagnosticKind {
    Lex(LexError),
    Parse(ParseError),
    Runtime(RuntimeError),
}

/// Print a diagnostic with enhanced formatting using ariadne
pub fn print_diagnostic(
    kind: DiagnosticKind,
    context: &DiagnosticContext,
) -> Result<(), Box<dyn std::error::Error>> {
    match kind {
        DiagnosticKind::Lex(error) => emitters::print_lex_error(error, context)?,
        DiagnosticKind::Parse(error) => emitters::print_parse_error(error, context)?,
        DiagnosticKind::Runtime(error) => emitters::print_runtime_error(error, context)?,
    }
    Ok(())
}
