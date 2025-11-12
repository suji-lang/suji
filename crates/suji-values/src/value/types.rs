use indexmap::IndexMap;
use regex::Regex;
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use rust_decimal::prelude::*;
use std::cell::{Cell, RefCell};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::ops::Neg;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use suji_ast::ast::Stmt;

use super::super::env_overlay::EnvProxy;

/// Runtime values in the SUJI language
#[derive(Debug, Clone)]
pub enum Value {
    /// Number value
    Number(DecimalNumber),
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
    /// Stream for I/O operations
    Stream(Rc<StreamHandle>),
    /// Dynamic IO stream proxy (resolved at access time via IO context)
    StreamProxy(StreamProxyKind),
    /// Environment variables map
    EnvMap(Rc<EnvProxy>),
    /// Lazily-loaded module that loads on first access
    Module(ModuleHandle),
    /// Nil value (absence of value)
    Nil,
}

/// Handle to a lazily-loaded module.
#[derive(Debug, Clone)]
pub struct ModuleHandle {
    /// Unique identifier for this module (e.g., "std:random", "myproject:utils")
    pub module_path: String,

    /// Path segments for resolution (e.g., ["std", "random"])
    pub segments: Vec<String>,

    /// Source code for virtual modules (None for filesystem modules)
    pub source: Option<&'static str>,

    /// Loaded module value (cached after first load)
    pub loaded: Rc<RefCell<Option<Box<Value>>>>,

    /// Registry reference for loading (stored as raw pointer to avoid circular dependency)
    /// We use *const instead of Weak because ModuleRegistry isn't behind Rc yet
    pub registry_ptr: *const (),
}

impl ModuleHandle {
    /// Create a new module handle
    pub fn new(
        module_path: String,
        segments: Vec<String>,
        source: Option<&'static str>,
        registry_ptr: *const (),
    ) -> Self {
        Self {
            module_path,
            segments,
            source,
            loaded: Rc::new(RefCell::new(None)),
            registry_ptr,
        }
    }

    /// Check if this module has been loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded.borrow().is_some()
    }
}

/// Kind of IO stream proxy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamProxyKind {
    /// Proxy for stdin (resolves dynamically)
    Stdin,
    /// Proxy for stdout (resolves dynamically)
    Stdout,
    /// Proxy for stderr (resolves dynamically)
    Stderr,
}

/// Decimal number wrapper for precise base-10 arithmetic
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DecimalNumber(pub Decimal);

impl DecimalNumber {
    pub fn parse(s: &str) -> Result<Self, rust_decimal::Error> {
        Decimal::from_str(s).map(DecimalNumber)
    }

    pub fn from_i64(n: i64) -> Self {
        DecimalNumber(Decimal::from(n))
    }

    /// Create a decimal from an unsigned integer
    pub fn from_u64(n: u64) -> Self {
        DecimalNumber(Decimal::from(n))
    }

    /// Create a decimal from a usize
    pub fn from_usize(n: usize) -> Self {
        DecimalNumber(Decimal::from(n))
    }

    /// Check if this decimal represents an integer (no fractional part)
    pub fn is_integer(&self) -> bool {
        self.0.fract() == Decimal::ZERO
    }

    /// Convert to i64 if possible (integer and within range)
    pub fn to_i64_checked(&self) -> Option<i64> {
        if self.is_integer() {
            self.0.to_i64()
        } else {
            None
        }
    }

    /// Get the underlying decimal
    pub fn inner(&self) -> Decimal {
        self.0
    }

    /// Arithmetic operations
    pub fn add(&self, other: &DecimalNumber) -> DecimalNumber {
        DecimalNumber(self.0 + other.0)
    }

    pub fn sub(&self, other: &DecimalNumber) -> DecimalNumber {
        DecimalNumber(self.0 - other.0)
    }

    pub fn mul(&self, other: &DecimalNumber) -> DecimalNumber {
        DecimalNumber(self.0 * other.0)
    }

