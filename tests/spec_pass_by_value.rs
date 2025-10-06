use suji_lang::runtime::value::DecimalNumber;
mod common;

use common::eval_program;
use suji_lang::runtime::value::Value;

#[test]
fn test_variable_assignment_pass_by_value() {
    let result = eval_program("a = [1, 2, 3]\nb = a\nb[0] = 99\nresult = a[0]").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));

    let result = eval_program("m1 = { a: 1, b: 2 }\nm2 = m1\nm2[\"c\"] = 3\nresult = m1").unwrap();
    assert!(!result.to_string().contains("c"));

    let result =
        eval_program("s1 = \"hello\"\ns2 = s1\ns2 = s2 + \" world\"\nresult = s1").unwrap();
    assert_eq!(result, Value::String("hello".to_string()));

    let result = eval_program("a = 42\nb = a\nb = 99\nresult = a").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(42)));

    let result = eval_program("a = true\nb = a\nb = false\nresult = a").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_function_parameter_pass_by_value() {
    let result = eval_program("modify = |list| { list[0] = 999; return list }\noriginal = [1, 2, 3]\nmodified = modify(original)\nresult = original[0]").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));

    let result = eval_program("modify = |map| { map[\"new\"] = \"value\"; return map }\noriginal = { a: 1 }\nmodified = modify(original)\nresult = original").unwrap();
    assert!(!result.to_string().contains("new"));

    let result = eval_program("modify = |str| { str = str + \" modified\"; return str }\noriginal = \"hello\"\nmodified = modify(original)\nresult = original").unwrap();
    assert_eq!(result, Value::String("hello".to_string()));

    let result = eval_program("modify = |list, map, str| { list[0] = 999; map[\"new\"] = \"value\"; str = str + \" modified\"; return list }\norig_list = [1, 2]\norig_map = { a: 1 }\norig_str = \"hello\"\nmodified = modify(orig_list, orig_map, orig_str)\nresult = orig_list[0] + orig_map[\"a\"]").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));
}

#[test]
fn test_indexing_slicing_pass_by_value() {
    let result = eval_program(
        "original = [1, 2, 3]\nelement = original[0]\nelement = 999\nresult = original[0]",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));

    let result = eval_program(
        "original = [1, 2, 3, 4, 5]\nslice = original[1:4]\nslice[0] = 999\nresult = original[1]",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));

    let result = eval_program("original = { a: [1, 2] }\nlist = original[\"a\"]\nlist[0] = 999\nresult = original[\"a\"][0]").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));

    let result = eval_program("original = { inner: { x: 1, y: 2 } }\ninner = original[\"inner\"]\ninner[\"z\"] = 3\nresult = original[\"inner\"]").unwrap();
    assert!(!result.to_string().contains("z"));
}

#[test]
fn test_element_assignment_pass_by_value() {
    let result = eval_program(
        "original = [1, 2, 3]\nlist = original\nlist[0] = [10, 20]\nresult = original[0]",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));

    let result = eval_program(
        "original = { a: 1 }\nmap = original\nmap[\"b\"] = [10, 20]\nresult = original",
    )
    .unwrap();
    assert!(!result.to_string().contains("b"));

    let result = eval_program("original = { data: [1, 2] }\ncopy = original\ncopy[\"data\"] = [99, 100]\nresult = original[\"data\"][0]").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));

    let result = eval_program(
        "original = [1, 2, 3]\nlist = original\nlist[0] = \"string\"\nresult = original[0]",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));
}

#[test]
fn test_return_value_pass_by_value_and_nested() {
    let result = eval_program("create_list = || { return [1, 2, 3] }\nlist1 = create_list()\nlist2 = create_list()\nlist1[0] = 999\nresult = list2[0]").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));

    let result = eval_program(
        "get_list = || { list = [1, 2, 3]; list[0] = 999; return list }\nresult = get_list()[0]",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(999)));

    let result = eval_program("create_map = || { return { a: 1, b: 2 } }\nmap1 = create_map()\nmap2 = create_map()\nmap1[\"c\"] = 3\nresult = map2").unwrap();
    assert!(!result.to_string().contains("c"));
}
