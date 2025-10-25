mod assignments;
mod indexing;

pub use assignments::eval_assignment;
pub use indexing::{
    eval_index, eval_map_access_by_name, eval_map_access_by_name_with_modules, eval_slice,
};
