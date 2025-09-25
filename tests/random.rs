mod common;
use common::eval_program_with_modules;

#[test]
fn random_import_and_seed() {
    let src = r#"
import std:random

random:seed(42)
val = random:random()
"#;
    let result = eval_program_with_modules(src).expect("eval ok");
    assert!(result.is_some());
}
