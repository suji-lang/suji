//! Thread-local IO context for dynamic stream redirection in pipe expressions

use super::value::StreamHandle;
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    /// Thread-local IO context for stream redirection
    static IO_CONTEXT: RefCell<IoContext> = RefCell::new(IoContext::default());
}

/// IO context holding optional stream overrides for stdin, stdout, stderr
#[derive(Clone, Debug, Default)]
pub struct IoContext {
    /// Overridden stdin (None = use default)
    pub stdin: Option<Rc<StreamHandle>>,
    /// Overridden stdout (None = use default)
    pub stdout: Option<Rc<StreamHandle>>,
    /// Overridden stderr (None = use default)
    pub stderr: Option<Rc<StreamHandle>>,
}

impl IoContext {
    /// Get the effective stdin (context override or default)
    pub fn effective_stdin() -> Rc<StreamHandle> {
        IO_CONTEXT.with(|ctx| {
            ctx.borrow()
                .stdin
                .clone()
                .unwrap_or_else(|| Rc::new(StreamHandle::new_stdin()))
        })
    }

    /// Get the effective stdout (context override or default)
    pub fn effective_stdout() -> Rc<StreamHandle> {
        IO_CONTEXT.with(|ctx| {
            ctx.borrow()
                .stdout
                .clone()
                .unwrap_or_else(|| Rc::new(StreamHandle::new_stdout()))
        })
    }

    /// Get the effective stderr (context override or default)
    pub fn effective_stderr() -> Rc<StreamHandle> {
        IO_CONTEXT.with(|ctx| {
            ctx.borrow()
                .stderr
                .clone()
                .unwrap_or_else(|| Rc::new(StreamHandle::new_stderr()))
        })
    }

    /// Execute a function with IO stream overrides
    ///
    /// This temporarily overrides the IO context for the duration of the function call,
    /// then restores the previous context. This allows pipe expressions to redirect
    /// IO streams for nested function calls.
    ///
    /// # Arguments
    ///
    /// * `stdin` - Optional stdin override
    /// * `stdout` - Optional stdout override
    /// * `stderr` - Optional stderr override
    /// * `f` - Function to execute with the overridden context
    ///
    /// # Returns
    ///
    /// The result of calling `f`
    pub fn with_overrides<F, R>(
        stdin: Option<Rc<StreamHandle>>,
        stdout: Option<Rc<StreamHandle>>,
        stderr: Option<Rc<StreamHandle>>,
        f: F,
    ) -> R
    where
        F: FnOnce() -> R,
    {
        // Save current context
        let saved = IO_CONTEXT.with(|ctx| ctx.borrow().clone());

        // Apply overrides (only set non-None values)
        IO_CONTEXT.with(|ctx| {
            let mut context = ctx.borrow_mut();
            if let Some(s) = stdin {
                context.stdin = Some(s);
            }
            if let Some(s) = stdout {
                context.stdout = Some(s);
            }
            if let Some(s) = stderr {
                context.stderr = Some(s);
            }
        });

        // Execute function with overrides
        let result = f();

        // Restore original context
        IO_CONTEXT.with(|ctx| *ctx.borrow_mut() = saved);

        result
    }

    /// Clear all overrides (reset to defaults)
    ///
    /// This is mainly useful for testing or cleanup scenarios.
    pub fn clear() {
        IO_CONTEXT.with(|ctx| *ctx.borrow_mut() = IoContext::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_streams() {
        IoContext::clear();

        // Default streams should be the standard system streams
        let stdin = IoContext::effective_stdin();
        let stdout = IoContext::effective_stdout();
        let stderr = IoContext::effective_stderr();

        // Just verify they're created without panicking
        assert!(Rc::strong_count(&stdin) >= 1);
        assert!(Rc::strong_count(&stdout) >= 1);
        assert!(Rc::strong_count(&stderr) >= 1);
    }

    #[test]
    fn test_stdout_override() {
        IoContext::clear();

        let captured = Rc::new(StreamHandle::new_memory_writable());
        let captured_clone = captured.clone();

        IoContext::with_overrides(None, Some(captured), None, || {
            let stdout = IoContext::effective_stdout();
            // Should be our captured stream
            assert!(Rc::ptr_eq(&stdout, &captured_clone));
        });

        // After the block, should be back to default
        let stdout = IoContext::effective_stdout();
        assert!(!Rc::ptr_eq(&stdout, &captured_clone));
    }

    #[test]
    fn test_nested_overrides() {
        IoContext::clear();

        let outer_stdout = Rc::new(StreamHandle::new_memory_writable());
        let inner_stdout = Rc::new(StreamHandle::new_memory_writable());

        IoContext::with_overrides(None, Some(outer_stdout.clone()), None, || {
            let stdout1 = IoContext::effective_stdout();
            assert!(Rc::ptr_eq(&stdout1, &outer_stdout));

            IoContext::with_overrides(None, Some(inner_stdout.clone()), None, || {
                let stdout2 = IoContext::effective_stdout();
                assert!(Rc::ptr_eq(&stdout2, &inner_stdout));
            });

            // After inner block, should be back to outer
            let stdout3 = IoContext::effective_stdout();
            assert!(Rc::ptr_eq(&stdout3, &outer_stdout));
        });
    }

    #[test]
    fn test_stdin_override() {
        IoContext::clear();

        let test_data = b"test input".to_vec();
        let captured = Rc::new(StreamHandle::new_memory_readable(test_data));
        let captured_clone = captured.clone();

        IoContext::with_overrides(Some(captured), None, None, || {
            let stdin = IoContext::effective_stdin();
            assert!(Rc::ptr_eq(&stdin, &captured_clone));
        });
    }
}
