// Control flow evaluation modules
mod loops;
mod match_expr;

// Re-export the main control flow evaluation functions
pub use loops::{
    eval_infinite_loop, eval_infinite_loop_with_modules, eval_loop_through,
    eval_loop_through_with_modules,
};
pub use match_expr::{eval_match, eval_match_expression};
