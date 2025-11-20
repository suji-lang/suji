/// Loop variable bindings for `loop through` statements
#[derive(Debug, Clone, PartialEq)]
pub enum LoopBindings {
    None,                // loop through iterable { ... }
    One(String),         // loop through iterable with x { ... }
    Two(String, String), // loop through iterable with k, v { ... }
}
