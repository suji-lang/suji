use crate::ast::Stmt;
use indexmap::IndexMap;
use regex::Regex;
use std::cell::{Cell, RefCell};
use std::io::BufReader;
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
    /// Stream for I/O operations
    Stream(Rc<StreamHandle>),
    /// Environment variables map
    EnvMap(Rc<EnvProxy>),
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

    /// Check if this stream is readable
    pub fn is_readable(&self) -> bool {
        matches!(
            self.backend,
            StreamBackend::Stdin(_) | StreamBackend::File(_)
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
            StreamBackend::Stdout(_) | StreamBackend::Stderr(_) | StreamBackend::File(_)
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

/// Proxy for environment variable operations
///
/// This struct provides a map-like interface to process environment variables.
/// All operations delegate to std::env functions and affect the actual process
/// environment, which means changes are visible to subprocesses.
#[derive(Debug)]
pub struct EnvProxy;

impl Default for EnvProxy {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvProxy {
    /// Create a new environment proxy
    pub fn new() -> Self {
        Self
    }

    /// Get an environment variable value
    pub fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    /// Set an environment variable
    pub fn set(&self, key: &str, value: &str) -> Result<(), super::RuntimeError> {
        if key.is_empty() {
            return Err(super::RuntimeError::InvalidOperation {
                message: "Environment variable key cannot be empty".to_string(),
            });
        }
        // SAFETY: We validate that key is not empty, which prevents undefined behavior
        // from setting environment variables with empty keys. The key and value are
        // valid UTF-8 strings, which is required by set_var.
        unsafe {
            std::env::set_var(key, value);
        }
        Ok(())
    }

    /// Delete an environment variable
    pub fn delete(&self, key: &str) -> bool {
        if std::env::var(key).is_ok() {
            // SAFETY: We check that the variable exists before removing it, and
            // the key is a valid UTF-8 string, which is required by remove_var.
            unsafe {
                std::env::remove_var(key);
            }
            true
        } else {
            false
        }
    }

    /// Check if an environment variable exists
    pub fn contains(&self, key: &str) -> bool {
        std::env::var(key).is_ok()
    }

    /// Get all environment variable keys
    pub fn keys(&self) -> Vec<String> {
        std::env::vars().map(|(k, _)| k).collect()
    }

    /// Get all environment variable values
    pub fn values(&self) -> Vec<String> {
        std::env::vars().map(|(_, v)| v).collect()
    }

    /// Get all environment variables as key-value pairs
    pub fn to_list(&self) -> Vec<(String, String)> {
        std::env::vars().collect()
    }

    /// Get the number of environment variables
    pub fn length(&self) -> usize {
        std::env::vars().count()
    }
}
