pub mod lexer;
pub use lexer::core::Lexer;
pub mod token {
    pub use crate::lexer::token::*;
}
pub use lexer::states::LexError;
