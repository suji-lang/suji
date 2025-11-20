use super::common::{eval_program, eval_string_expr};

/// Test undefined variable errors
#[test]
fn test_undefined_variable_errors() {
    // Simple undefined variable
    assert!(eval_string_expr("x + 1").is_err());
    assert!(eval_program("result = undefined_var").is_err());

    // Undefined variable in expression
    assert!(eval_string_expr("x * 2 + y").is_err());

    // Undefined variable in function call (closure captures, but y must exist when f is called)
    // Note: Closures capture variables, but if y doesn't exist when f is called, it errors
    assert!(eval_program("f = |x| x + y\nresult = f(1)").is_err());

    // Undefined variable after assignment scope
    assert!(eval_program("x = 5\nresult = y").is_err());

    // Undefined variable in nested scope
    assert!(eval_program("x = 1\nf = || { return z }\nresult = f()").is_err());
}

/// Test arity mismatch errors
#[test]
fn test_arity_mismatch_errors() {
    // Too few arguments
    assert!(eval_program("f = |x, y| x + y\nresult = f(1)").is_err());

    // Too many arguments
    assert!(eval_program("f = |x| x * 2\nresult = f(1, 2)").is_err());

    // Zero-argument function called with arguments
    assert!(eval_program("f = || 42\nresult = f(1)").is_err());

    // Function with defaults - wrong number
    assert!(eval_program("f = |x, y = 10| x + y\nresult = f(1, 2, 3)").is_err());

    // Nested function call with arity mismatch
    assert!(eval_program("g = |x, y| x + y\nf = |x| g(x)\nresult = f(1)").is_err());
}

/// Test pipe apply type errors
#[test]
fn test_pipe_apply_type_errors() {
    // Pipe apply forward (|>) with non-function RHS
    assert!(eval_program("1 |> 2").is_err());
    assert!(eval_program("1 |> \"hello\"").is_err());
    assert!(eval_program("1 |> [1, 2, 3]").is_err());
    assert!(eval_program("1 |> true").is_err());

    // Pipe apply backward (<|) with non-function LHS
    assert!(eval_program("1 <| 2").is_err());
    assert!(eval_program("\"hello\" <| 2").is_err());
    assert!(eval_program("[1, 2] <| 2").is_err());
    assert!(eval_program("false <| 2").is_err());

    // Pipe apply with non-function after expression (parenthesized)
    assert!(eval_program("(1 + 2) |> 5").is_err());
    assert!(eval_program("5 <| (1 + 2)").is_err());

    // Pipe apply with non-function variable
    assert!(eval_program("x = 42\n1 |> x").is_err());
    assert!(eval_program("x = \"hello\"\nx <| 2").is_err());
}

/// Test destructuring errors
#[test]
fn test_destructuring_errors() {
    // Destructuring non-tuple/non-list value (only tuples and lists can be destructured)
    assert!(eval_program("x, y = 42").is_err());
    assert!(eval_program("x, y = \"hello\"").is_err());
    assert!(eval_program("x, y = { a: 1 }").is_err());

    // Destructuring arity mismatch - too few values
    assert!(eval_program("x, y, z = (1, 2)").is_err());
    assert!(eval_program("x, y, z = [1, 2]").is_err());

    // Destructuring arity mismatch - too many values
    assert!(eval_program("x, y = (1, 2, 3)").is_err());
    assert!(eval_program("x, y = [1, 2, 3]").is_err());

    // Destructuring from function return that's not a tuple or list
    assert!(eval_program("f = || 42\nx, y = f()").is_err());
    assert!(eval_program("f = || \"hello\"\nx, y = f()").is_err());
}

