/// Literal values that can be used in patterns
#[derive(Debug, Clone, PartialEq)]
pub enum ValueLike {
    Number(f64),
    Boolean(bool),
    String(String),
    Tuple(Vec<ValueLike>),
    Nil,
}
