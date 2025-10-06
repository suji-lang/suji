use suji_lang::runtime::value::DecimalNumber;
mod common;

use common::eval_program;
use suji_lang::runtime::value::Value;

#[test]
fn test_yaml_parse_and_generate() {
    // Parse YAML string
    let program = r#"
import std:yaml

yaml_str = "name: Alice\nage: 30\nactive: true"
val = yaml:parse(yaml_str)
result = val:name
"#;
    let result = eval_program(program).unwrap();
    assert_eq!(result, Value::String("Alice".to_string()));

    // Generate YAML from SUJI value (map)
    let program2 = r#"
import std:yaml

user = { name: "Bob", age: 25, hobbies: ["reading", "coding"] }
out = yaml:generate(user)
# ensure some expected substrings exist
result = out::contains("name: Bob") && out::contains("hobbies:")
"#;
    let result2 = eval_program(program2).unwrap();
    assert_eq!(result2, Value::Boolean(true));

    // Parse YAML list
    let program3 = r#"
import std:yaml

nums_yaml = "- 1\n- 2\n- 3"
nums = yaml:parse(nums_yaml)
result = nums[1]
"#;
    let result3 = eval_program(program3).unwrap();
    assert_eq!(result3, Value::Number(DecimalNumber::from_i64(2)));
}

#[test]
fn test_toml_parse_and_generate() {
    // Parse TOML string
    let program = r#"
import std:toml

toml_str = "name = \"Alice\"\nage = 30\nactive = true"
val = toml:parse(toml_str)
result = val:name
"#;
    let result = eval_program(program).unwrap();
    assert_eq!(result, Value::String("Alice".to_string()));

    // Generate TOML from SUJI value (map)
    let program2 = r#"
import std:toml

user = { name: "Bob", age: 25, hobbies: ["reading", "coding"] }
out = toml:generate(user)
# expected substrings
result = out::contains("name = \"Bob\"") && out::contains("hobbies = [\"reading\", \"coding\"]")
"#;
    let result2 = eval_program(program2).unwrap();
    assert_eq!(result2, Value::Boolean(true));

    // Parse TOML array in table
    let program3 = r#"
import std:toml

cfg = toml:parse("values = [1, 2, 3]")
vals = cfg:values
result = vals[2]
"#;
    let result3 = eval_program(program3).unwrap();
    assert_eq!(result3, Value::Number(DecimalNumber::from_i64(3)));
}

#[test]
fn test_json_parse_and_generate() {
    // Parse JSON string
    let program = r#"
import std:json

json_str = "{\"name\": \"Alice\", \"age\": 30, \"active\": true}"
val = json:parse(json_str)
result = val:name
"#;
    let result = eval_program(program).unwrap();
    assert_eq!(result, Value::String("Alice".to_string()));

    // Generate JSON from SUJI value (map)
    let program2 = r#"
import std:json

user = { name: "Bob", age: 25, hobbies: ["reading", "coding"] }
out = json:generate(user)
# expected substrings
result = out::contains("\"name\":\"Bob\"") && out::contains("\"hobbies\":[\"reading\",\"coding\"]")
"#;
    let result2 = eval_program(program2).unwrap();
    assert_eq!(result2, Value::Boolean(true));

    // Parse JSON array
    let program3 = r#"
import std:json

arr = json:parse("[1, 2, 3, 4]")
result = arr[2]
"#;
    let result3 = eval_program(program3).unwrap();
    assert_eq!(result3, Value::Number(DecimalNumber::from_i64(3)));
}
