use suji_lexer::LexError;

use crate::diagnostics::context::DiagnosticContext;
use crate::diagnostics::error_builder::{ErrorBuilder, line_column_to_range};
use crate::diagnostics::lexer_errors;

pub(crate) fn print_lex_error(
    error: LexError,
    context: &DiagnosticContext,
) -> Result<(), Box<dyn std::error::Error>> {
    match error {
        LexError::UnterminatedString { line, column } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = lexer_errors::unterminated_string();
            ErrorBuilder::new(template, context.clone()).print_with_range(range)?;
        }
        LexError::UnterminatedShellCommand { line, column } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = lexer_errors::unterminated_shell_command();
            ErrorBuilder::new(template, context.clone()).print_with_range(range)?;
        }
        LexError::UnterminatedRegex { line, column } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = lexer_errors::unterminated_regex();
            ErrorBuilder::new(template, context.clone()).print_with_range(range)?;
        }
        LexError::InvalidEscape {
            escape,
            line,
            column,
        } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = lexer_errors::invalid_escape(escape);
            ErrorBuilder::new(template, context.clone()).print_with_range(range)?;
        }
        LexError::InvalidNumber {
            literal,
            line,
            column,
        } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = lexer_errors::invalid_number(&literal);
            ErrorBuilder::new(template, context.clone()).print_with_range(range)?;
        }
        LexError::UnexpectedCharacter { ch, line, column } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = lexer_errors::unexpected_character(ch);
            ErrorBuilder::new(template, context.clone()).print_with_range(range)?;
        }
    }
    Ok(())
}
