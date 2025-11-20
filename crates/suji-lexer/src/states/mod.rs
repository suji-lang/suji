pub mod context;
pub use context::ScannerContext;

mod types;
pub use types::{LexState, QuoteType, ScannerResult};

pub mod interpolation;
pub mod normal;
pub mod regex;
pub mod shell;
pub mod string;
