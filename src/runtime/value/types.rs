use crate::ast::Stmt;
use indexmap::IndexMap;
use regex::Regex;
use std::rc::Rc;

/// Runtime values in the NN language
#[derive(Debug, Clone)]
pub enum Value {
    /// 64-bit floating point number
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Unicode string
    String(String),
    /// Growable list of values (pass-by-value)
    List(Vec<Value>),
    /// Key-value map with deterministic iteration
    Map(IndexMap<MapKey, Value>),
    /// Immutable tuple
    Tuple(Vec<Value>),
    /// Compiled regular expression
    Regex(Regex),
    /// Function with closure
    Function(FunctionValue),
    /// Nil value (absence of value)
    Nil,
}

/// Function value with parameters, body, and captured environment
#[derive(Clone)]
pub struct FunctionValue {
    /// Parameter specifications with optional defaults
    pub params: Vec<ParamSpec>,
    /// Function body AST
    pub body: Stmt,
    /// Captured closure environment
    pub env: Rc<super::super::env::Env>,
}

/// Runtime parameter specification
#[derive(Debug, Clone, PartialEq)]
pub struct ParamSpec {
    /// Parameter name
    pub name: String,
    /// Optional default value expression (evaluated at call time)
    pub default: Option<crate::ast::Expr>,
}

/// Wrapper for map keys that implements Hash and Eq for valid key types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapKey {
    Number(OrderedFloat),
    Boolean(bool),
    String(String),
    Tuple(Vec<MapKey>),
}

/// Wrapper for f64 that implements Eq and Hash for use as map keys
#[derive(Debug, Clone, Copy, PartialOrd)]
pub struct OrderedFloat(pub f64);

/// Control flow signals for break/continue/return
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    Return(Box<Value>),
    Break(Option<String>),    // Optional label
    Continue(Option<String>), // Optional label
}