    pub fn div(&self, other: &DecimalNumber) -> Result<DecimalNumber, &'static str> {
        if other.0 == Decimal::ZERO {
            Err("Division by zero")
        } else {
            Ok(DecimalNumber(self.0 / other.0))
        }
    }

    pub fn rem(&self, other: &DecimalNumber) -> Result<DecimalNumber, &'static str> {
        if other.0 == Decimal::ZERO {
            Err("Modulo by zero")
        } else {
            Ok(DecimalNumber(self.0 % other.0))
        }
    }

    /// Power operation (integer exponent only for determinism)
    pub fn pow(&self, exponent: &DecimalNumber) -> Result<DecimalNumber, &'static str> {
        if !exponent.is_integer() {
            return Err("Power exponent must be an integer");
        }

        let exp = exponent.to_i64_checked().ok_or("Exponent too large")?;
        if exp < 0 {
            return Err("Negative exponents not supported");
        }

        // Manual power implementation using repeated multiplication
        let mut result = Decimal::ONE;
        let mut base = self.0;
        let mut exp = exp as u64;

        // Fast exponentiation by squaring
        while exp > 0 {
            if exp % 2 == 1 {
                result *= base;
            }
            base *= base;
            exp /= 2;
        }

        Ok(DecimalNumber(result))
    }

    /// Mathematical functions
    pub fn abs(&self) -> DecimalNumber {
        DecimalNumber(self.0.abs())
    }

    pub fn ceil(&self) -> DecimalNumber {
        DecimalNumber(self.0.ceil())
    }

    pub fn floor(&self) -> DecimalNumber {
        DecimalNumber(self.0.floor())
    }

    pub fn round(&self) -> DecimalNumber {
        DecimalNumber(
            self.0
                .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero),
        )
    }

    /// Square root using Newton's method with fixed precision
    pub fn sqrt(&self) -> Result<DecimalNumber, &'static str> {
        if self.0 < Decimal::ZERO {
            return Err("Square root of negative number");
        }

        if self.0 == Decimal::ZERO {
            return Ok(DecimalNumber(Decimal::ZERO));
        }

        // Newton's method: x_{n+1} = (x_n + a/x_n) / 2
        let mut x = self.0;
        let two = Decimal::from(2);

        // Iterate until convergence (max 50 iterations for safety)
        for _ in 0..50 {
            let x_next = (x + self.0 / x) / two;
            if (x - x_next).abs() < Decimal::new(1, 28) {
                // Very small epsilon
                break;
            }
            x = x_next;
        }

        Ok(DecimalNumber(x))
    }

    pub fn min(&self, other: &DecimalNumber) -> DecimalNumber {
        DecimalNumber(self.0.min(other.0))
    }

    pub fn max(&self, other: &DecimalNumber) -> DecimalNumber {
        DecimalNumber(self.0.max(other.0))
    }
}

impl FromStr for DecimalNumber {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Decimal::from_str(s).map(DecimalNumber)
    }
}

impl fmt::Display for DecimalNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display integers without decimal point
        if self.is_integer() {
            write!(f, "{}", self.0.trunc())
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl Neg for DecimalNumber {
    type Output = DecimalNumber;

    fn neg(self) -> Self::Output {
        DecimalNumber(-self.0)
    }
}

/// Wrapper for Decimal that implements Eq and Hash for use as map keys
#[derive(Debug, Clone)]
pub struct OrderedDecimal(pub Decimal);

impl OrderedDecimal {
    pub fn new(decimal: Decimal) -> Self {
        OrderedDecimal(decimal)
    }

    /// Normalize the decimal for consistent hashing and equality
    fn normalize(&self) -> Decimal {
        self.0.normalize()
    }
}

impl PartialEq for OrderedDecimal {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for OrderedDecimal {}

impl PartialOrd for OrderedDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedDecimal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for OrderedDecimal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the normalized decimal to ensure 1.0 and 1.00 hash the same
        let normalized = self.normalize();
        normalized.serialize().hash(state);
    }
}

/// Body of a function (supports multiple representations)
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    /// AST representation (used by interpreter)
    Ast(Stmt),
    /// Bytecode representation (used by VM)
    #[allow(dead_code)]
    Bytecode(Arc<BytecodeFunction>),
    /// Builtin function reference
    Builtin(&'static str),
}

/// Placeholder for bytecode representation
#[derive(Debug, Clone, PartialEq)]
pub struct BytecodeFunction {
    pub instructions: Vec<u8>,
    // To be defined in suji-runtime-vm
}

/// Function value with parameters, body, and captured environment
#[derive(Clone)]
pub struct FunctionValue {
    /// Parameter specifications with optional defaults
    pub params: Vec<ParamSpec>,
    /// Function body (AST, bytecode, or builtin reference)
    pub body: FunctionBody,
    /// Captured closure environment
    pub env: Rc<super::super::env::Env>,
}

impl FunctionValue {
    /// Helper for interpreter to get AST body
    pub fn as_ast_body(&self) -> Result<&Stmt, super::RuntimeError> {
        match &self.body {
            FunctionBody::Ast(stmt) => Ok(stmt),
            FunctionBody::Bytecode(_) => Err(super::RuntimeError::InvalidOperation {
                message: "Cannot execute bytecode function in AST interpreter".to_string(),
            }),
            FunctionBody::Builtin(_) => Err(super::RuntimeError::InvalidOperation {
                message: "Cannot get AST body from builtin function".to_string(),
            }),
        }
    }
}

/// Runtime parameter specification
#[derive(Debug, Clone, PartialEq)]
pub struct ParamSpec {
    /// Parameter name
    pub name: String,
    /// Optional default value expression (evaluated at call time)
    pub default: Option<suji_ast::ast::Expr>,
}

/// Wrapper for map keys that implements Hash and Eq for valid key types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapKey {
    Number(OrderedDecimal),
    Boolean(bool),
    String(String),
    Tuple(Vec<MapKey>),
}

/// Control flow signals for break/continue/return
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    Return(Box<Value>),
    Break(Option<String>),    // Optional label
    Continue(Option<String>), // Optional label
}

