use nnlang::diagnostics::{DiagnosticContext, DiagnosticKind, print_diagnostic};
use nnlang::parser::parse_program;
use nnlang::repl::Repl;
use nnlang::runtime::builtins::setup_global_env;
use nnlang::runtime::env::Env;
use nnlang::runtime::eval::eval_program_with_modules;
use nnlang::runtime::module::ModuleRegistry;
use std::env;
use std::fs;
use std::process;
use std::rc::Rc;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            // No arguments - start REPL
            if let Err(e) = run_repl() {
                eprintln!("REPL error: {}", e);
                process::exit(1);
            }
        }
        2 => {
            // One argument - execute file
            let filename = &args[1];
            if let Err(e) = run_file(filename) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: {} <file.nn>", args[0]);
            process::exit(1);
        }
    }
}

fn run_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read the file
    let source = fs::read_to_string(filename)?;

    // Parse the program
    let statements = match parse_program(&source) {
        Ok(stmts) => stmts,
        Err(e) => {
            let context = DiagnosticContext::from_file(filename)?;
            if let Err(_) = print_diagnostic(DiagnosticKind::Parse(e.clone()), &context) {
                eprintln!("Parse error: {}", e);
            }
            process::exit(1);
        }
    };

    // Create environment with built-ins
    let env = Rc::new(Env::new());
    setup_global_env(&env);

    // Create module registry
    let module_registry = ModuleRegistry::new();

    // Evaluate the program
    match eval_program_with_modules(&statements, env, &module_registry) {
        Ok(Some(value)) => {
            // Print the result if it's not null
            if !matches!(value, nnlang::runtime::value::Value::Null) {
                println!("{}", value);
            }
        }
        Ok(None) => {
            // Program completed without a value
        }
        Err(e) => {
            let context = DiagnosticContext::from_file(filename)?;
            if let Err(_) = print_diagnostic(DiagnosticKind::Runtime(e.clone()), &context) {
                eprintln!("Runtime error: {}", e);
            }
            process::exit(1);
        }
    }

    Ok(())
}

fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = Repl::new()?;
    repl.run()?;
    Ok(())
}
