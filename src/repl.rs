use crate::diagnostics::{DiagnosticContext, DiagnosticKind, print_diagnostic};
use crate::parser::{ParseError, parse_program};
use crate::runtime::builtins::setup_global_env;
use crate::runtime::env::Env;
use crate::runtime::eval::eval_program_with_modules;
use crate::runtime::module::ModuleRegistry;
use crate::runtime::value::{RuntimeError, Value};
use ariadne::{Color, Fmt};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};
use std::rc::Rc;

/// REPL state and configuration
pub struct Repl {
    editor: DefaultEditor,
    env: Rc<Env>,
    module_registry: ModuleRegistry,
    input_buffer: String,
    line_number: usize,
}

impl Repl {
    /// Create a new REPL instance
    pub fn new() -> RustylineResult<Self> {
        let editor = DefaultEditor::new()?;

        // Set up global environment with built-ins
        let env = Rc::new(Env::new());
        setup_global_env(&env);

        Ok(Repl {
            editor,
            env,
            module_registry: ModuleRegistry::new(),
            input_buffer: String::new(),
            line_number: 1,
        })
    }

    /// Run the REPL loop
    pub fn run(&mut self) -> RustylineResult<()> {
        self.print_welcome();

        loop {
            let prompt = if self.input_buffer.is_empty() {
                "nn> "
            } else {
                "  > "
            };

            match self.editor.readline(prompt) {
                Ok(line) => {
                    // Handle special commands
                    let trimmed = line.trim();
                    if trimmed == ":quit" || trimmed == ":exit" {
                        println!("Goodbye!");
                        break;
                    }
                    if trimmed == ":help" {
                        self.print_help();
                        continue;
                    }

                    // Add line to input buffer
                    if !self.input_buffer.is_empty() {
                        self.input_buffer.push('\n');
                    }
                    self.input_buffer.push_str(&line);

                    // Check if input is complete
                    if self.is_input_complete(&self.input_buffer) {
                        // Add to history
                        self.editor.add_history_entry(&self.input_buffer)?;

                        // Process the complete input
                        self.process_input();

                        // Reset for next input
                        self.input_buffer.clear();
                        self.line_number = 1;
                    } else {
                        self.line_number += 1;
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C - clear current input and continue
                    self.input_buffer.clear();
                    self.line_number = 1;
                    println!("^C");
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D - exit REPL
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error reading input: {}", err);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Check if the current input is syntactically complete
    fn is_input_complete(&self, input: &str) -> bool {
        let mut brace_count = 0;
        let mut paren_count = 0;
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut in_shell = false;
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' if !in_string && !in_shell => in_string = true,
                '"' if in_string => in_string = false,
                '`' if !in_string && !in_shell => in_shell = true,
                '`' if in_shell => in_shell = false,
                '{' if !in_string && !in_shell => brace_count += 1,
                '}' if !in_string && !in_shell => brace_count -= 1,
                '(' if !in_string && !in_shell => paren_count += 1,
                ')' if !in_string && !in_shell => paren_count -= 1,
                '[' if !in_string && !in_shell => bracket_count += 1,
                ']' if !in_string && !in_shell => bracket_count -= 1,
                '\\' if in_string || in_shell => {
                    // Skip escaped character
                    chars.next();
                }
                _ => {}
            }
        }

        // Input is complete if all brackets are balanced and we're not in a string/shell
        brace_count == 0 && paren_count == 0 && bracket_count == 0 && !in_string && !in_shell
    }

    /// Process a complete input and evaluate it
    fn process_input(&mut self) {
        let input = self.input_buffer.trim();

        // Skip empty input
        if input.is_empty() {
            return;
        }

        // Parse the input
        match parse_program(input) {
            Ok(statements) => {
                // Evaluate the statements
                match eval_program_with_modules(
                    &statements,
                    self.env.clone(),
                    &self.module_registry,
                ) {
                    Ok(Some(value)) => {
                        // Print the result if it's not null
                        if !matches!(value, Value::Null) {
                            println!("{}", value);
                        }
                    }
                    Ok(None) => {
                        // Program completed without a value - nothing to print
                    }
                    Err(e) => {
                        self.print_runtime_error(&e, input);
                    }
                }
            }
            Err(e) => {
                self.print_parse_error(&e, input);
            }
        }
    }

    /// Print a parse error with enhanced formatting
    fn print_parse_error(&self, error: &ParseError, input: &str) {
        let context = DiagnosticContext::new(input.to_string());
        if let Err(_) = print_diagnostic(DiagnosticKind::Parse((*error).clone()), &context) {
            // Fallback to simple error if diagnostics fail
            println!("{}: {}", "Parse Error".fg(Color::Red), error);
        }
    }

    /// Print a runtime error with enhanced formatting  
    fn print_runtime_error(&self, error: &RuntimeError, input: &str) {
        let context = DiagnosticContext::new(input.to_string());
        if let Err(_) = print_diagnostic(DiagnosticKind::Runtime((*error).clone()), &context) {
            // Fallback to simple error if diagnostics fail
            println!("{}: {}", "Runtime Error".fg(Color::Red), error);
        }
    }

    /// Print welcome message
    fn print_welcome(&self) {
        println!("{}", "NN Language REPL".fg(Color::Cyan));
        println!("Type expressions to evaluate them, or :help for commands");
        println!("Use Ctrl+C to cancel current input, Ctrl+D or :quit to exit");
        println!();
    }

    /// Print help message
    fn print_help(&self) {
        println!("Available commands:");
        println!("  :help      - Show this help message");
        println!("  :quit      - Exit the REPL");
        println!("  :exit      - Exit the REPL");
        println!();
        println!("Tips:");
        println!("  - Multi-line input is supported - keep typing until braces are balanced");
        println!("  - Use Ctrl+C to cancel current input");
        println!("  - Use Ctrl+D to exit");
        println!("  - Variable assignments persist between inputs");
        println!();
    }
}
