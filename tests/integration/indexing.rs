use suji_values::DecimalNumber;

use super::common::{eval_program, eval_program as eval_code, eval_string_expr};
use suji_values::Value;

#[test]
fn test_indexing_slicing_comprehensive() {
    // Test basic indexing
    let result = eval_string_expr("[1, 2, 3, 4][0]").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));

    let result = eval_string_expr("[1, 2, 3, 4][2]").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(3)));

    // Test slicing
    let result = eval_string_expr("[1, 2, 3, 4][1:3]").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(2)));
        assert_eq!(items[1], Value::Number(DecimalNumber::from_i64(3)));
    } else {
        panic!("Expected list from slicing");
    }

    // Test slicing with different ranges
    let result = eval_string_expr("[1, 2, 3, 4, 5][0:2]").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(1)));
        assert_eq!(items[1], Value::Number(DecimalNumber::from_i64(2)));
    } else {
        panic!("Expected list from slicing");
    }

    // Test on variables
    let result = eval_program("list = [1, 2, 3, 4]\nlist[1]").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));

    let result = eval_program("list = [1, 2, 3, 4]\nlist[1:3]").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 2);
    } else {
        panic!("Expected list from variable slicing");
    }
}

// ============================================================================
// Complex Indexing Tests
// ============================================================================

#[test]
fn test_index_with_function_call() {
    let code = r#"
        get_idx = || 2
        nums = [10, 20, 30, 40]
        nums[get_idx()]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_index_with_arithmetic() {
    let code = r#"
        nums = [10, 20, 30, 40]
        i = 1
        nums[i + 1]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_index_with_method_call() {
    let code = r#"
        nums = [10, 20, 30, 40]
        nums[nums::length() - 1]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "40");
}

#[test]
fn test_index_with_pipeline() {
    let code = r#"
        nums = [10, 20, 30, 40]
        get_two = |x| 2
        nums[0 |> get_two]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_nested_indexing() {
    let code = r#"
        indices = [0, 1, 2]
        data = [10, 20, 30]
        data[indices[1]]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "20");
}

#[test]
fn test_complex_mixed_operations() {
    let code = r#"
        get_one = || 1
        list = [0, 1]
        data = [100, 200, 300]
        data[get_one() + list[0]]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "200");
}

#[test]
fn test_assignment_with_function_call_index() {
    let code = r#"
        get_idx = || 2
        nums = [10, 20, 30, 40]
        nums[get_idx()] = 99
        nums[2]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "99");
}

#[test]
fn test_assignment_with_arithmetic_index() {
    let code = r#"
        nums = [10, 20, 30, 40]
        i = 1
        nums[i + 1] = 99
        nums[2]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "99");
}

#[test]
fn test_assignment_with_method_call_index() {
    let code = r#"
        nums = [10, 20, 30, 40]
        nums[nums::length() - 1] = 99
        nums[3]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "99");
}

#[test]
fn test_assignment_with_nested_index() {
    let code = r#"
        indices = [0, 1, 2]
        matrix = [[1, 2], [3, 4], [5, 6]]
        matrix[indices[1]][0] = 99
        matrix[1][0]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "99");
}

#[test]
fn test_slice_syntax_unaffected() {
    let code = r#"
        nums = [10, 20, 30, 40, 50]
        nums[1:3]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "[20, 30]");
}

#[test]
fn test_method_on_indexed_value() {
    let code = r#"
        words = ["hello", "world"]
        words[0]::length()
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "5");
}

#[test]
fn test_chain_multiple_complex_indices() {
    let code = r#"
        get_one = || 1
        get_zero = || 0
        matrix = [[1, 2], [3, 4], [5, 6]]
        matrix[get_one()][get_zero()]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "3");
}

#[test]
fn test_map_indexing_with_function_call() {
    let code = r#"
        get_key = || "b"
        data = { a: 10, b: 20, c: 30 }
        data[get_key()]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "20");
}

#[test]
fn test_map_indexing_with_arithmetic() {
    let code = r#"
        base = 1
        data = { 1: "one", 2: "two", 3: "three" }
        data[base + 1]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "two");
}

#[test]
fn test_string_indexing_with_function_call() {
    let code = r#"
        get_idx = || 1
        text = "hello"
        text[get_idx()]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "e");
}

#[test]
fn test_negative_index_with_arithmetic() {
    let code = r#"
        nums = [10, 20, 30, 40, 50]
        offset = 2
        nums[-1 - offset]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_complex_expression_multiple_operations() {
    let code = r#"
        double = |x| x * 2
        nums = [0, 1, 2, 3, 4, 5]
        base = 1
        nums[double(base) + 1]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "3");
}

#[test]
fn test_nested_function_calls_in_index() {
    let code = r#"
        get_two = || 2
        add_one = |x| x + 1
        nums = [10, 20, 30, 40, 50]
        nums[add_one(get_two())]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "40");
}

#[test]
fn test_method_chain_in_index() {
    let code = r#"
        nums = [10, 20, 30]
        indexes = [0, 1, 2]
        nums[indexes::length() - 1]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_pipeline_with_multiple_stages_in_index() {
    let code = r#"
        inc = |x| x + 1
        double = |x| x * 2
        nums = [10, 20, 30, 40, 50]
        nums[0 |> inc |> double]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_complex_assignment_deep_nesting() {
    let code = r#"
        get_i = || 0
        get_j = || 1
        matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
        matrix[get_i()][get_j()] = 99
        matrix[0][1]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "99");
}

#[test]
fn test_mixed_map_and_list_complex_index() {
    let code = r#"
        get_key = || "items"
        get_idx = || 1
        data = { 
            items: [10, 20, 30],
            count: 3
        }
        data[get_key()][get_idx()]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "20");
}

#[test]
fn test_assignment_mixed_structures_complex_index() {
    let code = r#"
        get_key = || "values"
        get_idx = || 2
        data = { 
            values: [1, 2, 3, 4, 5]
        }
        data[get_key()][get_idx()] = 99
        data["values"][2]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "99");
}

#[test]
fn test_ternary_like_index_expression() {
    let code = r#"
        use_first = true
        get_idx = || match use_first {
            true => 0,
            false => 1,
        }
        nums = [100, 200]
        nums[get_idx()]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "100");
}

#[test]
fn test_list_method_result_as_index() {
    let code = r#"
        nums = [10, 20, 30, 40]
        indices = [2]
        nums[indices::first()]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_very_complex_nested_expression() {
    let code = r#"
        get_offset = || 1
        multiplier = 2
        lists = [[10, 20], [30, 40], [50, 60]]
        idx = get_offset() * multiplier - 1
        lists[idx][get_offset()]
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result.to_string(), "40");
}
