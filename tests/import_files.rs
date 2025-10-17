use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use tempfile::tempdir;

use suji_lang::parser::parse_program;
use suji_lang::runtime::builtins::setup_global_env;
use suji_lang::runtime::env::Env;
use suji_lang::runtime::eval::eval_program_with_modules;
use suji_lang::runtime::module::ModuleRegistry;
use suji_lang::runtime::value::Value;

fn eval_in_dir(dir: &PathBuf, source: &str) -> Result<Option<Value>, Box<dyn std::error::Error>> {
    let stmts = parse_program(source)?;
    let env = Rc::new(Env::new());
    setup_global_env(&env);
    let registry = ModuleRegistry::new();
    registry.set_base_dir(dir);
    Ok(eval_program_with_modules(&stmts, env, &registry)?)
}

#[test]
fn import_file_leaf_and_map() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let root = tmp.path().to_path_buf();

    // one.si exports a leaf
    {
        let mut f = fs::File::create(root.join("one.si"))?;
        writeln!(f, "export 2")?;
    }
    // two.si exports a map
    {
        let mut f = fs::File::create(root.join("two.si"))?;
        writeln!(f, "export {{ a: 1, b: 2 }}")?;
    }

    // import one (leaf)
    let v = eval_in_dir(&root, "import one")?.unwrap_or(Value::Nil);
    assert!(
        matches!(v, Value::Nil),
        "import binds but program returns Nil"
    );

    // import two:a
    let v = eval_in_dir(&root, "import two:a; a")?;
    assert!(matches!(v, Some(Value::Number(_))));

    Ok(())
}

#[test]
fn import_nested_paths() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let root = tmp.path().to_path_buf();

    // a.si exports a map with key b
    {
        let mut f = fs::File::create(root.join("a.si"))?;
        writeln!(f, "export {{ b: 3 }}")?;
    }

    // a/b.si exports a leaf 4
    fs::create_dir_all(root.join("a"))?;
    {
        let mut f = fs::File::create(root.join("a").join("b.si"))?;
        writeln!(f, "export 4")?;
    }

    // a/b/ directory module with c.si
    fs::create_dir_all(root.join("a").join("b"))?;
    {
        let mut f = fs::File::create(root.join("a").join("b").join("c.si"))?;
        writeln!(f, "export 5")?;
    }

    // import a:b via a.si map
    let out = eval_in_dir(&root, "import a:b; b")?;
    assert!(matches!(out, Some(Value::Number(_))));

    // import a:b via a/b.si
    let out = eval_in_dir(&root, "import a:b; b")?;
    assert!(out.is_some());

    // import a:b:c via directory
    let out = eval_in_dir(&root, "import a:b:c as c; c")?;
    assert!(matches!(out, Some(Value::Number(_))));

    Ok(())
}

#[test]
fn import_errors() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let root = tmp.path().to_path_buf();

    // leaf module
    {
        let mut f = fs::File::create(root.join("leaf.si"))?;
        writeln!(f, "export 1")?;
    }

    // no export module
    {
        let mut f = fs::File::create(root.join("noexp.si"))?;
        writeln!(f, "x = 1")?;
    }

    // Cannot import item from leaf
    let err = eval_in_dir(&root, "import leaf:x").unwrap_err();
    assert!(format!("{}", err).contains("not a valid module"));

    // Missing export should surface
    let err = eval_in_dir(&root, "import noexp").unwrap_err();
    assert!(format!("{}", err).contains("has no export"));

    Ok(())
}
