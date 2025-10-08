/// Find the span of a variable usage in the source code
pub(crate) fn find_variable_usage(
    variable_name: &str,
    source: &str,
) -> Option<std::ops::Range<usize>> {
    use regex::Regex;

    // Create a regex pattern that matches the variable name as a whole word
    let pattern = format!(r"\b{}\b", regex::escape(variable_name));
    let regex = match Regex::new(&pattern) {
        Ok(regex) => regex,
        Err(_) => return None,
    };

    // Find the first match
    regex.find(source).map(|mat| mat.start()..mat.end())
}

/// Find similar variable names in the source code for suggestions
pub(crate) fn find_similar_variables(target: &str, source: &str) -> Vec<String> {
    use regex::Regex;

    let mut candidates = Vec::new();

    // Extract all identifiers from the source
    let identifier_regex = Regex::new(r"\b[a-zA-Z][a-zA-Z0-9_]*\b").unwrap();
    let mut seen = std::collections::HashSet::new();

    for mat in identifier_regex.find_iter(source) {
        let ident = mat.as_str();
        if ident != target && !seen.contains(ident) {
            seen.insert(ident.to_string());

            // Enhanced similarity check
            let similarity = calculate_similarity(target, ident);
            if similarity > 0.3 {
                // 30% similarity threshold
                candidates.push((ident.to_string(), similarity));
            }
        }
    }

    // Sort by similarity (highest first) and take top 3
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    candidates.truncate(3);
    candidates.into_iter().map(|(name, _)| name).collect()
}

/// Calculate similarity between two strings (Unicode-safe implementation)
pub(crate) fn calculate_similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }

    let char_count1 = s1.chars().count();
    let char_count2 = s2.chars().count();

    if char_count1 == 0 && char_count2 == 0 {
        return 1.0;
    }
    if char_count1 == 0 || char_count2 == 0 {
        return 0.0;
    }

    // Check for exact prefix match using single-pass iterator
    let min_char_count = char_count1.min(char_count2);
    let mut common_prefix = 0;
    for (a, b) in s1.chars().zip(s2.chars()) {
        if a == b {
            common_prefix += 1;
        } else {
            break;
        }
    }

    // Check for substring match
    let substring_match = if char_count1 <= char_count2 {
        s2.contains(s1)
    } else {
        s1.contains(s2)
    };

    // Calculate similarity score using character counts
    let prefix_score = common_prefix as f64 / min_char_count as f64;
    let length_score =
        1.0 - (char_count1 as f64 - char_count2 as f64).abs() / (char_count1 + char_count2) as f64;
    let substring_score = if substring_match { 0.5 } else { 0.0 };

    (prefix_score * 0.5 + length_score * 0.3 + substring_score * 0.2).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_similar_variables() {
        let source = "let my_variable = 42; let my_other_var = 10; let different = 5;";
        let suggestions = find_similar_variables("my_var", source);
        assert!(suggestions.contains(&"my_variable".to_string()));
        assert!(suggestions.contains(&"my_other_var".to_string()));
    }

    #[test]
    fn test_calculate_similarity() {
        assert_eq!(calculate_similarity("hello", "hello"), 1.0);
        assert!(calculate_similarity("hello", "hell") > 0.7);
        assert!(calculate_similarity("hello", "world") < 0.5);
        assert!(calculate_similarity("var", "variable") > 0.5);
    }
}