/// Backend for stream I/O operations
#[derive(Debug)]
pub enum StreamBackend {
    /// Standard input with buffered reader
    Stdin(RefCell<BufReader<std::io::Stdin>>),
    /// Standard output
    Stdout(RefCell<std::io::Stdout>),
    /// Standard error
    Stderr(RefCell<std::io::Stderr>),
    /// File handle (future-proofing)
    File(RefCell<std::fs::File>),
    /// Test backend with in-memory content (for testing only)
    #[cfg(test)]
    TestReadable(RefCell<std::io::Cursor<Vec<u8>>>),
    /// Test backend that captures writes (for testing only)
    #[cfg(test)]
    TestWritable(RefCell<Vec<u8>>),
    /// In-memory readable stream (general purpose)
    MemoryReadable(RefCell<std::io::Cursor<Vec<u8>>>),
    /// In-memory writable stream (general purpose)
    MemoryWritable(RefCell<Vec<u8>>),
}

/// Handle for stream I/O operations
#[derive(Debug)]
pub struct StreamHandle {
    /// The underlying I/O backend
    pub backend: StreamBackend,
    /// Whether the stream has been closed
    pub is_closed: Cell<bool>,
    /// Display name for the stream
    pub name: String,
}

impl StreamHandle {
    /// Create a new stdin stream handle
    pub fn new_stdin() -> Self {
        Self {
            backend: StreamBackend::Stdin(RefCell::new(BufReader::new(std::io::stdin()))),
            is_closed: Cell::new(false),
            name: "stdin".to_string(),
        }
    }

    /// Create a new stdout stream handle
    pub fn new_stdout() -> Self {
        Self {
            backend: StreamBackend::Stdout(RefCell::new(std::io::stdout())),
            is_closed: Cell::new(false),
            name: "stdout".to_string(),
        }
    }

    /// Create a new stderr stream handle
    pub fn new_stderr() -> Self {
        Self {
            backend: StreamBackend::Stderr(RefCell::new(std::io::stderr())),
            is_closed: Cell::new(false),
            name: "stderr".to_string(),
        }
    }

    /// Create a test readable stream with the given content (for testing only)
    #[cfg(test)]
    pub fn new_test_readable(content: &str) -> Self {
        Self {
            backend: StreamBackend::TestReadable(RefCell::new(std::io::Cursor::new(
                content.as_bytes().to_vec(),
            ))),
            is_closed: Cell::new(false),
            name: "test_readable".to_string(),
        }
    }

    /// Create a test writable stream that captures output (for testing only)
    #[cfg(test)]
    pub fn new_test_writable() -> Self {
        Self {
            backend: StreamBackend::TestWritable(RefCell::new(Vec::new())),
            is_closed: Cell::new(false),
            name: "test_writable".to_string(),
        }
    }

    /// Get the captured content from a test writable stream (for testing only)
    #[cfg(test)]
    pub fn get_test_output(&self) -> Option<String> {
        match &self.backend {
            StreamBackend::TestWritable(buffer) => String::from_utf8(buffer.borrow().clone()).ok(),
            _ => None,
        }
    }

    /// Create a new general-purpose memory readable stream from bytes
    pub fn new_memory_readable(bytes: Vec<u8>) -> Self {
        Self {
            backend: StreamBackend::MemoryReadable(RefCell::new(std::io::Cursor::new(bytes))),
            is_closed: Cell::new(false),
            name: "mem_read".to_string(),
        }
    }

    /// Create a new general-purpose memory writable stream
    pub fn new_memory_writable() -> Self {
        Self {
            backend: StreamBackend::MemoryWritable(RefCell::new(Vec::new())),
            is_closed: Cell::new(false),
            name: "mem_write".to_string(),
        }
    }

    /// Take the captured bytes from a memory writable stream, if applicable
    pub fn take_memory_output(&self) -> Option<Vec<u8>> {
        match &self.backend {
            StreamBackend::MemoryWritable(buffer) => {
                let mut buf = buffer.borrow_mut();
                let out = std::mem::take(&mut *buf);
                Some(out)
            }
            _ => None,
        }
    }

    /// Check if this stream is readable
    pub fn is_readable(&self) -> bool {
        matches!(
            self.backend,
            StreamBackend::Stdin(_) | StreamBackend::File(_) | StreamBackend::MemoryReadable(_)
        ) || {
            #[cfg(test)]
            {
                matches!(self.backend, StreamBackend::TestReadable(_))
            }
            #[cfg(not(test))]
            {
                false
            }
        }
    }

    /// Check if this stream is writable
    pub fn is_writable(&self) -> bool {
        matches!(
            self.backend,
            StreamBackend::Stdout(_)
                | StreamBackend::Stderr(_)
                | StreamBackend::File(_)
                | StreamBackend::MemoryWritable(_)
        ) || {
            #[cfg(test)]
            {
                matches!(self.backend, StreamBackend::TestWritable(_))
            }
            #[cfg(not(test))]
            {
                false
            }
        }
    }
}
