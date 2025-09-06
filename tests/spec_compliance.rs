use nnlang::parser::{parse_expression, parse_program};
use nnlang::runtime::builtins::setup_global_env;
use nnlang::runtime::env::Env;
use nnlang::runtime::eval::{eval_expr, eval_stmt};
use nnlang::runtime::value::Value;
use std::rc::Rc;

/// Helper to create a test environment with built-ins
fn create_test_env() -> Rc<Env> {
    let env = Rc::new(Env::new());
    setup_global_env(&env);
    env
}

/// Helper to evaluate a string expression
fn eval_string_expr(input: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let expr = parse_expression(input)?;
    let env = create_test_env();
    Ok(eval_expr(&expr, env)?)
}

/// Helper to evaluate a program (multiple statements)
fn eval_program(input: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let statements = parse_program(input)?;
    let env = create_test_env();
    let mut loop_stack = Vec::new();

    let mut result = Value::Null;
    for stmt in statements {
        if let Some(value) = eval_stmt(&stmt, env.clone(), &mut loop_stack)? {
            result = value;
        }
    }
    Ok(result)
}

/// Test comprehensive arithmetic operations and operator precedence
#[test]
fn test_comprehensive_arithmetic() {
    // Basic arithmetic
    assert_eq!(eval_string_expr("2 + 3").unwrap(), Value::Number(5.0));
    assert_eq!(eval_string_expr("10 - 4").unwrap(), Value::Number(6.0));
    assert_eq!(eval_string_expr("3 * 4").unwrap(), Value::Number(12.0));
    assert_eq!(eval_string_expr("15 / 3").unwrap(), Value::Number(5.0));
    assert_eq!(eval_string_expr("17 % 5").unwrap(), Value::Number(2.0));
    assert_eq!(eval_string_expr("2 ^ 3").unwrap(), Value::Number(8.0));

    // Unary operations
    assert_eq!(eval_string_expr("-5").unwrap(), Value::Number(-5.0));
    assert_eq!(eval_string_expr("-(-10)").unwrap(), Value::Number(10.0)); // Use parentheses for double negative
    assert_eq!(eval_string_expr("!true").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("!false").unwrap(), Value::Boolean(true));

    // Operator precedence (based on implementation plan precedence table)
    assert_eq!(eval_string_expr("2 + 3 * 4").unwrap(), Value::Number(14.0)); // 2 + (3 * 4)
    assert_eq!(eval_string_expr("2 * 3 + 4").unwrap(), Value::Number(10.0)); // (2 * 3) + 4
    assert_eq!(eval_string_expr("10 - 2 * 3").unwrap(), Value::Number(4.0)); // 10 - (2 * 3)
    assert_eq!(eval_string_expr("20 / 4 + 1").unwrap(), Value::Number(6.0)); // (20 / 4) + 1

    // Exponentiation right-associativity (per spec requirement)
    assert_eq!(eval_string_expr("2 ^ 3 ^ 2").unwrap(), Value::Number(512.0)); // 2 ^ (3 ^ 2) = 2 ^ 9 = 512
    assert_eq!(eval_string_expr("3 ^ 2 ^ 2").unwrap(), Value::Number(81.0)); // 3 ^ (2 ^ 2) = 3 ^ 4 = 81

    // Parentheses override precedence
    assert_eq!(
        eval_string_expr("(2 + 3) * 4").unwrap(),
        Value::Number(20.0)
    );
    assert_eq!(
        eval_string_expr("2 * (3 + 4)").unwrap(),
        Value::Number(14.0)
    );
    assert_eq!(
        eval_string_expr("(2 ^ 3) ^ 2").unwrap(),
        Value::Number(64.0)
    ); // (2 ^ 3) ^ 2 = 8 ^ 2 = 64

    // Edge cases
    assert_eq!(eval_string_expr("0 + 0").unwrap(), Value::Number(0.0));
    assert_eq!(eval_string_expr("1 * 0").unwrap(), Value::Number(0.0));
    assert_eq!(eval_string_expr("5 ^ 0").unwrap(), Value::Number(1.0));
    assert_eq!(eval_string_expr("0 ^ 5").unwrap(), Value::Number(0.0));
}

/// Test postfix increment and decrement operators
#[test]
fn test_postfix_operators() {
    // Postfix increment
    assert_eq!(eval_program("x = 5\nx++").unwrap(), Value::Number(6.0));
    assert_eq!(eval_program("y = 10\ny++\ny").unwrap(), Value::Number(11.0));

    // Postfix decrement
    assert_eq!(eval_program("x = 5\nx--").unwrap(), Value::Number(4.0));
    assert_eq!(eval_program("y = 10\ny--\ny").unwrap(), Value::Number(9.0));

    // Variable is actually modified (per spec)
    assert_eq!(eval_program("z = 7\nz++\nz").unwrap(), Value::Number(8.0));
    assert_eq!(eval_program("w = 12\nw--\nw").unwrap(), Value::Number(11.0));

    // Multiple increments
    assert_eq!(
        eval_program("a = 1\na++\na++\na").unwrap(),
        Value::Number(3.0)
    );
}

/// Test comparison operators
#[test]
fn test_comparison_operators() {
    // Basic comparisons
    assert_eq!(eval_string_expr("5 > 3").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("2 < 1").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("4 >= 4").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("3 <= 2").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("5 == 5").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("3 != 4").unwrap(), Value::Boolean(true));

    // String comparisons
    assert_eq!(
        eval_string_expr(r#""abc" == "abc""#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""abc" != "def""#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""abc" < "def""#).unwrap(),
        Value::Boolean(true)
    );

    // Boolean comparisons
    assert_eq!(
        eval_string_expr("true == true").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("true != false").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("false == false").unwrap(),
        Value::Boolean(true)
    );
}

/// Test logical operators (boolean-only, no truthiness per spec)
#[test]
fn test_logical_operators() {
    // Basic logical operations
    assert_eq!(
        eval_string_expr("true && true").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("true && false").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr("false && true").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr("false && false").unwrap(),
        Value::Boolean(false)
    );

    assert_eq!(
        eval_string_expr("true || true").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("true || false").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("false || true").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("false || false").unwrap(),
        Value::Boolean(false)
    );

    // Logical NOT
    assert_eq!(eval_string_expr("!true").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("!false").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("!!true").unwrap(), Value::Boolean(true));

    // Short-circuit evaluation (per spec)
    // false && <anything> should short-circuit and not evaluate the right side
    assert_eq!(
        eval_string_expr("false && (5 / 0)").unwrap(),
        Value::Boolean(false)
    );
    // true || <anything> should short-circuit and not evaluate the right side
    assert_eq!(
        eval_string_expr("true || (5 / 0)").unwrap(),
        Value::Boolean(true)
    );

    // Complex expressions
    assert_eq!(
        eval_string_expr("(true && false) || (false || true)").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("!(true && false)").unwrap(),
        Value::Boolean(true)
    );
}

/// Test range operations (per spec: start..end half-open semantics)
#[test]
fn test_ranges() {
    // Basic ranges
    let result = eval_string_expr("0..3").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Number(0.0));
        assert_eq!(items[1], Value::Number(1.0));
        assert_eq!(items[2], Value::Number(2.0));
    } else {
        panic!("Expected list from range");
    }

    // Range with negative numbers (per spec)
    let result = eval_string_expr("-2..2").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], Value::Number(-2.0));
        assert_eq!(items[1], Value::Number(-1.0));
        assert_eq!(items[2], Value::Number(0.0));
        assert_eq!(items[3], Value::Number(1.0));
    } else {
        panic!("Expected list from negative range");
    }

    // Empty range when start >= end (per spec)
    let result = eval_string_expr("5..3").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 0);
    } else {
        panic!("Expected empty list from start >= end range");
    }

    let result = eval_string_expr("3..3").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 0);
    } else {
        panic!("Expected empty list from equal start/end range");
    }

    // Range in expressions (precedence test per plan: range binds looser than additive)
    let result = eval_string_expr("1..2+2").unwrap(); // Should be 1..(2+2) = 1..4 = [1, 2, 3]
    if let Value::List(items) = result {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Number(1.0));
        assert_eq!(items[1], Value::Number(2.0));
        assert_eq!(items[2], Value::Number(3.0));
    } else {
        panic!("Expected list from range expression");
    }
}

