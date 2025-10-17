use suji_lexer::LexError;

use crate::diagnostics::context::DiagnosticContext;
use crate::diagnostics::error_builder::ErrorBuilder;
use crate::diagnostics::lexer_errors;

pub(crate) fn print_lex_error(
    error: LexError,
    context: &DiagnosticContext,
) -> Result<(), Box<dyn std::error::Error>> {
    match error {
        LexError::UnterminatedString { span } => {
            let template = lexer_errors::unterminated_string();
            ErrorBuilder::new(template, context.clone()).print_with_span(span)?;
        }
        LexError::UnterminatedShellCommand { span } => {
            let template = lexer_errors::unterminated_shell_command();
            ErrorBuilder::new(template, context.clone()).print_with_span(span)?;
        }
        LexError::UnterminatedRegex { span } => {
            let template = lexer_errors::unterminated_regex();
            ErrorBuilder::new(template, context.clone()).print_with_span(span)?;
        }
        LexError::InvalidEscape { escape, span } => {
            let template = lexer_errors::invalid_escape(escape);
            ErrorBuilder::new(template, context.clone()).print_with_span(span)?;
        }
        LexError::InvalidNumber { literal, span } => {
            let template = lexer_errors::invalid_number(&literal);
            ErrorBuilder::new(template, context.clone()).print_with_span(span)?;
        }
        LexError::UnexpectedCharacter { ch, span } => {
            let template = lexer_errors::unexpected_character(ch);
            ErrorBuilder::new(template, context.clone()).print_with_span(span)?;
        }
    }
    Ok(())
}
