//! Diagnostics: rich error reporting utilities.

use suji_lexer::LexError;
use suji_parser::ParseError;
use suji_runtime::value::RuntimeError;
pub mod error_builder;
pub mod error_codes;
pub mod error_template;
pub mod lexer_errors;
pub mod parser_errors;
pub mod runtime_errors;

mod context;
pub(crate) mod emitters;
mod suggestions; // internal helpers used by runtime emitter

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