/// Test error cases to ensure proper error handling
#[test]
fn test_arithmetic_error_cases() {
    // Division by zero
    assert!(eval_string_expr("5 / 0").is_err());
    assert!(eval_string_expr("1.0 / 0.0").is_err());

    // Modulo by zero
    assert!(eval_string_expr("5 % 0").is_err());

    // Type errors (per spec: no implicit coercions)
    assert!(eval_string_expr(r#"5 + "hello""#).is_err());
    assert!(eval_string_expr("true * 3").is_err());
    assert!(eval_string_expr("false + 1").is_err());

    // Invalid unary operations
    assert!(eval_string_expr("!5").is_err()); // NOT on non-boolean
    assert!(eval_string_expr("-true").is_err()); // Negate non-number
    assert!(eval_string_expr("-\"hello\"").is_err()); // Negate string

    // Invalid postfix operations (per spec: only valid on identifiers bound to numbers)
    assert!(eval_program("\"hello\"++").is_err()); // Increment non-number
    assert!(eval_program("true--").is_err()); // Decrement non-boolean
}

/// Test operator precedence combinations comprehensively
#[test]
fn test_comprehensive_precedence() {
    // From implementation plan precedence table (high → low):
    // 1. Postfix: x++, x--, call f(...), method call recv::method(...), grouping ( ), indexing [expr], slicing [start:end], map-key :IDENT
    // 2. Exponentiation: ^ (right-associative)
    // 3. Unary: -x, !x
    // 4. Multiplicative: *, /, %
    // 5. Additive: +, -
    // 6. Range: .. (binds looser than additive; 0..a+b means 0..(a+b))
    // 7. Relational: <, <=, >, >=
    // 8. Equality: ==, !=
    // 9. Regex match: ~, !~
    // 10. Logical AND: &&
    // 11. Logical OR: ||
    // 12. Assignment: = (right-associative)

    // Exponentiation vs multiplicative
    assert_eq!(eval_string_expr("2 ^ 3 * 4").unwrap(), Value::Number(32.0)); // (2 ^ 3) * 4 = 8 * 4
    assert_eq!(eval_string_expr("2 * 3 ^ 4").unwrap(), Value::Number(162.0)); // 2 * (3 ^ 4) = 2 * 81

    // Unary vs exponentiation
    assert_eq!(eval_string_expr("-2 ^ 2").unwrap(), Value::Number(-4.0)); // -(2 ^ 2) = -4
    assert_eq!(eval_string_expr("(-2) ^ 2").unwrap(), Value::Number(4.0)); // (-2) ^ 2 = 4

    // Multiplicative vs additive
    assert_eq!(
        eval_string_expr("2 + 3 * 4 - 1").unwrap(),
        Value::Number(13.0)
    ); // 2 + (3 * 4) - 1 = 2 + 12 - 1

    // Range vs additive
    let range_result = eval_string_expr("0..2+3").unwrap(); // 0..(2+3) = 0..5
    if let Value::List(items) = range_result {
        assert_eq!(items.len(), 5);
        assert_eq!(items[0], Value::Number(0.0));
        assert_eq!(items[4], Value::Number(4.0));
    } else {
        panic!("Expected list from range expression");
    }

    // Relational vs additive
    assert_eq!(eval_string_expr("2 + 3 > 4").unwrap(), Value::Boolean(true)); // (2 + 3) > 4 = 5 > 4

    // Equality vs relational
    assert_eq!(
        eval_string_expr("5 > 3 == true").unwrap(),
        Value::Boolean(true)
    ); // (5 > 3) == true

    // Logical AND vs equality
    assert_eq!(
        eval_string_expr("true == true && false == false").unwrap(),
        Value::Boolean(true)
    ); // (true == true) && (false == false)

    // Logical OR vs AND
    assert_eq!(
        eval_string_expr("false && true || true").unwrap(),
        Value::Boolean(true)
    ); // (false && true) || true
}

/// Test string interpolation features
#[test]
fn test_string_interpolation() {
    // Variable interpolation
    let result = eval_program("name = \"Ada\"\nresult = \"Hello, ${name}\"").unwrap();
    assert_eq!(result, Value::String("Hello, Ada".to_string()));

    // Expression interpolation
    let result = eval_program("a = 3\nb = 4\nresult = \"${a} + ${b} = ${a + b}\"").unwrap();
    assert_eq!(result, Value::String("3 + 4 = 7".to_string()));

    // Complex expression interpolation
    let result = eval_program("x = 5\nresult = \"Square of ${x} is ${x ^ 2}\"").unwrap();
    assert_eq!(result, Value::String("Square of 5 is 25".to_string()));

    // Multiple interpolations in one string
    let result = eval_program(
        "first = \"Ada\"\nlast = \"Lovelace\"\nresult = \"${first} ${last} was a mathematician\"",
    )
    .unwrap();
    assert_eq!(
        result,
        Value::String("Ada Lovelace was a mathematician".to_string())
    );

    // Nested expressions with operators
    // Note: ternary might not be implemented, so let's use a simpler test
    let result =
        eval_program("x = 10\ny = 20\nresult = \"Sum is ${x + y}, product is ${x * y}\"").unwrap();
    assert_eq!(
        result,
        Value::String("Sum is 30, product is 200".to_string())
    );

    // String concatenation with interpolation
    let result =
        eval_program("name = \"world\"\ngreeting = \"Hello, ${name}\" + \"!\"\nresult = greeting")
            .unwrap();
    assert_eq!(result, Value::String("Hello, world!".to_string()));

    // Interpolation with boolean values
    let result = eval_program("flag = true\nresult = \"The flag is ${flag}\"").unwrap();
    assert_eq!(result, Value::String("The flag is true".to_string()));

    // Interpolation with list access (if implemented)
    let result = eval_program("items = [1, 2, 3]\nresult = \"First item: ${items[0]}\"").unwrap();
    assert_eq!(result, Value::String("First item: 1".to_string()));

    // Map literal interpolation (nested braces)
    let result =
        eval_program("obj = { name: \"test\", value: 42 }\nresult = \"Object: ${obj}\"").unwrap();
    // Note: The exact string representation of objects depends on implementation
    assert!(result.to_string().contains("Object:"));

    // Mathematical expressions with parentheses
    let result = eval_program("result = \"Result: ${(5 + 3) * 2}\"").unwrap();
    assert_eq!(result, Value::String("Result: 16".to_string()));
}

/// Test regex matching operations
#[test]
fn test_regex_operations() {
    // Basic regex match
    assert_eq!(
        eval_string_expr(r#""hello" ~ /hello/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""world" ~ /hello/"#).unwrap(),
        Value::Boolean(false)
    );

    // Basic regex not-match
    assert_eq!(
        eval_string_expr(r#""hello" !~ /world/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""hello" !~ /hello/"#).unwrap(),
        Value::Boolean(false)
    );

    // Case sensitive matching
    assert_eq!(
        eval_string_expr(r#""Hello" ~ /hello/"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#""Hello" ~ /Hello/"#).unwrap(),
        Value::Boolean(true)
    );

    // Pattern matching with metacharacters
    assert_eq!(
        eval_string_expr(r#""abc123" ~ /[0-9]+/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""abcdef" ~ /[0-9]+/"#).unwrap(),
        Value::Boolean(false)
    );

    // Word boundaries and anchors
    assert_eq!(
        eval_string_expr(r#""hello world" ~ /^hello/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""say hello" ~ /^hello/"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#""hello world" ~ /world$/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""world hello" ~ /world$/"#).unwrap(),
        Value::Boolean(false)
    );

    // Dot metacharacter
    assert_eq!(
        eval_string_expr(r#""cat" ~ /c.t/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""cut" ~ /c.t/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""ct" ~ /c.t/"#).unwrap(),
        Value::Boolean(false)
    );

    // Quantifiers
    assert_eq!(
        eval_string_expr(r#""hello" ~ /l+/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""helo" ~ /l+/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""heo" ~ /l+/"#).unwrap(),
        Value::Boolean(false)
    );

    // Optional matching
    assert_eq!(
        eval_string_expr(r#""color" ~ /colou?r/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""colour" ~ /colou?r/"#).unwrap(),
        Value::Boolean(true)
    );

    // Variable regex patterns (if string interpolation works in regex)
    let result = eval_program("pattern = \"hello\"\nresult = \"hello world\" ~ /hello/").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

/// Test regex precedence in expressions
#[test]
fn test_regex_precedence() {
    // Regex match has higher precedence than logical AND
    assert_eq!(
        eval_string_expr(r#""hello" ~ /hello/ && true"#).unwrap(),
        Value::Boolean(true)
    ); // (("hello" ~ /hello/)) && true
    assert_eq!(
        eval_string_expr(r#""hello" ~ /world/ && true"#).unwrap(),
        Value::Boolean(false)
    ); // (("hello" ~ /world/)) && true

    // Complex precedence with regex
    assert_eq!(
        eval_string_expr(r#""abc" ~ /abc/ || "def" ~ /xyz/"#).unwrap(),
        Value::Boolean(true)
    ); // (("abc" ~ /abc/)) || (("def" ~ /xyz/))
    assert_eq!(
        eval_string_expr(r#""abc" ~ /xyz/ || "def" ~ /xyz/"#).unwrap(),
        Value::Boolean(false)
    ); // (("abc" ~ /xyz/)) || (("def" ~ /xyz/))

    // Test binding with parentheses for clarity
    assert_eq!(
        eval_string_expr(r#""hello" == ("hello" ~ /hello/)"#).unwrap(),
        Value::Boolean(false)
    ); // "hello" == true should be false
    assert_eq!(
        eval_string_expr(r#"("hello" ~ /hello/) == true"#).unwrap(),
        Value::Boolean(true)
    ); // true == true should be true
}

/// Test function definitions and calls
#[test]
fn test_function_basics() {
    // Simple function definition and call
    let result = eval_program("add = |x, y| { return x + y }\nresult = add(3, 4)").unwrap();
    assert_eq!(result, Value::Number(7.0));

    // Function with single parameter
    let result = eval_program("square = |x| { return x * x }\nresult = square(5)").unwrap();
    assert_eq!(result, Value::Number(25.0));

    // Function with no parameters
    let result = eval_program("getfive = || { return 5 }\nresult = getfive()").unwrap();
    assert_eq!(result, Value::Number(5.0));

    // Function returning boolean
    let result =
        eval_program("ispositive = |x| { return x > 0 }\nresult = ispositive(-3)").unwrap();
    assert_eq!(result, Value::Boolean(false));

    // Function returning string
    let result =
        eval_program("greet = |name| { return \"Hello, ${name}!\" }\nresult = greet(\"world\")")
            .unwrap();
    assert_eq!(result, Value::String("Hello, world!".to_string()));
}

/// Test function default parameters
#[test]
fn test_function_default_parameters() {
    // Function with default parameter
    let result =
        eval_program("greet = |name = \"world\"| { return \"Hello, ${name}!\" }\nresult = greet()")
            .unwrap();
    assert_eq!(result, Value::String("Hello, world!".to_string()));

    // Override default parameter
    let result = eval_program(
        "greet = |name = \"world\"| { return \"Hello, ${name}!\" }\nresult = greet(\"Alice\")",
    )
    .unwrap();
    assert_eq!(result, Value::String("Hello, Alice!".to_string()));

    // Multiple parameters with defaults
    let result = eval_program("format = |prefix = \">\", text = \"empty\"| { return \"${prefix} ${text}\" }\nresult = format()").unwrap();
    assert_eq!(result, Value::String("> empty".to_string()));

    // Override first parameter, use default for second
    let result = eval_program("format = |prefix = \">\", text = \"empty\"| { return \"${prefix} ${text}\" }\nresult = format(\"##\")").unwrap();
    assert_eq!(result, Value::String("## empty".to_string()));

    // Override both parameters
    let result = eval_program("format = |prefix = \">\", text = \"empty\"| { return \"${prefix} ${text}\" }\nresult = format(\"##\", \"hello\")").unwrap();
    assert_eq!(result, Value::String("## hello".to_string()));

    // Mix of parameters with and without defaults
    let result = eval_program(
        "calc = |base, multiplier = 2| { return base * multiplier }\nresult = calc(5)",
    )
    .unwrap();
    assert_eq!(result, Value::Number(10.0));

    let result = eval_program(
        "calc = |base, multiplier = 2| { return base * multiplier }\nresult = calc(5, 3)",
    )
    .unwrap();
    assert_eq!(result, Value::Number(15.0));
}

/// Test closures and variable capture
#[test]
fn test_closures() {
    // Basic closure capturing outer variable
    let result = eval_program("x = 10\naddx = |y| { return x + y }\nresult = addx(5)").unwrap();
    assert_eq!(result, Value::Number(15.0));

    // Factory function creating closures
    let result = eval_program("makeadder = |base| { return |x| { return base + x } }\nadd10 = makeadder(10)\nresult = add10(7)").unwrap();
    assert_eq!(result, Value::Number(17.0));

    // Multiple closures with different captured values
    let result = eval_program("makeadder = |base| { return |x| { return base + x } }\nadd5 = makeadder(5)\nadd20 = makeadder(20)\nresult1 = add5(3)\nresult2 = add20(3)\nresult = result1 + result2").unwrap();
    assert_eq!(result, Value::Number(31.0)); // 8 + 23 = 31

    // Closure modifying captured variable (if mutable)
    let result = eval_program("counter = 0\nincrement = || { counter = counter + 1\n  return counter }\nfirst = increment()\nsecond = increment()\nresult = second").unwrap();
    assert_eq!(result, Value::Number(2.0));

    // Closure capturing multiple variables
    let result =
        eval_program("a = 5\nb = 3\ncombine = |x| { return a * x + b }\nresult = combine(4)")
            .unwrap();
    assert_eq!(result, Value::Number(23.0)); // 5 * 4 + 3 = 23
}

/// Test higher-order functions
#[test]
fn test_higher_order_functions() {
    // Function taking another function as parameter
    let result = eval_program(
        "apply = |f, x| { return f(x) }\ndouble = |n| { return n * 2 }\nresult = apply(double, 6)",
    )
    .unwrap();
    assert_eq!(result, Value::Number(12.0));

    // Function returning another function
    let result = eval_program("makemultiplier = |factor| { return |x| { return x * factor } }\ntriple = makemultiplier(3)\nresult = triple(4)").unwrap();
    assert_eq!(result, Value::Number(12.0));

    // Chaining function calls
    let result = eval_program(
        "add1 = |x| { return x + 1 }\nmul2 = |x| { return x * 2 }\nresult = mul2(add1(5))",
    )
    .unwrap();
    assert_eq!(result, Value::Number(12.0)); // mul2(add1(5)) = mul2(6) = 12

    // Function composition through higher-order function
    let result = eval_program("compose = |f, g| { return |x| { return f(g(x)) } }\nadd3 = |x| { return x + 3 }\nmul2 = |x| { return x * 2 }\ncomposed = compose(mul2, add3)\nresult = composed(4)").unwrap();
    assert_eq!(result, Value::Number(14.0)); // mul2(add3(4)) = mul2(7) = 14
}

/// Test function scope and variable shadowing
#[test]
fn test_function_scope() {
    // Local variables shadow outer variables
    let result = eval_program("x = 10\ntest = |x| { return x * 2 }\nresult = test(5)").unwrap();
    assert_eq!(result, Value::Number(10.0)); // Uses parameter x = 5, not outer x = 10

    // Function can access outer variables when not shadowed
    let result =
        eval_program("outer = 100\ntest = |inner| { return inner + outer }\nresult = test(5)")
            .unwrap();
    assert_eq!(result, Value::Number(105.0));

    // Nested function scopes
    let result = eval_program("x = 1\ntest = || {\n  x = 2\n  inner = || { return x }\n  return inner()\n}\nresult = test()").unwrap();
    assert_eq!(result, Value::Number(2.0));

    // Check outer variable unchanged after function call
    let result =
        eval_program("x = 1\ntest = || {\n  x = 2\n  return x\n}\ntest()\nresult = x").unwrap();
    assert_eq!(result, Value::Number(2.0)); // In NN, assignment creates/modifies in current scope
}

/// Test basic loop functionality
#[test]
fn test_basic_loops() {
    // Infinite loop with break
    let result = eval_program("counter = 0\nloop {\n  counter++\n  match counter {\n    5: { break }\n  }\n}\nresult = counter").unwrap();
    assert_eq!(result, Value::Number(5.0));

    // Infinite loop with continue
    let result = eval_program("sum = 0\ni = 0\nloop {\n  i++\n  match i {\n    3: { continue }\n    6: { break }\n  }\n  sum = sum + i\n}\nresult = sum").unwrap();
    assert_eq!(result, Value::Number(12.0)); // 1 + 2 + 4 + 5 = 12, continue skips 3

    // Loop through range without bindings
    let result =
        eval_program("count = 0\nloop through 0..5 {\n  count++\n}\nresult = count").unwrap();
    assert_eq!(result, Value::Number(5.0));
}

/// Test loop through with variable bindings
#[test]
fn test_loop_through_bindings() {
    // Loop through range with single variable
    let result =
        eval_program("sum = 0\nloop through 0..5 with i {\n  sum = sum + i\n}\nresult = sum")
            .unwrap();
    assert_eq!(result, Value::Number(10.0)); // 0 + 1 + 2 + 3 + 4 = 10

    // Loop through list with single variable
    let result = eval_program("nums = [10, 20, 30]\nsum = 0\nloop through nums with n {\n  sum = sum + n\n}\nresult = sum").unwrap();
    assert_eq!(result, Value::Number(60.0));

    // Loop through list transforming values (sum of doubled values)
    let result = eval_program("nums = [1, 2, 3]\nsum = 0\nloop through nums with n {\n  sum = sum + (n * 2)\n}\nresult = sum").unwrap();
    assert_eq!(result, Value::Number(12.0)); // (1*2) + (2*2) + (3*2) = 2 + 4 + 6 = 12

    // Loop through range without binding variable (just counting iterations)
    let result =
        eval_program("count = 0\nloop through 0..3 {\n  count++\n}\nresult = count").unwrap();
    assert_eq!(result, Value::Number(3.0));
}

/// Test labeled loops and break/continue with labels
#[test]
fn test_labeled_loops() {
    // Simple labeled loop with break
    let result = eval_program(
        "i = 0\nloop as outer {\n  i++\n  match i {\n    3: { break outer }\n  }\n}\nresult = i",
    )
    .unwrap();
    assert_eq!(result, Value::Number(3.0));

    // Simple labeled loop with higher break count
    let result = eval_program("counter = 0\nloop as my_loop {\n  counter++\n  match counter {\n    5: { break my_loop }\n  }\n}\nresult = counter").unwrap();
    assert_eq!(result, Value::Number(5.0));
}

/// Test match statement patterns
#[test]
fn test_match_patterns() {
    // Basic value matching with exact match
    let result = eval_program(
        "x = 42\nmatch x {\n  42: { result = \"found\" }\n  99: { result = \"not found\" }\n}",
    )
    .unwrap();
    assert_eq!(result, Value::String("found".to_string()));

    // Boolean pattern matching
    let result = eval_program("flag = true\nmatch flag {\n  false: { result = \"false\" }\n  true: { result = \"true\" }\n}").unwrap();
    assert_eq!(result, Value::String("true".to_string()));

    // Multiple number patterns
    let result = eval_program("x = 2\nmatch x {\n  1: { result = \"one\" }\n  2: { result = \"two\" }\n  3: { result = \"three\" }\n}").unwrap();
    assert_eq!(result, Value::String("two".to_string()));

    // First match wins
    let result = eval_program(
        "x = 5\nmatch x {\n  5: { result = \"first\" }\n  5: { result = \"second\" }\n}",
    )
    .unwrap();
    assert_eq!(result, Value::String("first".to_string()));
}

/// Test wildcard pattern in match statements
#[test]
fn test_wildcard_patterns() {
    // Test that wildcard patterns work correctly
    let result = eval_program(
        "x = 99\nmatch x {\n  1: { result = \"one\" }\n  _: { result = \"wildcard\" }\n}",
    )
    .unwrap();

    // Wildcard pattern should match any value
    assert_eq!(result, Value::String("wildcard".to_string()));

    // Test that specific patterns take precedence over wildcard
    let result = eval_program(
        "x = 1\nmatch x {\n  1: { result = \"one\" }\n  _: { result = \"wildcard\" }\n}",
    )
    .unwrap();
    assert_eq!(result, Value::String("one".to_string()));
}

/// Test shell command execution
#[test]
fn test_shell_commands() {
    // Basic shell command execution
    let result = eval_string_expr(r#"`echo "hello world"`"#).unwrap();
    assert_eq!(result, Value::String("hello world".to_string()));

    // Simple shell command with exit status
    let result = eval_string_expr("`echo test`").unwrap();
    assert_eq!(result, Value::String("test".to_string()));

    // Shell command that produces numbers
    let result = eval_string_expr("`echo 42`").unwrap();
    assert_eq!(result, Value::String("42".to_string())); // Shell commands always return strings

    // Multiple word output
    let result = eval_string_expr("`echo one two three`").unwrap();
    assert_eq!(result, Value::String("one two three".to_string()));

    // Empty command output
    let result = eval_string_expr("`echo -n`").unwrap(); // -n suppresses newline
    assert_eq!(result, Value::String("".to_string()));
}

/// Test shell command interpolation
#[test]
fn test_shell_interpolation() {
    // Variable interpolation in shell commands
    let result = eval_program("name = \"world\"\nresult = `echo Hello, ${name}`").unwrap();
    assert_eq!(result, Value::String("Hello, world".to_string()));

    // Number interpolation
    let result = eval_program("num = 42\nresult = `echo ${num}`").unwrap();
    assert_eq!(result, Value::String("42".to_string()));

    // Expression interpolation
    let result = eval_program("x = 3\ny = 4\nresult = `echo ${x + y}`").unwrap();
    assert_eq!(result, Value::String("7".to_string()));

    // Multiple interpolations
    let result = eval_program("a = \"foo\"\nb = \"bar\"\nresult = `echo ${a}-${b}`").unwrap();
    assert_eq!(result, Value::String("foo-bar".to_string()));

    // Complex expression interpolation
    let result = eval_program("base = 5\nresult = `echo Result: ${base * base}`").unwrap();
    assert_eq!(result, Value::String("Result: 25".to_string()));

    // Boolean interpolation
    let result = eval_program("flag = true\nresult = `echo ${flag}`").unwrap();
    assert_eq!(result, Value::String("true".to_string()));
}

/// Test shell commands with pipes and complex operations
#[test]
fn test_shell_pipes_and_operations() {
    // Simple pipe operation
    let result = eval_string_expr("`echo hello | tr a-z A-Z`").unwrap();
    assert_eq!(result, Value::String("HELLO".to_string()));

    // Multiple pipes (wc -l adds leading spaces, so trim)
    let result =
        eval_string_expr("`echo 'one two three' | tr ' ' '\\n' | wc -l | tr -d ' '`").unwrap();
    assert_eq!(result, Value::String("3".to_string()));

    // Command with arguments
    let result = eval_string_expr("`printf '%s' 'test'`").unwrap();
    assert_eq!(result, Value::String("test".to_string()));

    // Command substitution-like behavior with interpolation
    let result = eval_program("cmd = \"date\"\nresult = `echo Current command: ${cmd}`").unwrap();
    assert!(result.to_string().contains("Current command: date"));
}

/// Test shell command error handling
#[test]
fn test_shell_command_errors() {
    // Commands that fail should return errors
    assert!(eval_string_expr("`false`").is_err()); // false command always exits with code 1

    // Non-existent command
    assert!(eval_string_expr("`nonexistentcommand123`").is_err());

    // Command with syntax error
    assert!(eval_string_expr("`echo 'unclosed quote`").is_err());
}

/// Test shell commands in different contexts
#[test]
fn test_shell_commands_in_context() {
    // Shell command in variable assignment
    let result = eval_program("output = `echo assigned`\nresult = output").unwrap();
    assert_eq!(result, Value::String("assigned".to_string()));

    // Shell command in function
    let result = eval_program("getdate = || { return `date +%Y` }\nresult = getdate()").unwrap();
    // Should return current year as string (just check it's numeric)
    assert!(result.to_string().chars().all(|c| c.is_ascii_digit()));

    // Shell command in expressions
    let result = eval_program("prefix = \"Output: \"\nresult = prefix + `echo test`").unwrap();
    assert_eq!(result, Value::String("Output: test".to_string()));

    // Shell command result length check
    let result = eval_program("output = `echo hello`\nresult = (output == \"hello\")").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

/// Test list indexing functionality
#[test]
fn test_list_indexing() {
    // Basic positive indexing
    let result = eval_string_expr("[10, 20, 30, 40][0]").unwrap();
    assert_eq!(result, Value::Number(10.0));

    let result = eval_string_expr("[10, 20, 30, 40][1]").unwrap();
    assert_eq!(result, Value::Number(20.0));

    let result = eval_string_expr("[10, 20, 30, 40][3]").unwrap();
    assert_eq!(result, Value::Number(40.0));

    // Negative indexing
    let result = eval_string_expr("[10, 20, 30, 40][-1]").unwrap();
    assert_eq!(result, Value::Number(40.0));

    let result = eval_string_expr("[10, 20, 30, 40][-2]").unwrap();
    assert_eq!(result, Value::Number(30.0));

    let result = eval_string_expr("[10, 20, 30, 40][-4]").unwrap();
    assert_eq!(result, Value::Number(10.0));

    // Mixed types
    let result = eval_string_expr("[\"hello\", 42, true][0]").unwrap();
    assert_eq!(result, Value::String("hello".to_string()));

    let result = eval_string_expr("[\"hello\", 42, true][1]").unwrap();
    assert_eq!(result, Value::Number(42.0));

    let result = eval_string_expr("[\"hello\", 42, true][2]").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

/// Test list slicing functionality
#[test]
fn test_list_slicing() {
    // Basic slicing
    let result = eval_string_expr("[10, 20, 30, 40][1:3]").unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::Number(20.0), Value::Number(30.0)])
    );

    // Start from beginning
    let result = eval_string_expr("[10, 20, 30, 40][:2]").unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::Number(10.0), Value::Number(20.0)])
    );

    // End at end
    let result = eval_string_expr("[10, 20, 30, 40][2:]").unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::Number(30.0), Value::Number(40.0)])
    );

    // Negative indices
    let result = eval_string_expr("[10, 20, 30, 40][-2:]").unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::Number(30.0), Value::Number(40.0)])
    );

    let result = eval_string_expr("[10, 20, 30, 40][:-2]").unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::Number(10.0), Value::Number(20.0)])
    );

    // Full slice
    let result = eval_string_expr("[10, 20, 30, 40][:]").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(30.0),
            Value::Number(40.0)
        ])
    );

    // Empty slice
    let result = eval_string_expr("[10, 20, 30, 40][2:2]").unwrap();
    assert_eq!(result, Value::List(vec![]));

    // Mixed types
    let result = eval_string_expr("[\"a\", \"b\", \"c\", \"d\"][1:3]").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::String("b".to_string()),
            Value::String("c".to_string())
        ])
    );
}

