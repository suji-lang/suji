use std::env;
use std::fs;
use std::process;
use std::rc::Rc;
use suji_diagnostics::{DiagnosticContext, DiagnosticKind, print_diagnostic};
use suji_parser::parse_program;
use suji_repl::Repl;
use suji_runtime::env::Env;
use suji_runtime::eval::eval_program_with_modules;
use suji_runtime::module::ModuleRegistry;
use suji_stdlib::setup_global_env;

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
            eprintln!("Usage: {} <file.si>", args[0]);
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
            if print_diagnostic(DiagnosticKind::Parse(e.clone()), &context).is_err() {
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
        Ok(_) => process::exit(0),
        Err(e) => {
            let context = DiagnosticContext::from_file(filename)?;
            if print_diagnostic(DiagnosticKind::Runtime(e.clone()), &context).is_err() {
                eprintln!("Runtime error: {}", e);
            }
            process::exit(1);
        }
    }
}

fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = Repl::new()?;
    repl.run()?;
    Ok(())
}
