use nnlang::runtime::value::DecimalNumber;
mod common;

use common::eval_program;
use nnlang::runtime::value::Value;

#[test]
fn test_function_basics_and_defaults() {
    let result = eval_program("add = |x, y| { return x + y }\nresult = add(3, 4)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(7)));

    let result = eval_program("square = |x| { return x * x }\nresult = square(5)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(25)));

    let result = eval_program("getfive = || { return 5 }\nresult = getfive()").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));

    let result =
        eval_program("ispositive = |x| { return x > 0 }\nresult = ispositive(-3)").unwrap();
    assert_eq!(result, Value::Boolean(false));

    let result =
        eval_program("greet = |name| { return \"Hello, ${name}!\" }\nresult = greet(\"world\")")
            .unwrap();
    assert_eq!(result, Value::String("Hello, world!".to_string()));

    let result =
        eval_program("greet = |name = \"world\"| { return \"Hello, ${name}!\" }\nresult = greet()")
            .unwrap();
    assert_eq!(result, Value::String("Hello, world!".to_string()));

    let result = eval_program(
        "greet = |name = \"world\"| { return \"Hello, ${name}!\" }\nresult = greet(\"Alice\")",
    )
    .unwrap();
    assert_eq!(result, Value::String("Hello, Alice!".to_string()));
}

#[test]
fn test_closures_and_hof() {
    let result = eval_program("x = 10\naddx = |y| { return x + y }\nresult = addx(5)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(15)));

    let result = eval_program("makeadder = |base| { return |x| { return base + x } }\nadd10 = makeadder(10)\nresult = add10(7)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(17)));

    let result = eval_program("makeadder = |base| { return |x| { return base + x } }\nadd5 = makeadder(5)\nadd20 = makeadder(20)\nresult1 = add5(3)\nresult2 = add20(3)\nresult = result1 + result2").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(31)));

    let result = eval_program("counter = 0\nincrement = || { counter = counter + 1; return counter }\nfirst = increment()\nsecond = increment()\nresult = second").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));

    let result = eval_program(
        "apply = |f, x| { return f(x) }\ndouble = |n| { return n * 2 }\nresult = apply(double, 6)",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(12)));

    let result = eval_program("makemultiplier = |factor| { return |x| { return x * factor } }\ntriple = makemultiplier(3)\nresult = triple(4)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(12)));

    let result = eval_program(
        "add1 = |x| { return x + 1 }\nmul2 = |x| { return x * 2 }\nresult = mul2(add1(5))",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(12)));

    let result = eval_program("compose = |f, g| { return |x| { return f(g(x)) } }\nadd3 = |x| { return x + 3 }\nmul2 = |x| { return x * 2 }\ncomposed = compose(mul2, add3)\nresult = composed(4)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(14)));
}

#[test]
fn test_function_scope_and_loops() {
    let result = eval_program("x = 10\ntest = |x| { return x * 2 }\nresult = test(5)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(10)));

    let result =
        eval_program("outer = 100\ntest = |inner| { return inner + outer }\nresult = test(5)")
            .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(105)));

    let result = eval_program(
        "x = 1\ntest = || { x = 2; inner = || { return x }; return inner() }\nresult = test()",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));

    let result = eval_program("x = 1\ntest = || { x = 2; return x }\ntest()\nresult = x").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));

    let result = eval_program(
        "counter = 0\nloop { counter++; match counter { 5 => { break }, } }\nresult = counter",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));

    let result = eval_program("sum = 0\ni = 0\nloop { i++; match i { 3 => { continue }, 6 => { break }, } sum = sum + i }\nresult = sum").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(12)));

    let result = eval_program("count = 0\nloop through 0..5 { count++ }\nresult = count").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));
}