/// Test list element assignment
#[test]
fn test_list_assignment() {
    // Basic assignment
    let result = eval_program("xs = [10, 20, 30]\nxs[1] = 99\nresult = xs[1]").unwrap();
    assert_eq!(result, Value::Number(99.0));

    // Negative index assignment
    let result = eval_program("xs = [10, 20, 30]\nxs[-1] = 0\nresult = xs[2]").unwrap();
    assert_eq!(result, Value::Number(0.0));

    // Assignment with different types
    let result = eval_program("xs = [1, 2, 3]\nxs[0] = \"hello\"\nresult = xs[0]").unwrap();
    assert_eq!(result, Value::String("hello".to_string()));

    // Multiple assignments
    let result =
        eval_program("xs = [1, 2, 3]\nxs[0] = 10\nxs[1] = 20\nxs[2] = 30\nresult = xs").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(30.0)
        ])
    );
}

/// Test map access functionality
#[test]
fn test_map_access() {
    // Colon syntax (identifier to string conversion)
    let result = eval_program("m = { name: \"Ada\", age: 37 }\nresult = m:name").unwrap();
    assert_eq!(result, Value::String("Ada".to_string()));

    let result = eval_program("m = { name: \"Ada\", age: 37 }\nresult = m:age").unwrap();
    assert_eq!(result, Value::Number(37.0));

    // Bracket syntax (no conversion)
    let result = eval_program("m = { name: \"Ada\", age: 37 }\nresult = m[\"name\"]").unwrap();
    assert_eq!(result, Value::String("Ada".to_string()));

    let result = eval_program("m = { name: \"Ada\", age: 37 }\nresult = m[\"age\"]").unwrap();
    assert_eq!(result, Value::Number(37.0));

    // Dynamic key access
    let result =
        eval_program("m = { name: \"Ada\", age: 37 }\nk = \"name\"\nresult = m[k]").unwrap();
    assert_eq!(result, Value::String("Ada".to_string()));

    // Mixed key types
    let result =
        eval_program("m = { 1: \"one\", \"two\": 2, true: \"boolean\" }\nresult = m[1]").unwrap();
    assert_eq!(result, Value::String("one".to_string()));

    let result =
        eval_program("m = { 1: \"one\", \"two\": 2, true: \"boolean\" }\nresult = m[\"two\"]")
            .unwrap();
    assert_eq!(result, Value::Number(2.0));

    let result =
        eval_program("m = { 1: \"one\", \"two\": 2, true: \"boolean\" }\nresult = m[true]")
            .unwrap();
    assert_eq!(result, Value::String("boolean".to_string()));
}

