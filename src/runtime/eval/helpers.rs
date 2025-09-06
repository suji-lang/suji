/// Helper function to determine if a break should target this loop
pub fn should_break_loop(current_label: Option<&str>, break_label: &Option<String>) -> bool {
    match break_label {
        Some(target_label) => current_label.map(|l| l == target_label).unwrap_or(false),
        None => true, // Unlabeled break targets innermost loop
    }
}

/// Helper function to determine if a continue should target this loop
pub fn should_continue_loop(current_label: Option<&str>, continue_label: &Option<String>) -> bool {
    match continue_label {
        Some(target_label) => current_label.map(|l| l == target_label).unwrap_or(false),
        None => true, // Unlabeled continue targets innermost loop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_break_loop_unlabeled() {
        // Unlabeled break should target any loop
        assert!(should_break_loop(None, &None));
        assert!(should_break_loop(Some("outer"), &None));
    }

    #[test]
    fn test_should_break_loop_labeled() {
        // Labeled break should only target matching loop
        assert!(should_break_loop(Some("outer"), &Some("outer".to_string())));
        assert!(!should_break_loop(
            Some("inner"),
            &Some("outer".to_string())
        ));
        assert!(!should_break_loop(None, &Some("outer".to_string())));
    }

    #[test]
    fn test_should_continue_loop_unlabeled() {
        // Unlabeled continue should target any loop
        assert!(should_continue_loop(None, &None));
        assert!(should_continue_loop(Some("outer"), &None));
    }

    #[test]
    fn test_should_continue_loop_labeled() {
        // Labeled continue should only target matching loop
        assert!(should_continue_loop(
            Some("outer"),
            &Some("outer".to_string())
        ));
        assert!(!should_continue_loop(
            Some("inner"),
            &Some("outer".to_string())
        ));
        assert!(!should_continue_loop(None, &Some("outer".to_string())));
    }
}
