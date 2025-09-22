pub mod context;
pub mod traits;
pub mod types;

pub use context::ScannerContext;
pub use traits::{ScannerResult, StateScanner};
pub use types::{LexError, LexState, QuoteType};

pub mod interpolation;
pub mod normal;
pub mod regex;
pub mod shell;
pub mod string;