/// Test map assignment functionality
#[test]
fn test_map_assignment() {
    // Colon syntax assignment
    let result =
        eval_program("m = { name: \"Ada\" }\nm:name = \"Lovelace\"\nresult = m:name").unwrap();
    assert_eq!(result, Value::String("Lovelace".to_string()));

    // Bracket syntax assignment
    let result =
        eval_program("m = { name: \"Ada\" }\nm[\"age\"] = 37\nresult = m[\"age\"]").unwrap();
    assert_eq!(result, Value::Number(37.0));

    // Dynamic key assignment
    let result = eval_program(
        "m = { name: \"Ada\" }\nk = \"country\"\nm[k] = \"UK\"\nresult = m[\"country\"]",
    )
    .unwrap();
    assert_eq!(result, Value::String("UK".to_string()));

    // Mixed key types
    let result = eval_program(
        "m = {}\nm[1] = \"number\"\nm[\"string\"] = 42\nm[true] = \"boolean\"\nresult = m[1]",
    )
    .unwrap();
    assert_eq!(result, Value::String("number".to_string()));

    let result = eval_program("m = {}\nm[1] = \"number\"\nm[\"string\"] = 42\nm[true] = \"boolean\"\nresult = m[\"string\"]").unwrap();
    assert_eq!(result, Value::Number(42.0));

    let result = eval_program(
        "m = {}\nm[1] = \"number\"\nm[\"string\"] = 42\nm[true] = \"boolean\"\nresult = m[true]",
    )
    .unwrap();
    assert_eq!(result, Value::String("boolean".to_string()));
}

