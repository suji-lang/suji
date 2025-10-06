// Data structure evaluation modules
mod assignments;
mod indexing;

// Re-export the main data structure evaluation functions
pub use assignments::eval_assignment;
pub use indexing::{eval_index, eval_map_access_by_name, eval_slice};
