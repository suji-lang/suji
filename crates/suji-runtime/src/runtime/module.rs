use super::value::{RuntimeError, Value};
use crate::runtime::builtins::list_builtins;
use crate::runtime::env::Env;
use crate::runtime::eval::eval_stmt_with_modules;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use suji_ast::ast::Stmt;

/// Result from virtual std resolution
#[derive(Debug, Clone)]
pub enum VirtualStdResult {
    /// A single file with its source content
    File(&'static str),
    /// A directory with list of child names (without .si extension)
    Directory(Vec<String>),
}

/// Callback type for resolving virtual std sources
/// Takes path segments and returns resolution result
pub type VirtualStdResolver = fn(&[&str]) -> Option<VirtualStdResult>;

/// Registry for managing modules in the SUJI language runtime
#[derive(Clone)]
pub struct ModuleRegistry {
    /// Built-in modules that are always available
    builtins: HashMap<String, Value>,
    /// Cache of loaded module values keyed by canonical absolute paths (files and directories)
    file_cache: RefCell<HashMap<PathBuf, Value>>,
    /// Stack of importer directories for relative resolution
    dir_stack: RefCell<Vec<PathBuf>>,
    /// Optional virtual std resolver (set by suji-stdlib)
    virtual_std_resolver: Option<VirtualStdResolver>,
}

// Manual Debug implementation since function pointers don't impl Debug
impl std::fmt::Debug for ModuleRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleRegistry")
            .field("builtins", &self.builtins.keys())
            .field("file_cache", &"<cached>")
            .field("dir_stack", &self.dir_stack)
            .field("virtual_std_resolver", &self.virtual_std_resolver.is_some())
            .finish()
    }
}

impl ModuleRegistry {
    /// Create a special built-in function value that can be called normally
    fn create_builtin_function_wrapper(name: &str) -> Value {
        use super::value::{FunctionValue, ParamSpec};
        use suji_ast::ast::Stmt as AstStmt;
        use suji_lexer::token::Span as AstSpan;

        Value::Function(FunctionValue {
            params: vec![ParamSpec {
                name: format!("__builtin_{}__", name),
                default: None,
            }],
            body: AstStmt::Block {
                statements: vec![],
                span: AstSpan::default(),
            },
            env: Rc::new(Env::new()),
        })
    }
    /// Create a new module registry with built-in modules
    pub fn new() -> Self {
        let mut registry = Self {
            builtins: HashMap::new(),
            file_cache: RefCell::new(HashMap::new()),
            dir_stack: RefCell::new(Vec::new()),
            virtual_std_resolver: None,
        };

        // Register built-in modules
        registry.register_builtin_modules();
        registry
    }

    /// Set the virtual std resolver (called by suji-stdlib during initialization)
    pub fn set_virtual_std_resolver(&mut self, resolver: VirtualStdResolver) {
        self.virtual_std_resolver = Some(resolver);
    }

    /// Create a clone of this registry with a custom override for the `std` module
    pub fn with_custom_std(&self, std_value: Value) -> Self {
        let mut new_registry = self.clone();
        new_registry.builtins.insert("std".to_string(), std_value);
        new_registry
    }

    /// Register all built-in modules
    fn register_builtin_modules(&mut self) {
        // Register the special __builtins__ virtual module
        let mut builtins_map: IndexMap<super::value::MapKey, Value> = IndexMap::new();

        // Add builtin function wrappers
        for name in list_builtins() {
            builtins_map.insert(
                super::value::MapKey::String(name.clone()),
                Self::create_builtin_function_wrapper(&name),
            );
        }

        // Add IO stream proxies (resolve dynamically via IoContext)
        builtins_map.insert(
            super::value::MapKey::String("io_stdin".to_string()),
            Value::StreamProxy(super::value::StreamProxyKind::Stdin),
        );
        builtins_map.insert(
            super::value::MapKey::String("io_stdout".to_string()),
            Value::StreamProxy(super::value::StreamProxyKind::Stdout),
        );
        builtins_map.insert(
            super::value::MapKey::String("io_stderr".to_string()),
            Value::StreamProxy(super::value::StreamProxyKind::Stderr),
        );

        // Add environment variable map (via EnvProxy)
        use super::env_overlay::EnvProxy;
        let env_proxy = Value::EnvMap(Rc::new(EnvProxy::new()));
        builtins_map.insert(
            super::value::MapKey::String("env_var".to_string()),
            env_proxy,
        );

        // Snapshot command-line arguments into maps at startup
        // Exclude interpreter name (argv[0]) and any interpreter options (leading '-' args)
        let mut args_map: IndexMap<super::value::MapKey, Value> = IndexMap::new();
        let mut iter = std::env::args();
        // Skip interpreter name
        let _ = iter.next();
        // Skip interpreter options (starting with '-') until first non-option (script path)
        let mut script_seen = false;
        for a in iter {
            if !script_seen {
                if a.starts_with('-') {
                    continue;
                } else {
                    // First non-option is script path; include as "0" and mark seen
                    args_map.insert(
                        super::value::MapKey::String("0".to_string()),
                        Value::String(a),
                    );
                    script_seen = true;
                    continue;
                }
            }
            // Subsequent args are positional starting from "1"
            let idx = args_map.len().saturating_sub(1); // exclude "0"
            args_map.insert(
                super::value::MapKey::String(idx.to_string()),
                Value::String(a),
            );
        }

        builtins_map.insert(
            super::value::MapKey::String("env_args".to_string()),
            Value::Map(args_map.clone()),
        );
        builtins_map.insert(
            super::value::MapKey::String("env_argv".to_string()),
            Value::Map(args_map),
        );

        self.builtins
            .insert("__builtins__".to_string(), Value::Map(builtins_map));
    }