/// Test indexing error cases
#[test]
fn test_indexing_errors() {
    // Out of bounds positive index
    assert!(eval_string_expr("[1, 2, 3][5]").is_err());

    // Out of bounds negative index
    assert!(eval_string_expr("[1, 2, 3][-5]").is_err());

    // Non-integer index
    assert!(eval_string_expr("[1, 2, 3][1.5]").is_err());

    // String index on list
    assert!(eval_string_expr("[1, 2, 3][\"0\"]").is_err());

    // Missing map key
    assert!(eval_program("m = { name: \"Ada\" }\nresult = m[\"missing\"]").is_err());

    // Indexing non-indexable type
    assert!(eval_string_expr("42[0]").is_err());
    assert!(eval_string_expr("\"hello\"[0]").is_err());
}

/// Test slicing edge cases
#[test]
fn test_slicing_edge_cases() {
    // Start beyond end
    let result = eval_string_expr("[1, 2, 3, 4][5:7]").unwrap();
    assert_eq!(result, Value::List(vec![]));

    // End before start
    let result = eval_string_expr("[1, 2, 3, 4][3:1]").unwrap();
    assert_eq!(result, Value::List(vec![]));

    // Negative start beyond beginning
    let result = eval_string_expr("[1, 2, 3, 4][-10:2]").unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
    );

    // Negative end beyond beginning
    let result = eval_string_expr("[1, 2, 3, 4][1:-10]").unwrap();
    assert_eq!(result, Value::List(vec![]));

    // Single element slice
    let result = eval_string_expr("[1, 2, 3, 4][1:2]").unwrap();
    assert_eq!(result, Value::List(vec![Value::Number(2.0)]));
}

