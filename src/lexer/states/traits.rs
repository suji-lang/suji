use crate::lexer::states::types::{LexError, LexState};
use crate::lexer::token::TokenWithSpan;

/// Result type for state scanner operations
pub type ScannerResult = Result<TokenWithSpan, LexError>;

/// Trait for state scanners
pub trait StateScanner {
    fn scan(
        &mut self,
        context: &mut crate::lexer::states::context::ScannerContext,
        state: &LexState,
    ) -> ScannerResult;
}
