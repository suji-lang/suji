use suji_lang::runtime::env::Env;
use suji_lang::runtime::module::ModuleRegistry;
use suji_lang::runtime::value::{MapKey, Value};

#[test]
fn std_env_args_and_argv_present_and_shaped() {
    let env = std::rc::Rc::new(Env::new());
    let registry = ModuleRegistry::new();

    // Load std module via registry
    let std_val = registry
        .resolve_module_path(&env, "std", false)
        .expect("std module should load");
    let std_map = match std_val {
        Value::Map(m) => m,
        _ => panic!("std should be a map"),
    };

    // Load std:env submodule (may be a lazy module, so force-load it)
    let env_val = std_map
        .get(&MapKey::String("env".to_string()))
        .expect("std:env");
    let env_val = match env_val {
        Value::Module(handle) => registry
            .force_load_module(handle)
            .expect("force load std:env"),
        other => other.clone(),
    };
    let env_map = match env_val {
        Value::Map(m) => m,
        _ => panic!("std:env should be a map after loading"),
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