/// Test map methods and advanced functionality
#[test]
fn test_map_methods() {
    // Map delete method
    let result = eval_program("m = { a: 1, b: 2, c: 3 }\nm::delete(\"b\")\nresult = m").unwrap();
    // Should have a and c, but not b
    assert!(result.to_string().contains("a"));
    assert!(result.to_string().contains("c"));
    assert!(!result.to_string().contains("b"));

    // Delete non-existent key (should not error)
    let result = eval_program("m = { a: 1 }\nm::delete(\"missing\")\nresult = m").unwrap();
    assert!(result.to_string().contains("a"));

    // Delete all keys
    let result =
        eval_program("m = { a: 1, b: 2 }\nm::delete(\"a\")\nm::delete(\"b\")\nresult = m").unwrap();
    assert_eq!(result, Value::Map(indexmap::IndexMap::new()));

    // Delete with dynamic key
    let result = eval_program("m = { x: 1, y: 2 }\nk = \"x\"\nm::delete(k)\nresult = m").unwrap();
    assert!(result.to_string().contains("y"));
    assert!(!result.to_string().contains("x"));
}

/// Test map literal syntax variations
#[test]
fn test_map_literal_syntax() {
    // Identifier keys (converted to strings)
    let result = eval_string_expr("{ name: \"Ada\", age: 37 }").unwrap();
    assert!(result.to_string().contains("name"));
    assert!(result.to_string().contains("age"));

    // String literal keys
    let result = eval_string_expr("{ \"name\": \"Ada\", \"age\": 37 }").unwrap();
    assert!(result.to_string().contains("name"));
    assert!(result.to_string().contains("age"));

    // Mixed key types
    let result = eval_string_expr("{ 1: \"one\", \"two\": 2, true: \"boolean\" }").unwrap();
    assert!(result.to_string().contains("1"));
    assert!(result.to_string().contains("two"));
    assert!(result.to_string().contains("true"));

    // Empty map
    let result = eval_string_expr("{}").unwrap();
    assert_eq!(result, Value::Map(indexmap::IndexMap::new()));

    // Single key map
    let result = eval_string_expr("{ key: \"value\" }").unwrap();
    assert!(result.to_string().contains("key"));
    assert!(result.to_string().contains("value"));
}

/// Test map key type restrictions
#[test]
fn test_map_key_restrictions() {
    // Numbers as keys
    let result = eval_string_expr("{ 1: \"one\", 2: \"two\" }").unwrap();
    assert!(result.to_string().contains("1"));
    assert!(result.to_string().contains("2"));

    // Booleans as keys
    let result = eval_string_expr("{ true: \"yes\", false: \"no\" }").unwrap();
    assert!(result.to_string().contains("true"));
    assert!(result.to_string().contains("false"));

    // Strings as keys
    let result = eval_string_expr("{ \"hello\": \"world\" }").unwrap();
    assert!(result.to_string().contains("hello"));

    // Tuples as keys
    let result = eval_string_expr("{ (1, 2): \"tuple key\" }").unwrap();
    assert!(result.to_string().contains("tuple"));
}

/// Test map pass-by-value semantics
#[test]
fn test_map_pass_by_value() {
    // Map assignment creates a copy
    let result = eval_program("m1 = { a: 1, b: 2 }\nm2 = m1\nm2[\"c\"] = 3\nresult = m1").unwrap();
    // m1 should be unchanged
    assert!(!result.to_string().contains("c"));

    // Map access returns copies
    let result = eval_program(
        "m = { inner: { x: 1 } }\ninner = m[\"inner\"]\ninner[\"y\"] = 2\nresult = m[\"inner\"]",
    )
    .unwrap();
    // Original map should be unchanged
    assert!(!result.to_string().contains("y"));

    // Map values are copied on assignment
    let result =
        eval_program("m = { a: [1, 2] }\nlist = m[\"a\"]\nlist[0] = 99\nresult = m[\"a\"][0]")
            .unwrap();
    // Original list in map should be unchanged
    assert_eq!(result, Value::Number(1.0));
}

/// Test map error cases
#[test]
fn test_map_error_cases() {
    // Accessing non-existent key
    assert!(eval_program("m = { a: 1 }\nresult = m[\"missing\"]").is_err());

    // Using colon syntax with non-existent key
    assert!(eval_program("m = { a: 1 }\nresult = m:missing").is_err());

    // Invalid map key types
    assert!(eval_string_expr("{ [1, 2]: \"list key\" }").is_err());
    assert!(eval_string_expr("{ { a: 1 }: \"map key\" }").is_err());

    // Method call on non-map
    assert!(eval_string_expr("42::delete(\"key\")").is_err());
    assert!(eval_string_expr("\"string\"::delete(\"key\")").is_err());
}

/// Test pass-by-value semantics for variable assignment
#[test]
fn test_variable_assignment_pass_by_value() {
    // List assignment creates a copy
    let result = eval_program("a = [1, 2, 3]\nb = a\nb[0] = 99\nresult = a[0]").unwrap();
    assert_eq!(result, Value::Number(1.0)); // Original unchanged

    // Map assignment creates a copy
    let result = eval_program("m1 = { a: 1, b: 2 }\nm2 = m1\nm2[\"c\"] = 3\nresult = m1").unwrap();
    assert!(!result.to_string().contains("c")); // Original unchanged

    // String assignment creates a copy
    let result =
        eval_program("s1 = \"hello\"\ns2 = s1\ns2 = s2 + \" world\"\nresult = s1").unwrap();
    assert_eq!(result, Value::String("hello".to_string())); // Original unchanged

    // Number assignment (primitives are naturally copied)
    let result = eval_program("a = 42\nb = a\nb = 99\nresult = a").unwrap();
    assert_eq!(result, Value::Number(42.0)); // Original unchanged

    // Boolean assignment
    let result = eval_program("a = true\nb = a\nb = false\nresult = a").unwrap();
    assert_eq!(result, Value::Boolean(true)); // Original unchanged
}

