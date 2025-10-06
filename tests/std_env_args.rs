use suji_lang::runtime::builtins;
use suji_lang::runtime::env::Env;
use suji_lang::runtime::value::{MapKey, Value};

#[test]
fn std_env_args_and_argv_present_and_shaped() {
    let env = std::rc::Rc::new(Env::new());
    builtins::setup_global_env(&env);

    let std_val = env.get("std").expect("std in global env");
    let std_map = match std_val {
        Value::Map(m) => m,
        _ => panic!("std should be a map"),
    };

    let env_val = std_map
        .get(&MapKey::String("env".to_string()))
        .expect("std:env");
    let env_map = match env_val {
        Value::Map(m) => m,
        _ => panic!("std:env should be a map"),
    };

    // args exists and is a map
    let args_val = env_map
        .get(&MapKey::String("args".to_string()))
        .expect("std:env:args");
    let args_map = match args_val {
        Value::Map(m) => m,
        _ => panic!("std:env:args should be a map"),
    };

    // argv exists and is a map
    let argv_val = env_map
        .get(&MapKey::String("argv".to_string()))
        .expect("std:env:argv");
    let argv_map = match argv_val {
        Value::Map(m) => m,
        _ => panic!("std:env:argv should be a map"),
    };

    // Lengths should match
    assert_eq!(args_map.len(), argv_map.len());
}
