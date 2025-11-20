use crate::eval::{EvalResult, call_function, eval_expr};
use std::rc::Rc;
use suji_ast::{BinaryOp, Expr};
use suji_runtime::ModuleRegistry;
use suji_values::Env;
use suji_values::IoContext;
use suji_values::evaluate_string_template;
use suji_values::run_shell_bytes_with_input;
use suji_values::{FunctionValue, RuntimeError, StreamHandle, Value};

#[derive(Clone)]
enum PipeStage {
    Function(Box<FunctionValue>),
    Invocation {
        function: Box<FunctionValue>,
        args: Vec<Value>,
    },
    ShellTemplate(Vec<suji_ast::StringPart>),
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
    registry: &ModuleRegistry,
    out: &mut Vec<PipeStage>,
) -> Result<(), RuntimeError> {
    match expr {
        Expr::Binary {
            left,
            op: BinaryOp::Pipe,
            right,
            ..
        } => {
            collect_pipe_stages(left, env.clone(), registry, out)?;
            collect_pipe_stages(right, env, registry, out)?;
            Ok(())
        }
        Expr::Grouping { expr: inner, .. } => {
            // Unwrap grouping and continue collection
            collect_pipe_stages(inner, env, registry, out)
        }
        Expr::Call { callee, args, .. } => {
            // Defer invocation to stage execution to run under redirected std/io
            let callee_val = eval_expr(callee, env.clone(), Some(registry))?;
            match callee_val {
                Value::Function(f) => {
                    // Evaluate arguments left-to-right
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(eval_expr(arg, env.clone(), Some(registry))?);
                    }
                    out.push(PipeStage::Invocation {
                        function: Box::new(f),
                        args: arg_values,
                    });
                    Ok(())
                }
                _ => Err(RuntimeError::PipeStageTypeError {
                    message: "Pipe operator requires each stage to be either a function or a backtick command".to_string(),
                }),
            }
        }
        Expr::ShellCommandTemplate { parts, .. } => {
            out.push(PipeStage::ShellTemplate(parts.clone()));
            Ok(())
        }
        _ => {
            let is_invocation = is_invocation_expr(expr);
            let v = eval_expr(expr, env, Some(registry))?;
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

/// Evaluate a pipe expression with optional module registry
pub fn eval_pipe_expression(
    left: &Expr,
    right: &Expr,
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Value> {
    // Pipe expressions require a registry
    let registry = registry.ok_or_else(|| RuntimeError::InvalidOperation {
        message: "Pipe expressions require a module registry".to_string(),
    })?;

    // Build stages
    let mut stages: Vec<PipeStage> = Vec::new();
    collect_pipe_stages(left, env.clone(), registry, &mut stages)?;
    collect_pipe_stages(right, env.clone(), registry, &mut stages)?;

    if stages.is_empty() {
        return Err(RuntimeError::EmptyPipeExpression {
            message: "Empty pipe expression".to_string(),
        });
    }

    let mut current_input: Option<Vec<u8>> = None;
    let last_index = stages.len() - 1;
    let mut last_result: Option<Value> = None;

    for (i, stage) in stages.into_iter().enumerate() {
        match stage {
            PipeStage::Function(func) => {
                // Prepare stdin override if there is input
                let stdin_override = current_input
                    .take()
                    .map(|bytes| Rc::new(StreamHandle::new_memory_readable(bytes)));

                // Prepare stdout capture for non-last stages
                let stdout_override = if i != last_index {
                    Some(Rc::new(StreamHandle::new_memory_writable()))
                } else {
                    None
                };

                // Execute function within IO context overrides
                let result: Result<Value, RuntimeError> = IoContext::with_overrides(
                    stdin_override.clone(),
                    stdout_override.clone(),
                    None, // stderr unchanged
                    || {
                        // Call function with no args; allow default params
                        let result = call_function(
                            &func,
                            Vec::new(),
                            Some(env.clone()),
                            Some(registry),
                            None, // no env overrides needed
                        )?;

                        // If the result is a function, call it too (handles nested closures)
                        // The inner function inherits the same redirected IO context
                        if let Value::Function(ref inner_fn) = result {
                            match call_function(
                                inner_fn,
                                Vec::new(),
                                Some(env.clone()),
                                Some(registry),
                                None,
                            ) {
                                Ok(v) => Ok(v),
                                Err(_) => Ok(result),
                            }
                        } else {
                            Ok(result)
                        }
                    },
                );

                match result {
                    Ok(val) => {
                        last_result = Some(val);
                        // Capture stdout AFTER calling any inner functions
                        if let Some(stdout_handle) = stdout_override {
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
            PipeStage::Invocation { function, args } => {
                // Prepare stdin override if there is input
                let stdin_override = current_input
                    .take()
                    .map(|bytes| Rc::new(StreamHandle::new_memory_readable(bytes)));

                // Prepare stdout capture for non-last stages
                let stdout_override = if i != last_index {
                    Some(Rc::new(StreamHandle::new_memory_writable()))
                } else {
                    None
                };

                // Execute function within IO context overrides
                let result: Result<Value, RuntimeError> = IoContext::with_overrides(
                    stdin_override.clone(),
                    stdout_override.clone(),
                    None, // stderr unchanged
                    || {
                        // Call function with args
                        let result = call_function(
                            &function,
                            args,
                            Some(env.clone()),
                            Some(registry),
                            None, // no env overrides needed
                        )?;

                        // If the result is a function, call it too (handles nested closures)
                        // The inner function inherits the same redirected IO context
                        if let Value::Function(ref inner_fn) = result {
                            match call_function(
                                inner_fn,
                                Vec::new(),
                                Some(env.clone()),
                                Some(registry),
                                None,
                            ) {
                                Ok(v) => Ok(v),
                                Err(_) => Ok(result),
                            }
                        } else {
                            Ok(result)
                        }
                    },
                );

                match result {
                    Ok(val) => {
                        last_result = Some(val);
                        // Capture stdout AFTER calling any inner functions
                        if let Some(stdout_handle) = stdout_override {
                            current_input = stdout_handle.take_memory_output();
                        }
                    }
                    Err(e) => {
                        return Err(RuntimeError::PipeExecutionError {
                            stage: "call".to_string(),
                            message: e.to_string(),
                        });
                    }
                }
            }
            PipeStage::ShellTemplate(parts) => {
                let command_string =
                    evaluate_string_template(&parts, |expr| eval_expr(expr, env.clone(), None))?;
                let output = run_shell_bytes_with_input(&command_string, current_input.clone())?;

                if i < last_index {
                    current_input = Some(output.clone());
                }

                last_result = Some(Value::String(String::from_utf8_lossy(&output).to_string()));
            }
        }
    }

    Ok(last_result.unwrap_or(Value::Nil))
}

/// Evaluate pipe apply expressions (|> and <|) with optional module registry
pub fn eval_pipe_apply_expression(
    left: &Expr,
    op: &BinaryOp,
    right: &Expr,
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<Value> {
    // Evaluate both sides left-to-right for consistent side effects
    let left_eval = eval_expr(left, env.clone(), registry)?;
    let right_eval = eval_expr(right, env.clone(), registry)?;

    // Decide which value is the function and which is the argument
    let (func_value, arg_value, non_func_error) = match op {
        BinaryOp::PipeApplyFwd => (right_eval, left_eval, RuntimeError::PipeApplyRightTypeError),
        BinaryOp::PipeApplyBwd => (left_eval, right_eval, RuntimeError::PipeApplyLeftTypeError),
        _ => unreachable!(
            "eval_pipe_apply_expression must be called with PipeApplyFwd or PipeApplyBwd",
        ),
    };

    match func_value {
        Value::Function(f) => call_function(&f, vec![arg_value], Some(env), registry, None),
        _ => Err(non_func_error),
    }
}