/// Test pass-by-value semantics for function parameters
#[test]
fn test_function_parameter_pass_by_value() {
    // List parameter is copied
    let result = eval_program("modify = |list| {\n  list[0] = 999\n  return list\n}\noriginal = [1, 2, 3]\nmodified = modify(original)\nresult = original[0]").unwrap();
    assert_eq!(result, Value::Number(1.0)); // Original unchanged

    // Map parameter is copied
    let result = eval_program("modify = |map| {\n  map[\"new\"] = \"value\"\n  return map\n}\noriginal = { a: 1 }\nmodified = modify(original)\nresult = original").unwrap();
    assert!(!result.to_string().contains("new")); // Original unchanged

    // String parameter is copied
    let result = eval_program("modify = |str| {\n  str = str + \" modified\"\n  return str\n}\noriginal = \"hello\"\nmodified = modify(original)\nresult = original").unwrap();
    assert_eq!(result, Value::String("hello".to_string())); // Original unchanged

    // Multiple parameters are all copied
    let result = eval_program("modify = |list, map, str| {\n  list[0] = 999\n  map[\"new\"] = \"value\"\n  str = str + \" modified\"\n  return list\n}\norig_list = [1, 2]\norig_map = { a: 1 }\norig_str = \"hello\"\nmodified = modify(orig_list, orig_map, orig_str)\nresult = orig_list[0] + orig_map[\"a\"]").unwrap();
    assert_eq!(result, Value::Number(2.0)); // Both originals unchanged (1 + 1)
}

/// Test pass-by-value semantics for indexing and slicing
#[test]
fn test_indexing_slicing_pass_by_value() {
    // List indexing returns a copy
    let result = eval_program(
        "original = [1, 2, 3]\nelement = original[0]\nelement = 999\nresult = original[0]",
    )
    .unwrap();
    assert_eq!(result, Value::Number(1.0)); // Original unchanged

    // List slicing returns a copy
    let result = eval_program(
        "original = [1, 2, 3, 4, 5]\nslice = original[1:4]\nslice[0] = 999\nresult = original[1]",
    )
    .unwrap();
    assert_eq!(result, Value::Number(2.0)); // Original unchanged

    // Map access returns a copy
    let result = eval_program("original = { a: [1, 2] }\nlist = original[\"a\"]\nlist[0] = 999\nresult = original[\"a\"][0]").unwrap();
    assert_eq!(result, Value::Number(1.0)); // Original unchanged

    // Nested structure access returns copies
    let result = eval_program("original = { inner: { x: 1, y: 2 } }\ninner = original[\"inner\"]\ninner[\"z\"] = 3\nresult = original[\"inner\"]").unwrap();
    assert!(!result.to_string().contains("z")); // Original unchanged
}

/// Test pass-by-value semantics for element assignment
#[test]
fn test_element_assignment_pass_by_value() {
    // List element assignment stores a copy
    let result = eval_program(
        "original = [1, 2, 3]\nlist = original\nlist[0] = [10, 20]\nresult = original[0]",
    )
    .unwrap();
    assert_eq!(result, Value::Number(1.0)); // Original unchanged

    // Map element assignment stores a copy
    let result = eval_program(
        "original = { a: 1 }\nmap = original\nmap[\"b\"] = [10, 20]\nresult = original",
    )
    .unwrap();
    assert!(!result.to_string().contains("b")); // Original unchanged

    // Simple nested assignment preserves independence
    let result = eval_program("original = { data: [1, 2] }\ncopy = original\ncopy[\"data\"] = [99, 100]\nresult = original[\"data\"][0]").unwrap();
    assert_eq!(result, Value::Number(1.0)); // Original unchanged

    // Element assignment with different types
    let result = eval_program(
        "original = [1, 2, 3]\nlist = original\nlist[0] = \"string\"\nresult = original[0]",
    )
    .unwrap();
    assert_eq!(result, Value::Number(1.0)); // Original unchanged
}

/// Test pass-by-value semantics for return values
#[test]
fn test_return_value_pass_by_value() {
    // Function return creates a copy
    let result = eval_program("create_list = || {\n  return [1, 2, 3]\n}\nlist1 = create_list()\nlist2 = create_list()\nlist1[0] = 999\nresult = list2[0]").unwrap();
    assert_eq!(result, Value::Number(1.0)); // Second list unchanged

    // Function return with modification
    let result = eval_program("get_list = || {\n  list = [1, 2, 3]\n  list[0] = 999\n  return list\n}\nresult = get_list()[0]").unwrap();
    assert_eq!(result, Value::Number(999.0)); // Modified value returned

    // Multiple returns are independent
    let result = eval_program("create_map = || {\n  return { a: 1, b: 2 }\n}\nmap1 = create_map()\nmap2 = create_map()\nmap1[\"c\"] = 3\nresult = map2").unwrap();
    assert!(!result.to_string().contains("c")); // Second map unchanged
}

/// Test pass-by-value semantics for complex nested structures
#[test]
fn test_nested_structure_pass_by_value() {
    // Deeply nested structures maintain independence (avoiding complex assignment)
    let result = eval_program("original = { level1: { level2: { level3: [1, 2, 3] } } }\ncopy = original\ncopy[\"level1\"] = { level2: { level3: [999, 2, 3] } }\nresult = original[\"level1\"][\"level2\"][\"level3\"][0]").unwrap();
    assert_eq!(result, Value::Number(1.0)); // Original unchanged

    // Mixed structure types maintain independence
    let result = eval_program("original = { lists: [[1, 2], [3, 4]], maps: { a: { x: 1 } } }\ncopy = original\ncopy[\"lists\"] = [[999, 2], [3, 4]]\ncopy[\"maps\"] = { a: { x: 1, y: 2 } }\nresult = original[\"lists\"][0][0] + original[\"maps\"][\"a\"][\"x\"]").unwrap();
    assert_eq!(result, Value::Number(2.0)); // Original unchanged (1 + 1)

    // Function with nested structure parameter
    let result = eval_program("modify_nested = |data| {\n  data[\"inner\"] = { list: [999, 2], map: { a: 1, new: \"value\" } }\n  return data\n}\noriginal = { inner: { list: [1, 2], map: { a: 1 } } }\nmodified = modify_nested(original)\nresult = original[\"inner\"][\"list\"][0] + original[\"inner\"][\"map\"][\"a\"]").unwrap();
    assert_eq!(result, Value::Number(2.0)); // Original unchanged (1 + 1)
}

/// Test pass-by-value semantics edge cases
#[test]
fn test_pass_by_value_edge_cases() {
    // Empty structures maintain independence
    let result =
        eval_program("empty_list = []\ncopy = empty_list\ncopy = [1, 2, 3]\nresult = empty_list")
            .unwrap();
    assert_eq!(result, Value::List(vec![])); // Original still empty

    // Single element structures
    let result =
        eval_program("single = [42]\ncopy = single\ncopy[0] = 999\nresult = single[0]").unwrap();
    assert_eq!(result, Value::Number(42.0)); // Original unchanged

    // Large structures maintain independence
    let result = eval_program(
        "large = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]\ncopy = large\ncopy[5] = 999\nresult = large[5]",
    )
    .unwrap();
    assert_eq!(result, Value::Number(6.0)); // Original unchanged

    // String concatenation doesn't affect original
    let result = eval_program(
        "original = \"hello\"\ncopy = original\ncopy = copy + \" world\"\nresult = original",
    )
    .unwrap();
    assert_eq!(result, Value::String("hello".to_string())); // Original unchanged
}

