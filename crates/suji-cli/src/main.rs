use std::env;
use std::fs;
use std::process;
use std::rc::Rc;
use suji_diagnostics::{DiagnosticContext, DiagnosticKind, print_diagnostic};
use suji_interpreter::{AstInterpreter, eval_module_source_callback};
use suji_repl::Repl;
use suji_runtime::{Executor, ModuleRegistry};
use suji_stdlib::{setup_global_env, setup_module_registry};
use suji_values::Env;

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

    // Create environment with built-ins
    let env = Rc::new(Env::new());
    setup_global_env(&env);

    // Create module registry and wire the evaluator callback
    let mut module_registry = ModuleRegistry::new();
    module_registry.set_source_evaluator(eval_module_source_callback);
    setup_module_registry(&mut module_registry);
    if let Some(parent) = std::path::Path::new(filename).parent() {
        module_registry.set_base_dir(parent);
    }

    // Create interpreter instance
    let interpreter = AstInterpreter;

    // Evaluate the source using the interpreter
    if let Err(e) = interpreter.eval_source(&source, env.clone(), &module_registry, false) {
        let context = DiagnosticContext::from_file(filename)?;
        if print_diagnostic(DiagnosticKind::Runtime(e.clone()), &context).is_err() {
            eprintln!("Error: {}", e);
        }
        process::exit(1);
    }

    process::exit(0)
}

fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = Repl::new()?;
    repl.run()?;
    Ok(())
}