#[test]
fn test_loop_through_bindings_and_match() {
    let result =
        eval_program("sum = 0\nloop through 0..5 with i { sum = sum + i }\nresult = sum").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(10)));

    let result = eval_program(
        "nums = [10, 20, 30]\nsum = 0\nloop through nums with n { sum = sum + n }\nresult = sum",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(60)));

    let result = eval_program(
        "nums = [1, 2, 3]\nsum = 0\nloop through nums with n { sum = sum + (n * 2) }\nresult = sum",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(12)));

    let result = eval_program("count = 0\nloop through 0..3 { count++ }\nresult = count").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(3)));

    let result =
        eval_program("i = 0\nloop as outer { i++; match i { 3 => { break outer }, } }\nresult = i")
            .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(3)));

    let result = eval_program("counter = 0\nloop as my_loop { counter++; match counter { 5 => { break my_loop }, } }\nresult = counter").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));

    let result = eval_program(
        "x = 42\nmatch x { 42 => { result = \"found\" }, 99 => { result = \"not found\" }, }",
    )
    .unwrap();
    assert_eq!(result, Value::String("found".to_string()));

    let result = eval_program(
        "flag = true\nmatch flag { false => { result = \"false\" }, true => { result = \"true\" }, }",
    )
    .unwrap();
    assert_eq!(result, Value::String("true".to_string()));

    let result = eval_program("x = 2\nmatch x { 1 => { result = \"one\" }, 2 => { result = \"two\" }, 3 => { result = \"three\" }, }").unwrap();
    assert_eq!(result, Value::String("two".to_string()));

    let result = eval_program(
        "x = 5\nmatch x { 5 => { result = \"first\" }, 5 => { result = \"second\" }, }",
    )
    .unwrap();
    assert_eq!(result, Value::String("first".to_string()));

    let result = eval_program(
        "x = 99\nmatch x { 1 => { result = \"one\" }, _ => { result = \"wildcard\" }, }",
    )
    .unwrap();
    assert_eq!(result, Value::String("wildcard".to_string()));

    let result = eval_program(
        "x = 1\nmatch x { 1 => { result = \"one\" }, _ => { result = \"wildcard\" }, }",
    )
    .unwrap();
    assert_eq!(result, Value::String("one".to_string()));
}

#[test]
fn test_match_patterns_regex_tuples_and_nil() {
    // Regex pattern in match arm
    let result = eval_program(
        "s = \"user_123\"\nresult = match s { /^user_[0-9]+$/ => \"ok\", _ => \"no\", }",
    )
    .unwrap();
    assert_eq!(result, Value::String("ok".to_string()));

    // Tuple patterns and wildcards
    let result = eval_program(
        "t = (2, 10, 0)\nresult = match t {\n  (3, 4, 0) => \"exact\",\n  (2, 10, 0) => \"partial\",\n  _ => \"none\",\n }",
    )
    .unwrap();
    assert_eq!(result, Value::String("partial".to_string()));

    // Nil in match arms
    let result =
        eval_program("x = nil\nresult = match x { nil => \"none\", _ => \"some\", }").unwrap();
    assert_eq!(result, Value::String("none".to_string()));
}

#[test]
fn test_match_without_expression_and_method_calls_in_conditions() {
    // Conditional match (no expression)
    let result = eval_program(
        "x = 5\nresult = match { x > 10 => \"big\", x > 0 => \"positive\", _ => \"other\", }",
    )
    .unwrap();
    assert_eq!(result, Value::String("positive".to_string()));

    // Method calls in conditions
    let result = eval_program(
        "text = \"hello world\"\nresult = match { text::contains(\"world\") => \"has world\", text::length() > 10 => \"long\", _ => \"short\", }",
    )
    .unwrap();
    assert_eq!(result, Value::String("has world".to_string()));

    // Labeled continue example
    let result = eval_program(
        "outer = 0\nloop as outer_loop {\n  match outer { 2 => { break outer_loop }, }\n  inner = 0\n  loop {\n    inner++\n    match inner { 3 => { outer++; continue outer_loop }, }\n  }\n}\nresult = outer",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));
}

#[test]
fn test_tuple_wildcard_pattern() {
    let result =
        eval_program("t = (2, 3)\nresult = match t { (2, _) => \"ok\", _ => \"no\", }").unwrap();
    assert_eq!(result, Value::String("ok".to_string()));
}