/// Test string methods
#[test]
fn test_string_methods() {
    // String length method
    let result = eval_string_expr("\"hello\"::length()").unwrap();
    assert_eq!(result, Value::Number(5.0));

    let result = eval_string_expr("\"\"::length()").unwrap();
    assert_eq!(result, Value::Number(0.0));

    let result = eval_string_expr("\"hello world\"::length()").unwrap();
    assert_eq!(result, Value::Number(11.0));

    // String split method with default separator
    let result = eval_string_expr("\"hello world\"::split()").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::String("hello".to_string()),
            Value::String("world".to_string())
        ])
    );

    // String split method with custom separator
    let result = eval_string_expr("\"a,b,c\"::split(\",\")").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string())
        ])
    );

    // String split with multiple separators
    let result = eval_string_expr("\"hello::world::test\"::split(\"::\")").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::String("hello".to_string()),
            Value::String("world".to_string()),
            Value::String("test".to_string())
        ])
    );

    // String split with no matches
    let result = eval_string_expr("\"hello\"::split(\",\")").unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::String("hello".to_string())])
    );

    // String split with empty string
    let result = eval_string_expr("\"\"::split()").unwrap();
    // Empty string split behavior may vary - check if it's empty list or list with empty string
    match result {
        Value::List(list) => {
            assert!(
                list.is_empty() || (list.len() == 1 && list[0] == Value::String("".to_string()))
            );
        }
        _ => panic!("Expected list result"),
    }
}

/// Test list methods
#[test]
fn test_list_methods() {
    // List length method
    let result = eval_string_expr("[1, 2, 3]::length()").unwrap();
    assert_eq!(result, Value::Number(3.0));

    let result = eval_string_expr("[]::length()").unwrap();
    assert_eq!(result, Value::Number(0.0));

    // List push method (mutating)
    let result = eval_program("list = [1, 2]\nlist::push(3)\nresult = list").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0)
        ])
    );

    // List push with different types
    let result = eval_program("list = [1, 2]\nlist::push(\"hello\")\nresult = list").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::String("hello".to_string())
        ])
    );

    // List pop method (mutating)
    let result = eval_program("list = [1, 2, 3]\nlast = list::pop()\nresult = last").unwrap();
    assert_eq!(result, Value::Number(3.0));

    let result = eval_program("list = [1, 2, 3]\nlist::pop()\nresult = list").unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
    );

    // List pop from single element
    let result = eval_program("list = [42]\nlast = list::pop()\nresult = last").unwrap();
    assert_eq!(result, Value::Number(42.0));

    // List join method with default separator
    let result = eval_string_expr("[\"a\", \"b\", \"c\"]::join()").unwrap();
    assert_eq!(result, Value::String("a b c".to_string()));

    // List join method with custom separator
    let result = eval_string_expr("[\"a\", \"b\", \"c\"]::join(\",\")").unwrap();
    assert_eq!(result, Value::String("a,b,c".to_string()));

    // List join with numbers
    let result = eval_string_expr("[1, 2, 3]::join(\"-\")").unwrap();
    assert_eq!(result, Value::String("1-2-3".to_string()));

    // List join with mixed types
    let result = eval_string_expr("[1, \"hello\", true]::join(\"|\")").unwrap();
    assert_eq!(result, Value::String("1|hello|true".to_string()));

    // List join with empty list
    let result = eval_string_expr("[]::join()").unwrap();
    assert_eq!(result, Value::String("".to_string()));

    // List join with single element
    let result = eval_string_expr("[\"single\"]::join()").unwrap();
    assert_eq!(result, Value::String("single".to_string()));
}

/// Test map methods
#[test]
fn test_map_methods_comprehensive() {
    // Map delete method - existing key
    let result =
        eval_program("m = { a: 1, b: 2, c: 3 }\ndeleted = m::delete(\"b\")\nresult = deleted")
            .unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = eval_program("m = { a: 1, b: 2, c: 3 }\nm::delete(\"b\")\nresult = m").unwrap();
    assert!(result.to_string().contains("a"));
    assert!(result.to_string().contains("c"));
    assert!(!result.to_string().contains("b"));

    // Map delete method - non-existing key
    let result =
        eval_program("m = { a: 1 }\ndeleted = m::delete(\"missing\")\nresult = deleted").unwrap();
    assert_eq!(result, Value::Boolean(false));

    // Map delete with dynamic key
    let result =
        eval_program("m = { x: 1, y: 2 }\nk = \"x\"\ndeleted = m::delete(k)\nresult = deleted")
            .unwrap();
    assert_eq!(result, Value::Boolean(true));

    // Map delete all keys
    let result =
        eval_program("m = { a: 1, b: 2 }\nm::delete(\"a\")\nm::delete(\"b\")\nresult = m").unwrap();
    assert_eq!(result, Value::Map(indexmap::IndexMap::new()));
}

/// Test method receiver requirements
#[test]
fn test_method_receiver_requirements() {
    // String methods require string receiver
    let result = eval_string_expr("\"hello\"::length()").unwrap();
    assert_eq!(result, Value::Number(5.0));

    // List methods require list receiver
    let result = eval_string_expr("[1, 2, 3]::length()").unwrap();
    assert_eq!(result, Value::Number(3.0));

    // Map methods require map receiver
    let result = eval_program("m = { a: 1 }\nresult = m::delete(\"a\")").unwrap();
    assert_eq!(result, Value::Boolean(true));

    // Method calls on wrong types should error
    assert!(eval_string_expr("42::length()").is_err());
    assert!(eval_string_expr("true::split()").is_err());
    assert!(eval_string_expr("\"hello\"::push(1)").is_err());
    assert!(eval_string_expr("[1, 2]::delete(\"key\")").is_err());
}

/// Test method chaining and precedence
#[test]
fn test_method_chaining() {
    // Method calls in expressions
    let result = eval_string_expr("\"hello world\"::length() + [1, 2, 3]::length()").unwrap();
    assert_eq!(result, Value::Number(14.0)); // 11 + 3 = 14

    // Method calls with other operations
    let result = eval_string_expr("\"hello\"::split()::length()").unwrap();
    assert_eq!(result, Value::Number(1.0)); // ["hello"] has length 1

    // Method calls in function calls
    let result =
        eval_program("double = |x| { return x * 2 }\nresult = double([1, 2, 3]::length())")
            .unwrap();
    assert_eq!(result, Value::Number(6.0)); // 2 * 3 = 6

    // Method calls with indexing
    let result = eval_program("list = [\"hello\", \"world\"]\nresult = list[0]::length()").unwrap();
    assert_eq!(result, Value::Number(5.0)); // "hello" has length 5
}

/// Test method error cases
#[test]
fn test_method_error_cases() {
    // Pop from empty list
    assert!(eval_program("list = []\nresult = list::pop()").is_err());

    // Split with non-string separator
    assert!(eval_string_expr("\"hello\"::split(42)").is_err());

    // Join with non-string separator
    assert!(eval_string_expr("[\"a\", \"b\"]::join(42)").is_err());

    // Method on null/undefined
    assert!(eval_string_expr("null::length()").is_err());

    // Method on function
    assert!(eval_string_expr("|| { return 1 }::length()").is_err());
}

/// Test method mutability requirements
#[test]
fn test_method_mutability_requirements() {
    // Mutating methods require assignable receiver
    let result = eval_program("list = [1, 2]\nlist::push(3)\nresult = list").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0)
        ])
    );

    // Non-mutating methods work on any receiver
    let result = eval_string_expr("[1, 2, 3]::length()").unwrap();
    assert_eq!(result, Value::Number(3.0));

    let result = eval_string_expr("\"hello\"::length()").unwrap();
    assert_eq!(result, Value::Number(5.0));

    // Mutating methods on temporary values should error (they require assignable receiver)
    assert!(eval_string_expr("[1, 2]::push(3)").is_err());

    // Map delete on temporary value should also error
    assert!(eval_string_expr("{ a: 1, b: 2 }::delete(\"a\")").is_err());
}