    /// Resolve a module by name from built-ins (filesystem modules are handled via resolve_module_path)
    pub fn resolve_module(&self, name: &str) -> Result<Value, RuntimeError> {
        if let Some(module) = self.builtins.get(name) {
            return Ok(module.clone());
        }

        // Check for virtual std
        if name == "std"
            && let Ok(Some(std_module)) = self.resolve_virtual_std_root()
        {
            return Ok(std_module);
        }

        // Module not found
        Err(RuntimeError::InvalidOperation {
            message: format!("Module '{}' not found", name),
        })
    }

    /// Resolve a specific item from a module
    pub fn resolve_module_item(
        &self,
        module_name: &str,
        item_name: &str,
    ) -> Result<Value, RuntimeError> {
        let module = self.resolve_module(module_name)?;

        match module {
            Value::Map(map) => {
                let key = super::value::MapKey::String(item_name.to_string());
                match map.get(&key) {
                    Some(value) => Ok(value.clone()),
                    None => Err(RuntimeError::InvalidOperation {
                        message: format!(
                            "Item '{}' not found in module '{}'",
                            item_name, module_name
                        ),
                    }),
                }
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Module '{}' is not a valid module (not a map)", module_name),
            }),
        }
    }

    /// Resolve a nested module item (e.g., "std:json:parse")
    pub fn resolve_nested_module_item(
        &self,
        module_path: &str,
        item_name: &str,
    ) -> Result<Value, RuntimeError> {
        let parts: Vec<&str> = module_path.split(':').collect();
        if parts.is_empty() {
            return Err(RuntimeError::InvalidOperation {
                message: "Empty module path".to_string(),
            });
        }

        // Start with the root module
        let mut current_module = self.resolve_module(parts[0])?;

        // Navigate through nested modules
        for part in &parts[1..] {
            match current_module {
                Value::Map(map) => {
                    let key = super::value::MapKey::String(part.to_string());
                    match map.get(&key) {
                        Some(Value::Map(nested_map)) => {
                            current_module = Value::Map(nested_map.clone());
                        }
                        Some(_) => {
                            return Err(RuntimeError::InvalidOperation {
                                message: format!(
                                    "'{}' is not a module in '{}'",
                                    part,
                                    parts[..parts.len() - 1].join(":")
                                ),
                            });
                        }
                        None => {
                            return Err(RuntimeError::InvalidOperation {
                                message: format!(
                                    "Module '{}' not found in '{}'",
                                    part,
                                    parts[..parts.len() - 1].join(":")
                                ),
                            });
                        }
                    }
                }
                _ => {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("'{}' is not a valid module (not a map)", parts[0]),
                    });
                }
            }
        }

        // Now resolve the final item
        match current_module {
            Value::Map(map) => {
                let key = super::value::MapKey::String(item_name.to_string());
                match map.get(&key) {
                    Some(value) => Ok(value.clone()),
                    None => Err(RuntimeError::InvalidOperation {
                        message: format!(
                            "Item '{}' not found in module '{}'",
                            item_name, module_path
                        ),
                    }),
                }
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Module '{}' is not a valid module (not a map)", module_path),
            }),
        }
    }

    /// Check if a builtin module exists
    pub fn has_module(&self, name: &str) -> bool {
        self.builtins.contains_key(name)
    }

    /// Set the base importer directory (clears and initializes the dir stack)
    pub fn set_base_dir(&self, base: impl AsRef<Path>) {
        let mut stack = self.dir_stack.borrow_mut();
        stack.clear();
        stack.push(base.as_ref().to_path_buf());
    }

    /// Push a directory onto the importer dir stack, run f, then pop (even on panic)
    pub fn with_dir<F, R>(&self, dir: &Path, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        struct Guard<'a> {
            stack: &'a RefCell<Vec<PathBuf>>,
        }
        impl<'a> Drop for Guard<'a> {
            fn drop(&mut self) {
                let mut s = self.stack.borrow_mut();
                let _ = s.pop();
            }
        }

        {
            let mut stack = self.dir_stack.borrow_mut();
            stack.push(dir.to_path_buf());
        }
        let guard = Guard {
            stack: &self.dir_stack,
        };
        let result = f();
        drop(guard);
        result
    }

    /// Resolve a module path like "a:b:c" using env → filesystem → builtins
    /// When `prefer_dir_last` is true, prefer resolving the last segment as a directory module
    /// if both a map key on the previous segment and a subdirectory exist.
    pub fn resolve_module_path(
        &self,
        env: &Rc<Env>,
        module_path: &str,
        prefer_dir_last: bool,
    ) -> Result<Value, RuntimeError> {
        let parts: Vec<&str> = module_path.split(':').collect();
        if parts.is_empty() {
            return Err(RuntimeError::InvalidOperation {
                message: "Empty module path".to_string(),
            });
        }

        // 1) Try env-first for the root
        if let Ok(mut current) = env.get(parts[0]) {
            for part in &parts[1..] {
                match current {
                    Value::Map(ref map) => {
                        let key = super::value::MapKey::String((*part).to_string());
                        current = map.get(&key).cloned().ok_or_else(|| {
                            RuntimeError::InvalidOperation {
                                message: format!(
                                    "Module '{}' not found in '{}'",
                                    part,
                                    parts[..parts.len() - 1].join(":")
                                ),
                            }
                        })?;
                    }
                    _ => {
                        return Err(RuntimeError::InvalidOperation {
                            message: format!("'{}' is not a module (not a map)", parts[0]),
                        });
                    }
                }
            }
            return Ok(current);
        }

        // 2) Try filesystem resolution relative to current dir
        if let Ok(v) = self.resolve_via_fs_path(&parts, prefer_dir_last) {
            return Ok(v);
        }

        // 3) Builtins fallback for the root, then traverse
        let mut current = self.resolve_module(parts[0])?;
        for part in &parts[1..] {
            match current {
                Value::Map(ref map) => {
                    let key = super::value::MapKey::String((*part).to_string());
                    current =
                        map.get(&key)
                            .cloned()
                            .ok_or_else(|| RuntimeError::InvalidOperation {
                                message: format!(
                                    "Module '{}' not found in '{}'",
                                    part,
                                    parts[..parts.len() - 1].join(":")
                                ),
                            })?;
                }
                _ => {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("'{}' is not a module (not a map)", parts[0]),
                    });
                }
            }
        }
        Ok(current)
    }

    /// Resolve the root segment by trying env, then filesystem relative to current dir, then builtins
    pub fn resolve_module_root(&self, env: &Rc<Env>, root: &str) -> Result<Value, RuntimeError> {
        if let Ok(v) = env.get(root) {
            return Ok(v);
        }
        if let Some(v) = self.resolve_fs_root(root)? {
            return Ok(v);
        }
        self.resolve_module(root)
    }

    /// Try to resolve a root name from the filesystem relative to the current importer directory
    /// For "std", checks virtual std sources first
    fn resolve_fs_root(&self, name: &str) -> Result<Option<Value>, RuntimeError> {
        // Special case: check virtual std sources for "std" module
        // This works even when no dir_stack is set (e.g., in tests)
        if name == "std"
            && let Some(v) = self.resolve_virtual_std_root()?
        {
            return Ok(Some(v));
        }

        // If no dir_stack, can't resolve from filesystem
        let current_dir = {
            let stack_ref = self.dir_stack.borrow();
            match stack_ref.last() {
                Some(d) => d.clone(),
                None => return Ok(None),
            }
        };

        let file_path = current_dir.join(format!("{}.si", name));
        if file_path.exists() {
            let value = self.load_file_value(&file_path)?;
            return Ok(Some(value));
        }

        let dir_path = current_dir.join(name);
        if dir_path.is_dir() {
            let value = self.build_dir_module_map(&dir_path)?;
            return Ok(Some(value));
        }

        Ok(None)
    }

    /// Resolve "std" root from virtual std sources (builds a module map from all .si files)
    fn resolve_virtual_std_root(&self) -> Result<Option<Value>, RuntimeError> {
        // Build a module map from all virtual std sources at root level
        self.build_virtual_std_module_map(&[])
    }

    /// Build a module map from virtual std sources at given path segments
    fn build_virtual_std_module_map(
        &self,
        segments: &[&str],
    ) -> Result<Option<Value>, RuntimeError> {
        let resolver = match self.virtual_std_resolver {
            Some(r) => r,
            None => return Ok(None),
        };

        // Query for directory listing at this path
        match resolver(segments) {
            Some(VirtualStdResult::Directory(children)) => {
                // Build a module map from the children
                let mut map: IndexMap<super::value::MapKey, Value> = IndexMap::new();

                for child in children {
                    // Try to load each child as a file
                    let mut child_segments = segments.to_vec();
                    child_segments.push(&child);

                    match resolver(&child_segments) {
                        Some(VirtualStdResult::File(source)) => {
                            let path = format!("std/{}.si", child_segments.join("/"));
                            if let Some(value) = self.load_virtual_std_file(source, &path)? {
                                map.insert(super::value::MapKey::String(child.clone()), value);
                            }
                        }
                        Some(VirtualStdResult::Directory(_)) => {
                            // Nested directory - recursively build its module map
                            if let Some(value) =
                                self.build_virtual_std_module_map(&child_segments)?
                            {
                                map.insert(super::value::MapKey::String(child.clone()), value);
                            }
                        }
                        None => {
                            // Child not found, skip it
                        }
                    }
                }

                Ok(Some(Value::Map(map)))
            }
            Some(VirtualStdResult::File(_)) => {
                // Caller asked for a directory but got a file
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Load and evaluate a virtual std source file
    fn load_virtual_std_file(
        &self,
        source: &str,
        path: &str,
    ) -> Result<Option<Value>, RuntimeError> {
        // Check cache using virtual path
        let cache_key = PathBuf::from(format!("<virtual>/{}", path));
        if let Some(v) = self.file_cache.borrow().get(&cache_key) {
            return Ok(Some(v.clone()));
        }

        // Parse the source
        let statements = match suji_parser::parse_program(source) {
            Ok(stmts) => stmts,
            Err(e) => {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Parse error in virtual module '{}': {}", path, e),
                });
            }
        };

        // Evaluate with isolated environment
        let env = Rc::new(Env::new());
        let mut loop_stack = Vec::new();
        let mut export_value: Option<Value> = None;

        for stmt in &statements {
            let result = eval_stmt_with_modules(stmt, env.clone(), &mut loop_stack, self);
            match result {
                Ok(Some(v)) => {
                    if matches!(stmt, Stmt::Export { .. }) {
                        export_value = Some(v);
                    }
                }
                Ok(None) => {
                    // continue
                }
                Err(e) => return Err(e),
            }
        }

        let value = export_value.ok_or_else(|| RuntimeError::InvalidOperation {
            message: format!("Virtual module '{}' has no export", path),
        })?;

        // Cache the result
        self.file_cache
            .borrow_mut()
            .insert(cache_key, value.clone());
        Ok(Some(value))
    }

    /// Resolve a colon-separated path via filesystem rules, relative to current dir.
    /// Implements 0.1.16 resolution for multi-segment paths.
    fn resolve_via_fs_path(
        &self,
        parts: &[&str],
        prefer_dir_last: bool,
    ) -> Result<Value, RuntimeError> {
        if parts.is_empty() {
            return Err(RuntimeError::InvalidOperation {
                message: "Empty module path".to_string(),
            });
        }
        if parts.len() == 1 {
            if let Some(v) = self.resolve_fs_root(parts[0])? {
                return Ok(v);
            }
            return Err(RuntimeError::InvalidOperation {
                message: format!("Module '{}' not found", parts[0]),
            });
        }

        // Current importer directory
        let current_dir = {
            let stack_ref = self.dir_stack.borrow();
            match stack_ref.last() {
                Some(d) => d.clone(),
                None => PathBuf::from("."),
            }
        };

        // A) Try D/a.si then fetch parts[1] from map. If this route proves non-map before consuming
        // all segments, fall back to B/C instead of erroring immediately.
        let a_file = current_dir.join(format!("{}.si", parts[0]));
        if a_file.exists() {
            let root_val = self.load_file_value(&a_file)?;
            let mut current = root_val;
            let mut a_failed_nonmap = false;
            for (i, part) in parts[1..].iter().enumerate() {
                match current {
                    Value::Map(ref map) => {
                        let key = super::value::MapKey::String((*part).to_string());
                        match map.get(&key).cloned() {
                            Some(v) => current = v,
                            None => {
                                // Missing key along A-route: treat as hard error for this route and let fallthrough try B/C
                                a_failed_nonmap = true;
                                break;
                            }
                        }
                    }
                    _ => {
                        // Became a leaf while segments remain
                        if i + 1 < parts[1..].len() {
                            a_failed_nonmap = true;
                            break;
                        }
                    }
                }
            }
            if !a_failed_nonmap {
                // If caller intends to access an item from this base (prefer_dir_last) and
                // the resolved base is not a module map while an alternative directory exists,
                // fall through to try directory/file alternatives.
                if prefer_dir_last && parts.len() == 2 {
                    let ab_dir = current_dir.join(parts[0]).join(parts[1]);
                    let ab_file = current_dir.join(parts[0]).join(format!("{}.si", parts[1]));
                    if ab_dir.is_dir() || ab_file.exists() {
                        if !matches!(current, Value::Map(_)) {
                            // fall through to B/C
                        } else {
                            return Ok(current);
                        }
                    } else {
                        return Ok(current);
                    }
                } else {
                    return Ok(current);
                }
            }
            // else fall through to try B/C
        }

        // B) Try directory D/a/b/ (prefer directory when both exist for module paths)
        let ab_dir = current_dir.join(parts[0]).join(parts[1]);
        if ab_dir.is_dir() {
            let mut current = self.build_dir_module_map(&ab_dir)?;
            for (i, part) in parts[2..].iter().enumerate() {
                match current {
                    Value::Map(ref map) => {
                        let key = super::value::MapKey::String((*part).to_string());
                        current = map.get(&key).cloned().ok_or_else(|| {
                            RuntimeError::InvalidOperation {
                                message: format!(
                                    "Module '{}' not found in '{}'",
                                    part,
                                    parts[..parts.len() - 1].join(":")
                                ),
                            }
                        })?;
                    }
                    _ => {
                        let prefix = parts[..i + 2].join(":");
                        return Err(RuntimeError::InvalidOperation {
                            message: format!("'{}' is not a module (not a map)", prefix),
                        });
                    }
                }
            }
            return Ok(current);
        }

        // C) Try D/a/b.si directly
        let ab_file = current_dir.join(parts[0]).join(format!("{}.si", parts[1]));
        if ab_file.exists() {
            let mut current = self.load_file_value(&ab_file)?;
            let mut b_failed_nonmap = false;
            for (i, part) in parts[2..].iter().enumerate() {
                match current {
                    Value::Map(ref map) => {
                        let key = super::value::MapKey::String((*part).to_string());
                        match map.get(&key).cloned() {
                            Some(v) => current = v,
                            None => {
                                b_failed_nonmap = true;
                                break;
                            }
                        }
                    }
                    _ => {
                        if i + 1 < parts[2..].len() {
                            b_failed_nonmap = true;
                            break;
                        }
                    }
                }
            }
            if !b_failed_nonmap {
                return Ok(current);
            }
        }

        Err(RuntimeError::InvalidOperation {
            message: format!(
                "Module '{}' not found in '{}'",
                parts[1],
                parts[..parts.len() - 1].join(":")
            ),
        })
    }

    /// Load and evaluate a module file, caching by canonical absolute path
    fn load_file_value(&self, path: &Path) -> Result<Value, RuntimeError> {
        let canonical = match fs::canonicalize(path) {
            Ok(p) => p,
            Err(e) => {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Failed to canonicalize '{}': {}", path.display(), e),
                });
            }
        };

        if let Some(v) = self.file_cache.borrow().get(&canonical) {
            return Ok(v.clone());
        }

        let source =
            fs::read_to_string(&canonical).map_err(|e| RuntimeError::InvalidOperation {
                message: format!("Failed to read '{}': {}", canonical.display(), e),
            })?;

        let statements = match suji_parser::parse_program(&source) {
            Ok(stmts) => stmts,
            Err(e) => {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Parse error in module '{}': {}", canonical.display(), e),
                });
            }
        };

        // Evaluate with isolated environment and capture Export value explicitly
        let value = self.with_dir(canonical.parent().unwrap_or(Path::new(".")), || {
            let env = Rc::new(Env::new());
            let mut loop_stack = Vec::new();
            let mut export_value: Option<Value> = None;

            for stmt in &statements {
                let result = eval_stmt_with_modules(stmt, env.clone(), &mut loop_stack, self);
                match result {
                    Ok(Some(v)) => {
                        if matches!(stmt, Stmt::Export { .. }) {
                            export_value = Some(v);
                        }
                    }
                    Ok(None) => {
                        // continue
                    }
                    Err(e) => return Err(e),
                }
            }

            export_value.ok_or_else(|| RuntimeError::InvalidOperation {
                message: format!("Module file '{}' has no export", canonical.display()),
            })
        })?;

        self.file_cache
            .borrow_mut()
            .insert(canonical.clone(), value.clone());
        Ok(value)
    }

    /// Build a module map from a directory (recursively), caching by canonical absolute path
    fn build_dir_module_map(&self, dir: &Path) -> Result<Value, RuntimeError> {
        let canonical = match fs::canonicalize(dir) {
            Ok(p) => p,
            Err(e) => {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Failed to canonicalize '{}': {}", dir.display(), e),
                });
            }
        };

        if let Some(v) = self.file_cache.borrow().get(&canonical) {
            return Ok(v.clone());
        }

        // Collect entries (files and subdirectories), skipping hidden dot entries
        let mut file_entries: Vec<(String, PathBuf)> = Vec::new();
        let mut dir_entries: Vec<(String, PathBuf)> = Vec::new();

        for entry in fs::read_dir(&canonical).map_err(|e| RuntimeError::InvalidOperation {
            message: format!("Failed to read directory '{}': {}", canonical.display(), e),
        })? {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!(
                            "Failed to read directory entry '{}': {}",
                            canonical.display(),
                            e
                        ),
                    });
                }
            };
            let path = entry.path();
            let name_os = match path.file_name() {
                Some(n) => n,
                None => continue,
            };
            let name = name_os.to_string_lossy();
            if name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                dir_entries.push((name.to_string(), path.clone()));
            } else if path.is_file()
                && path.extension().map(|e| e == "si").unwrap_or(false)
                && path.file_stem().is_some()
            {
                let key = path.file_stem().unwrap().to_string_lossy().to_string();
                file_entries.push((key, path.clone()));
            }
        }

        // Sort keys deterministically
        file_entries.sort_by(|a, b| a.0.cmp(&b.0));
        dir_entries.sort_by(|a, b| a.0.cmp(&b.0));

        let mut map: IndexMap<super::value::MapKey, Value> = IndexMap::new();

        // Evaluate files in the directory context
        for (key, file_path) in file_entries.into_iter() {
            let val = self.with_dir(&canonical, || self.load_file_value(&file_path))?;
            map.insert(super::value::MapKey::String(key), val);
        }

        // Recurse into subdirectories
        for (key, subdir_path) in dir_entries.into_iter() {
            let val = self.with_dir(&canonical, || self.build_dir_module_map(&subdir_path))?;
            map.insert(super::value::MapKey::String(key), val);
        }

        let module_value = Value::Map(map);
        self.file_cache
            .borrow_mut()
            .insert(canonical.clone(), module_value.clone());
        Ok(module_value)
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_registry_creation() {
        let registry = ModuleRegistry::new();
        // std is now provided via virtual std resolver (set up by facade layer)
        assert!(!registry.has_module("nonexistent"));
    }

    #[test]
    #[ignore] // std is now provided by facade layer with virtual std resolver
    fn test_std_module_resolution() {
        let registry = ModuleRegistry::new();

        // Test resolving the std module
        let std_module = registry.resolve_module("std").unwrap();
        assert!(matches!(std_module, Value::Map(_)));
    }

    #[test]
    fn test_module_not_found() {
        let registry = ModuleRegistry::new();

        let result = registry.resolve_module("nonexistent");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Module 'nonexistent' not found")
        );
    }

    #[test]
    #[ignore] // std is now provided by facade layer with virtual std resolver
    fn test_item_not_found() {
        let registry = ModuleRegistry::new();

        let result = registry.resolve_module_item("std", "nonexistent");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Item 'nonexistent' not found")
        );
    }
}
