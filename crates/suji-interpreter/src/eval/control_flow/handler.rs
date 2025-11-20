use suji_values::{ControlFlow, RuntimeError, Value};

/// Action to take after handling control flow
#[derive(Debug, Clone)]
pub enum ControlFlowAction {
    /// Continue loop iteration
    Continue,
    /// Break from loop with return value
    Break(Value),
    /// Return from function with value
    Return(Value),
    /// Propagate error to outer scope
    Propagate(RuntimeError),
}

/// Extract ControlFlow from an error, even if it's wrapped in WithSpan
fn extract_control_flow(error: &RuntimeError) -> Option<ControlFlow> {
    match error.without_span() {
        RuntimeError::ControlFlow { flow } => Some(flow.clone()),
        _ => None,
    }
}

/// Handle control flow error and determine appropriate action
pub fn handle_control_flow(error: &RuntimeError, label: Option<&str>) -> ControlFlowAction {
    if let Some(flow) = extract_control_flow(error) {
        match flow {
            ControlFlow::Break(None) => ControlFlowAction::Break(Value::Nil),
            ControlFlow::Break(Some(ref target)) => {
                if label.is_some_and(|l| l == target) {
                    ControlFlowAction::Break(Value::Nil)
                } else {
                    // Propagate the original error (may be wrapped)
                    ControlFlowAction::Propagate(error.clone())
                }
            }
            ControlFlow::Continue(None) => ControlFlowAction::Continue,
            ControlFlow::Continue(Some(ref target)) => {
                if label.is_some_and(|l| l == target) {
                    ControlFlowAction::Continue
                } else {
                    // Propagate the original error (may be wrapped)
                    ControlFlowAction::Propagate(error.clone())
                }
            }
            ControlFlow::Return(value) => {
                // Always propagate return - it should escape the loop
                ControlFlowAction::Return((*value).clone())
            }
        }
    } else {
        // Not a control flow error - propagate as-is
        ControlFlowAction::Propagate(error.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_values::DecimalNumber;

    #[test]
    fn test_break_none() {
        let error = RuntimeError::ControlFlow {
            flow: ControlFlow::Break(None),
        };
        let action = handle_control_flow(&error, None);
        assert!(matches!(action, ControlFlowAction::Break(Value::Nil)));

        let action = handle_control_flow(&error, Some("outer"));
        assert!(matches!(action, ControlFlowAction::Break(Value::Nil)));
    }

    #[test]
    fn test_break_with_label_match() {
        let error = RuntimeError::ControlFlow {
            flow: ControlFlow::Break(Some("outer".to_string())),
        };
        let action = handle_control_flow(&error, Some("outer"));
        assert!(matches!(action, ControlFlowAction::Break(Value::Nil)));
    }

    #[test]
    fn test_break_with_label_mismatch() {
        let error = RuntimeError::ControlFlow {
            flow: ControlFlow::Break(Some("outer".to_string())),
        };
        let action = handle_control_flow(&error, Some("inner"));
        assert!(matches!(action, ControlFlowAction::Propagate(_)));
    }

    #[test]
    fn test_continue_none() {
        let error = RuntimeError::ControlFlow {
            flow: ControlFlow::Continue(None),
        };
        let action = handle_control_flow(&error, None);
        assert!(matches!(action, ControlFlowAction::Continue));

        let action = handle_control_flow(&error, Some("outer"));
        assert!(matches!(action, ControlFlowAction::Continue));
    }

    #[test]
    fn test_continue_with_label_match() {
        let error = RuntimeError::ControlFlow {
            flow: ControlFlow::Continue(Some("outer".to_string())),
        };
        let action = handle_control_flow(&error, Some("outer"));
        assert!(matches!(action, ControlFlowAction::Continue));
    }

    #[test]
    fn test_continue_with_label_mismatch() {
        let error = RuntimeError::ControlFlow {
            flow: ControlFlow::Continue(Some("outer".to_string())),
        };
        let action = handle_control_flow(&error, Some("inner"));
        assert!(matches!(action, ControlFlowAction::Propagate(_)));
    }

    #[test]
    fn test_return_always_propagates() {
        let value = Value::Number(DecimalNumber::from_i64(42));
        let error = RuntimeError::ControlFlow {
            flow: ControlFlow::Return(Box::new(value.clone())),
        };
        let action = handle_control_flow(&error, None);
        match action {
            ControlFlowAction::Return(v) => assert_eq!(v, value),
            _ => panic!("Expected Return action"),
        }

        let action = handle_control_flow(&error, Some("outer"));
        match action {
            ControlFlowAction::Return(v) => assert_eq!(v, value),
            _ => panic!("Expected Return action"),
        }
    }

    #[test]
    fn test_non_control_flow_error() {
        let error = RuntimeError::TypeError {
            message: "Test error".to_string(),
        };
        let action = handle_control_flow(&error, None);
        assert!(matches!(action, ControlFlowAction::Propagate(_)));
    }
}
