use super::{EvalResult, eval_expr};
use crate::ast::{BinaryOp, Expr};
use crate::runtime::env::Env;
use crate::runtime::module::ModuleRegistry;
use crate::runtime::shell::run_shell_bytes_with_input;
use crate::runtime::string_template::evaluate_string_template;
use crate::runtime::value::{FunctionValue, RuntimeError, Value};
use std::rc::Rc;

#[derive(Clone)]
enum PipeStage {
    Function(Box<FunctionValue>),
    ShellTemplate(Vec<crate::ast::StringPart>),
}

const PIPE_INVOCATION_MESSAGE: &str =
    "Pipe requires function invocations; write a(x) | b(y) instead of a | b";

fn is_invocation_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Call { .. } => true,
        Expr::MethodCall { .. } => true,
        Expr::Grouping { expr, .. } => is_invocation_expr(expr),
        _ => false,
    }
}

fn collect_pipe_stages(
    expr: &Expr,
    env: Rc<Env>,
    out: &mut Vec<PipeStage>,
) -> Result<(), RuntimeError> {
    match expr {
        Expr::Binary {
            left,
            op: BinaryOp::Pipe,
            right,
            ..
        } => {
            collect_pipe_stages(left, env.clone(), out)?;
            collect_pipe_stages(right, env, out)?;
            Ok(())
        }
        Expr::ShellCommandTemplate { parts, .. } => {
            out.push(PipeStage::ShellTemplate(parts.clone()));
            Ok(())
        }
        _ => {
            let is_invocation = is_invocation_expr(expr);
            let v = eval_expr(expr, env)?;
            match v {
                Value::Function(f) => {
                    if !is_invocation {
                        return Err(RuntimeError::PipeStageTypeError {
                            message: PIPE_INVOCATION_MESSAGE.to_string(),
                        });
                    }
                    out.push(PipeStage::Function(Box::new(f)));
                    Ok(())
                }
                _ => Err(RuntimeError::PipeStageTypeError {
                    message: "Pipe operator requires each stage to be either a function or a backtick command".to_string(),
                }),
            }
        }
    }
}

fn pipe_std_map_from(value: Value) -> indexmap::IndexMap<crate::runtime::value::MapKey, Value> {
    if let Value::Map(m) = value {
        m
    } else if let Value::Map(m) = crate::runtime::builtins::create_std_module() {
        m
    } else {
        indexmap::IndexMap::new()
    }
}

pub fn eval_pipe_expression(left: &Expr, right: &Expr, env: Rc<Env>) -> EvalResult<Value> {
    // Build stages
    let mut stages: Vec<PipeStage> = Vec::new();
    collect_pipe_stages(left, env.clone(), &mut stages)?;
    collect_pipe_stages(right, env.clone(), &mut stages)?;

    if stages.is_empty() {
        return Err(RuntimeError::EmptyPipeExpression {
            message: "Empty pipe expression".to_string(),
        });
    }

    // Base std module snapshot (from first function stage, if any)
    let base_std = match stages.first() {
        Some(PipeStage::Function(f)) => f.env.get("std").unwrap_or(Value::Nil),
        _ => Value::Nil,
    };

    let mut current_input: Option<Vec<u8>> = None;
    let last_index = stages.len() - 1;
    let mut last_result: Option<Value> = None;

    for (i, stage) in stages.into_iter().enumerate() {
        match stage {
            PipeStage::Function(func) => {
                // Start from a std map snapshot
                let mut std_map = pipe_std_map_from(base_std.clone());

                // Get io submodule map
                let io_key = crate::runtime::value::MapKey::String("io".to_string());
                let mut io_map = match std_map.get(&io_key) {
                    Some(Value::Map(m)) => m.clone(),
                    _ => match crate::runtime::builtins::create_std_module() {
                        Value::Map(std_m) => match std_m.get(&io_key) {
                            Some(Value::Map(m)) => m.clone(),
                            _ => indexmap::IndexMap::new(),
                        },
                        _ => indexmap::IndexMap::new(),
                    },
                };

                // Replace stdin if there is input
                if let Some(bytes) = current_input.take() {
                    let stdin_key = crate::runtime::value::MapKey::String("stdin".to_string());
                    let stdin_stream = Rc::new(
                        crate::runtime::value::StreamHandle::new_memory_readable(bytes),
                    );
                    io_map.insert(stdin_key, Value::Stream(stdin_stream));
                }

                // Replace stdout for non-last stages to capture output
                let mut capture_stdout = None;
                if i != last_index {
                    let stdout_key = crate::runtime::value::MapKey::String("stdout".to_string());
                    let stdout_handle =
                        Rc::new(crate::runtime::value::StreamHandle::new_memory_writable());
                    capture_stdout = Some(stdout_handle.clone());
                    io_map.insert(stdout_key, Value::Stream(stdout_handle));
                }

                // Put io back into std (keep a clone for env override)
                let io_map_clone = io_map.clone();
                std_map.insert(io_key.clone(), Value::Map(io_map));

                // Build overridden std value and module registry
                let overridden_std = Value::Map(std_map.clone());
                let registry = ModuleRegistry::new().with_custom_std(overridden_std);

                // Also inject overridden std and io directly into the call environment
                let env_overrides = Some(vec![
                    ("std".to_string(), Value::Map(std_map)),
                    ("io".to_string(), Value::Map(io_map_clone)),
                ]);

                // Call function with no args; allow default params
                match crate::runtime::eval::call_function_with_modules(
                    &func,
                    Vec::new(),
                    Some(env.clone()),
                    &registry,
                    env_overrides,
                ) {
                    Ok(val) => {
                        last_result = Some(val);
                        if let Some(stdout_handle) = capture_stdout {
                            current_input = stdout_handle.take_memory_output();
                        }
                    }
                    Err(e) => {
                        return Err(RuntimeError::PipeExecutionError {
                            stage: "closure".to_string(),
                            message: e.to_string(),
                        });
                    }
                }
            }
            PipeStage::ShellTemplate(parts) => {
                // Build command string by evaluating the template parts
                let command =
                    evaluate_string_template(&parts, |expr| eval_expr(expr, env.clone()))?;
                // Run command, providing current_input as stdin if present
                let out_bytes = run_shell_bytes_with_input(&command, current_input.take())
                    .map_err(|e| RuntimeError::PipeExecutionError {
                        stage: "command".to_string(),
                        message: e.to_string(),
                    })?;

                if i != last_index {
                    current_input = Some(out_bytes);
                } else {
                    // Last stage -> convert to UTF-8 string and return
                    let s =
                        String::from_utf8(out_bytes).map_err(|err| RuntimeError::ShellError {
                            message: format!("Shell command output is not valid UTF-8: {}", err),
                        })?;
                    last_result = Some(Value::String(s));
                }
            }
        }
    }

    Ok(last_result.unwrap_or(Value::Nil))
}