/// Test list concatenation errors
#[test]
fn test_list_concatenation_errors() {
    // List + non-list
    assert!(eval_string_expr("[1, 2] + \"hello\"").is_err());
    assert!(eval_string_expr("[1, 2] + 42").is_err());
    assert!(eval_string_expr("[1, 2] + true").is_err());
    assert!(eval_string_expr("[1, 2] + { a: 1 }").is_err());

    // Non-list + list
    assert!(eval_string_expr("\"hello\" + [1, 2]").is_err());
    assert!(eval_string_expr("42 + [1, 2]").is_err());
    assert!(eval_string_expr("true + [1, 2]").is_err());
}

/// Test map contains errors
#[test]
fn test_map_contains_errors() {
    // Map contains with invalid key type (lists, maps, nil cannot be keys)
    assert!(eval_program("m = { a: 1 }\nresult = m::contains([1, 2])").is_err());
    assert!(eval_program("m = { a: 1 }\nresult = m::contains({ b: 2 })").is_err());
    assert!(eval_program("m = { a: 1 }\nresult = m::contains(nil)").is_err());

    // Map contains called on non-map (strings and lists have contains, but numbers don't)
    assert!(eval_string_expr("42::contains(1)").is_err());
    assert!(eval_string_expr("true::contains(1)").is_err());
    assert!(eval_string_expr("nil::contains(1)").is_err());
}

/// Test invalid number conversion errors
#[test]
fn test_invalid_number_conversion_errors() {
    // Invalid string to number conversion
    assert!(eval_program("x = \"hello\"\nresult = x::to_number()").is_err());
    assert!(eval_program("x = \"abc123\"\nresult = x::to_number()").is_err());
    assert!(eval_program("x = \"\"\nresult = x::to_number()").is_err());

    // Invalid conversion operations
    assert!(eval_program("x = \"not a number\"\ny = x::to_number()").is_err());
}

/// Test additional method error cases
#[test]
fn test_method_call_errors() {
    // Method called on wrong type
    assert!(eval_string_expr("42::length()").is_err());
    assert!(eval_string_expr("true::length()").is_err());
    assert!(eval_string_expr("nil::length()").is_err());

    // Method arity mismatches (split() and join() have defaults, so test with too many args)
    assert!(eval_string_expr("\"hello\"::split(\",\", \"extra\")").is_err()); // too many args
    assert!(eval_string_expr("[1, 2]::join(\",\", \"extra\")").is_err()); // too many args

    // Invalid method arguments
    assert!(eval_string_expr("\"hello\"::split(42)").is_err()); // wrong type
    assert!(eval_string_expr("[1, 2]::join(42)").is_err()); // wrong type
    assert!(eval_string_expr("\"hello\"::repeat(-1)").is_err()); // negative repeat
    assert!(eval_string_expr("\"hello\"::repeat(\"bad\")").is_err()); // wrong type

    // Method on non-method-supporting types
    assert!(eval_string_expr("|| { return 1 }::length()").is_err());
    assert!(eval_string_expr("nil::split(\",\")").is_err());
}

/// Test index out of bounds errors (additional cases)
#[test]
fn test_index_out_of_bounds_additional() {
    // String index out of bounds (already tested in spec_lists_maps, but adding more)
    assert!(eval_string_expr("\"\"[0]").is_err());
    assert!(eval_string_expr("\"a\"[1]").is_err());
    assert!(eval_string_expr("\"hello\"[10]").is_err());

    // Negative index out of bounds
    assert!(eval_string_expr("\"\"[-1]").is_err());
    assert!(eval_string_expr("\"a\"[-2]").is_err());
}

/// Test invalid operations on types
#[test]
fn test_invalid_operations() {
    // Invalid binary operations
    assert!(eval_string_expr("\"hello\" * \"world\"").is_err());
    assert!(eval_string_expr("[1, 2] - [3, 4]").is_err());
    assert!(eval_string_expr("{ a: 1 } * { b: 2 }").is_err());

    // Invalid unary operations
    assert!(eval_string_expr("!\"hello\"").is_err());
    assert!(eval_string_expr("![1, 2]").is_err());
    assert!(eval_string_expr("-{ a: 1 }").is_err());
}
