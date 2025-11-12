use crate::builtins::list_builtins;
use crate::executor::Executor;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use suji_values::{
    Env, EnvProxy, FunctionBody, FunctionValue, MapKey, ModuleHandle, RuntimeError,
    StreamProxyKind, Value,
};

/// Type for source evaluator callback (parse-agnostic)
/// Takes an executor, source code, environment, and registry, returns the export value
pub type SourceEvaluator =
    fn(&dyn Executor, &str, Rc<Env>, &ModuleRegistry) -> Result<Value, RuntimeError>;

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

/// Stable identity for module caching and cycle detection
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum CacheKey {
    /// Filesystem module (canonicalized absolute path)
    Filesystem(PathBuf),
    /// Virtual module (path segments, e.g., ["std", "json"])
    Virtual(Vec<String>),
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheKey::Filesystem(path) => write!(f, "{}", path.display()),
            CacheKey::Virtual(segments) => write!(f, "std:{}", segments.join(":")),
        }
    }
}

/// Result of attempting to load a module
#[derive(Debug)]
pub enum ModuleLoadAction {
    /// Module is already cached
    Cached(Value),
    /// Module source that needs parsing and execution
    NeedsLoad { source: String, cache_key: CacheKey },
    /// Directory with eager listing but lazy child loading
    LazyDirectory {
        children: IndexMap<String, CacheKey>,
        cache_key: CacheKey,
    },
}

/// Load state for cycle detection
#[derive(Debug, Clone, PartialEq)]
enum LoadState {
    Loading,            // Sentinel for cycle detection
    Loaded(Box<Value>), // Actual cached value
}

/// RAII guard for module loading lifecycle
pub struct LoadGuard<'a> {
    key: CacheKey,
    registry: &'a ModuleRegistry,
    committed: bool,
}

impl<'a> LoadGuard<'a> {
    /// Commit the load with the final value
    pub fn commit(mut self, value: Value) {
        self.registry
            .load_states
            .borrow_mut()
            .insert(self.key.clone(), LoadState::Loaded(Box::new(value)));
        self.committed = true;
    }
}

impl<'a> Drop for LoadGuard<'a> {
    fn drop(&mut self) {
        if !self.committed {
            // Abort the load on error/panic - remove from cache
            self.registry.load_states.borrow_mut().remove(&self.key);
        }
    }
}

/// Registry for managing modules in the SUJI language runtime
#[derive(Clone)]
pub struct ModuleRegistry {
    /// Built-in modules that are always available
    builtins: HashMap<String, Value>,
    /// Cache of loaded module values keyed by canonical absolute paths (files and directories)
    file_cache: RefCell<HashMap<PathBuf, Value>>,
    /// Load states for cycle detection (new CacheKey-based system)
    load_states: RefCell<HashMap<CacheKey, LoadState>>,
    /// Stack of importer directories for relative resolution
    dir_stack: RefCell<Vec<PathBuf>>,
    /// Optional virtual std resolver (set by suji-stdlib)
    virtual_std_resolver: Option<VirtualStdResolver>,
    /// Source evaluator callback (parse-agnostic, set by runtime)
    source_evaluator: Option<SourceEvaluator>,
}

// Manual Debug implementation since function pointers don't impl Debug
impl std::fmt::Debug for ModuleRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleRegistry")
            .field("builtins", &self.builtins.keys())
            .field("file_cache", &"<cached>")
            .field("load_states", &"<load_states>")
            .field("dir_stack", &self.dir_stack)
            .field("virtual_std_resolver", &self.virtual_std_resolver.is_some())
            .field("source_evaluator", &self.source_evaluator.is_some())
            .finish()
    }
}

impl ModuleRegistry {
    /// Create a builtin function wrapper
    fn create_builtin_function_wrapper(name: &'static str) -> Value {
        Value::Function(FunctionValue {
            params: vec![], // Arity checking deferred to builtin registry
            body: FunctionBody::Builtin(name),
            env: Rc::new(Env::new()),
        })
    }
    /// Create a new module registry with built-in modules
    pub fn new() -> Self {
        let mut registry = Self {
            builtins: HashMap::new(),
            file_cache: RefCell::new(HashMap::new()),
            load_states: RefCell::new(HashMap::new()),
            dir_stack: RefCell::new(Vec::new()),
            virtual_std_resolver: None,
            source_evaluator: None,
        };

        // Register built-in modules
        registry.register_builtin_modules();
        registry
    }

    /// Set the virtual std resolver
    pub fn set_virtual_std_resolver(&mut self, resolver: VirtualStdResolver) {
        self.virtual_std_resolver = Some(resolver);
    }

    /// Set the source evaluator callback (parse-agnostic)
    pub fn set_source_evaluator(&mut self, evaluator: SourceEvaluator) {
        self.source_evaluator = Some(evaluator);
    }

    /// Create a clone of this registry with a custom override for the `std` module
    pub fn with_custom_std(&self, std_value: Value) -> Self {
        let mut new_registry = self.clone();
        new_registry.builtins.insert("std".to_string(), std_value);
        new_registry
    }

    /// Begin loading a module (returns guard for cycle detection)
    pub fn begin_load(&self, key: &CacheKey) -> Result<LoadGuard<'_>, RuntimeError> {
        {
            let mut cache = self.load_states.borrow_mut();

            // Check for cycles
            if let Some(LoadState::Loading) = cache.get(key) {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Circular module dependency detected: {}", key),
                });
            }

            // Mark as loading
            cache.insert(key.clone(), LoadState::Loading);
        } // Drop the borrow before creating the LoadGuard

        Ok(LoadGuard {
            key: key.clone(),
            registry: self,
            committed: false,
        })
    }

    /// Execute a closure with directory context set (for relative imports)
    pub fn with_directory_context<F, R>(&self, cache_key: &CacheKey, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Extract directory from cache key
        let dir = match cache_key {
            CacheKey::Filesystem(path) => path.parent().map(|p| p.to_path_buf()),
            CacheKey::Virtual(_) => None, // Virtual modules don't have filesystem context
        };

        if let Some(dir) = dir {
            self.dir_stack.borrow_mut().push(dir);
            let result = f();
            self.dir_stack.borrow_mut().pop();
            result
        } else {
            f()
        }
    }

    /// Register all built-in modules
    fn register_builtin_modules(&mut self) {
        // Register the special __builtins__ virtual module
        let mut builtins_map: IndexMap<MapKey, Value> = IndexMap::new();

        // Add builtin function wrappers
        for name in list_builtins() {
            // Leak the string to get a 'static lifetime for the builtin name
            // This is acceptable since builtin names are registered once at startup
            let static_name: &'static str = Box::leak(name.clone().into_boxed_str());
            builtins_map.insert(
                MapKey::String(name),
                Self::create_builtin_function_wrapper(static_name),
            );
        }

        // Add IO stream proxies (resolve dynamically via IoContext)
        builtins_map.insert(
            MapKey::String("io_stdin".to_string()),
            Value::StreamProxy(StreamProxyKind::Stdin),
        );
        builtins_map.insert(
            MapKey::String("io_stdout".to_string()),
            Value::StreamProxy(StreamProxyKind::Stdout),
        );
        builtins_map.insert(
            MapKey::String("io_stderr".to_string()),
            Value::StreamProxy(StreamProxyKind::Stderr),
        );

        // Add environment variable map (via EnvProxy)
        let env_proxy = Value::EnvMap(Rc::new(EnvProxy::new()));
        builtins_map.insert(MapKey::String("env_var".to_string()), env_proxy);

        // Snapshot command-line arguments into maps at startup
        // Exclude interpreter name (argv[0]) and any interpreter options (leading '-' args)
        let mut args_map: IndexMap<MapKey, Value> = IndexMap::new();
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
                    args_map.insert(MapKey::String("0".to_string()), Value::String(a));
                    script_seen = true;
                    continue;
                }
            }
            // Subsequent args are positional starting from "1"
            let idx = args_map.len().saturating_sub(1); // exclude "0"
            args_map.insert(MapKey::String(idx.to_string()), Value::String(a));
        }

        builtins_map.insert(
            MapKey::String("env_args".to_string()),
            Value::Map(args_map.clone()),
        );
        builtins_map.insert(MapKey::String("env_argv".to_string()), Value::Map(args_map));

        self.builtins
            .insert("__builtins__".to_string(), Value::Map(builtins_map));
    }

    /// Resolve a builtin module by name
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
        executor: &dyn Executor,
        module_name: &str,
        item_name: &str,
    ) -> Result<Value, RuntimeError> {
        let module = self.resolve_module(module_name)?;

        match module {
            Value::Map(map) => {
                let key = MapKey::String(item_name.to_string());
                match map.get(&key) {
                    Some(value) => {
                        // Force-load if it's a module
                        match value {
                            Value::Module(handle) => self.force_load_module(executor, handle),
                            _ => Ok(value.clone()),
                        }
                    }
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
        executor: &dyn Executor,
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
                    let key = MapKey::String(part.to_string());
                    match map.get(&key) {
                        Some(Value::Map(nested_map)) => {
                            current_module = Value::Map(nested_map.clone());
                        }
                        Some(Value::Module(handle)) => {
                            // Force-load lazy module
                            current_module = self.force_load_module(executor, handle)?;
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
                let key = MapKey::String(item_name.to_string());
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

    /// Create a lazy module handle (load deferred)
    pub fn create_lazy_module(
        &self,
        module_path: String,
        segments: Vec<String>,
        source: Option<&'static str>,
    ) -> ModuleHandle {
        ModuleHandle::new(
            module_path,
            segments,
            source,
            self as *const ModuleRegistry as *const (),
        )
    }

    /// Force a lazy module to load
    pub fn force_load_module(
        &self,
        executor: &dyn Executor,
        handle: &ModuleHandle,
    ) -> Result<Value, RuntimeError> {
        // Check if already loaded
        if let Some(loaded) = handle.loaded.borrow().as_ref() {
            return Ok((**loaded).clone());
        }

        // Load the module based on type
        let value = if let Some(source) = handle.source {
            // Virtual module (e.g., std lib)
            let cache_key = PathBuf::from(format!("<virtual>/{}.si", handle.segments.join("/")));
            self.load_virtual_module_internal(executor, source, &cache_key)?
        } else {
            // Filesystem module
            let file_path = PathBuf::from(handle.segments.join("/") + ".si");
            self.load_file_value(executor, &file_path)?
        };

        // Cache the loaded value
        *handle.loaded.borrow_mut() = Some(Box::new(value.clone()));

        Ok(value)
    }

    /// Internal helper for loading virtual modules with cycle detection
    fn load_virtual_module_internal(
        &self,
        executor: &dyn Executor,
        source: &str,
        cache_key: &Path,
    ) -> Result<Value, RuntimeError> {
        // Check cache first (for cycle detection and performance)
        if let Some(cached) = self.file_cache.borrow().get(cache_key) {
            return Ok(cached.clone());
        }

        // Insert sentinel for cycle detection
        self.file_cache.borrow_mut().insert(
            cache_key.to_path_buf(),
            Value::String("__LOADING__".to_string()),
        );

        // Get source evaluator
        let source_eval = self.source_evaluator.ok_or_else(|| RuntimeError::InvalidOperation {
            message: "Module evaluation callback not set. Call set_source_evaluator() on the registry.".to_string(),
        })?;

        // Evaluate source
        let env = Rc::new(Env::new());
        let result = match source_eval(executor, source, env, self) {
            Ok(value) => value,
            Err(e) => {
                self.file_cache.borrow_mut().remove(cache_key);
                return Err(e);
            }
        };

        // Update cache with real value
        self.file_cache
            .borrow_mut()
            .insert(cache_key.to_path_buf(), result.clone());

        Ok(result)
    }

    /// Set the base importer directory (clears and initializes the dir stack)
    pub fn set_base_dir(&self, base: impl AsRef<Path>) {
        let mut stack = self.dir_stack.borrow_mut();
        stack.clear();
        stack.push(base.as_ref().to_path_buf());
    }

    /// Push a directory, run f, then pop (panic-safe)
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

    /// Resolve a module path (e.g., "a:b:c") via env → filesystem → builtins
    pub fn resolve_module_path(
        &self,
        executor: &dyn Executor,
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
                        let key = MapKey::String((*part).to_string());
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
        if let Ok(v) = self.resolve_via_fs_path(executor, &parts, prefer_dir_last) {
            return Ok(v);
        }

        // 3) Builtins fallback for the root, then traverse
        let mut current = self.resolve_module(parts[0])?;
        for part in &parts[1..] {
            match current {
                Value::Map(ref map) => {
                    let key = MapKey::String((*part).to_string());
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
    pub fn resolve_module_root(
        &self,
        executor: &dyn Executor,
        env: &Rc<Env>,
        root: &str,
    ) -> Result<Value, RuntimeError> {
        if let Ok(v) = env.get(root) {
            return Ok(v);
        }
        if let Some(v) = self.resolve_fs_root(executor, root)? {
            return Ok(v);
        }
        self.resolve_module(root)
    }

    /// Try to resolve a root from filesystem (checks virtual std for "std")
    fn resolve_fs_root(
        &self,
        executor: &dyn Executor,
        name: &str,
    ) -> Result<Option<Value>, RuntimeError> {
        // Check virtual std sources for "std" module
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
            let value = self.load_file_value(executor, &file_path)?;
            return Ok(Some(value));
        }

        let dir_path = current_dir.join(name);
        if dir_path.is_dir() {
            let value = self.build_dir_module_map(executor, &dir_path)?;
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
                let mut map: IndexMap<MapKey, Value> = IndexMap::new();

                for child in children {
                    // Try to load each child as a file
                    let mut child_segments = segments.to_vec();
                    child_segments.push(&child);

                    match resolver(&child_segments) {
                        Some(VirtualStdResult::File(source)) => {
                            // Create a lazy module instead of loading immediately
                            let module_path = format!("std:{}", child_segments.join(":"));
                            let segments_owned: Vec<String> =
                                child_segments.iter().map(|s| s.to_string()).collect();

                            let handle =
                                self.create_lazy_module(module_path, segments_owned, Some(source));

                            map.insert(MapKey::String(child.clone()), Value::Module(handle));
                        }
                        Some(VirtualStdResult::Directory(_)) => {
                            // Nested directory - recursively build its module map
                            if let Some(value) =
                                self.build_virtual_std_module_map(&child_segments)?
                            {
                                map.insert(MapKey::String(child.clone()), value);
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

    /// Resolve a colon-separated path via filesystem
    fn resolve_via_fs_path(
        &self,
        executor: &dyn Executor,
        parts: &[&str],
        prefer_dir_last: bool,
    ) -> Result<Value, RuntimeError> {
        if parts.is_empty() {
            return Err(RuntimeError::InvalidOperation {
                message: "Empty module path".to_string(),
            });
        }
        if parts.len() == 1 {
            if let Some(v) = self.resolve_fs_root(executor, parts[0])? {
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
            let root_val = self.load_file_value(executor, &a_file)?;
            let mut current = root_val;
            let mut a_failed_nonmap = false;
            for (i, part) in parts[1..].iter().enumerate() {
                match current {
                    Value::Map(ref map) => {
                        let key = MapKey::String((*part).to_string());
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
            let mut current = self.build_dir_module_map(executor, &ab_dir)?;
            for (i, part) in parts[2..].iter().enumerate() {
                match current {
                    Value::Map(ref map) => {
                        let key = MapKey::String((*part).to_string());
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
            let mut current = self.load_file_value(executor, &ab_file)?;
            let mut b_failed_nonmap = false;
            for (i, part) in parts[2..].iter().enumerate() {
                match current {
                    Value::Map(ref map) => {
                        let key = MapKey::String((*part).to_string());
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
    fn load_file_value(&self, executor: &dyn Executor, path: &Path) -> Result<Value, RuntimeError> {
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

        // Get source evaluator
        let source_eval = self.source_evaluator.ok_or_else(|| RuntimeError::InvalidOperation {
            message: "Module evaluation callback not set. Call set_source_evaluator() on the registry.".to_string(),
        })?;

        // Evaluate source
        let value = self.with_dir(canonical.parent().unwrap_or(Path::new(".")), || {
            let env = Rc::new(Env::new());
            source_eval(executor, &source, env, self)
        })?;

        self.file_cache
            .borrow_mut()
            .insert(canonical.clone(), value.clone());
        Ok(value)
    }

    /// Build a module map from a directory (recursively), caching by canonical absolute path
    fn build_dir_module_map(
        &self,
        executor: &dyn Executor,
        dir: &Path,
    ) -> Result<Value, RuntimeError> {
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

        let mut map: IndexMap<MapKey, Value> = IndexMap::new();

        // Evaluate files in the directory context
        for (key, file_path) in file_entries.into_iter() {
            let val = self.with_dir(&canonical, || self.load_file_value(executor, &file_path))?;
            map.insert(MapKey::String(key), val);
        }

        // Recurse into subdirectories
        for (key, subdir_path) in dir_entries.into_iter() {
            let val = self.with_dir(&canonical, || {
                self.build_dir_module_map(executor, &subdir_path)
            })?;
            map.insert(MapKey::String(key), val);
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
    fn test_cache_key_display() {
        let fs_key = CacheKey::Filesystem(PathBuf::from("/path/to/module.si"));
        assert_eq!(format!("{}", fs_key), "/path/to/module.si");

        let virt_key = CacheKey::Virtual(vec!["json".to_string()]);
        assert_eq!(format!("{}", virt_key), "std:json");

        let nested_key = CacheKey::Virtual(vec!["json".to_string(), "parse".to_string()]);
        assert_eq!(format!("{}", nested_key), "std:json:parse");
    }

    #[test]
    fn test_cache_key_equality() {
        let key1 = CacheKey::Filesystem(PathBuf::from("/test/module.si"));
        let key2 = CacheKey::Filesystem(PathBuf::from("/test/module.si"));
        let key3 = CacheKey::Filesystem(PathBuf::from("/other/module.si"));

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);

        let virt1 = CacheKey::Virtual(vec!["std".to_string(), "json".to_string()]);
        let virt2 = CacheKey::Virtual(vec!["std".to_string(), "json".to_string()]);
        let virt3 = CacheKey::Virtual(vec!["std".to_string(), "yaml".to_string()]);

        assert_eq!(virt1, virt2);
        assert_ne!(virt1, virt3);
        assert_ne!(key1, virt1); // Filesystem != Virtual
    }

    #[test]
    fn test_begin_load_cycle_detection() {
        let registry = ModuleRegistry::new();
        let key = CacheKey::Virtual(vec!["test".to_string()]);

        let _guard = registry.begin_load(&key).unwrap();
        // Try to load same module again - should error
        assert!(registry.begin_load(&key).is_err());
    }

    #[test]
    fn test_load_guard_commit() {
        let registry = ModuleRegistry::new();
        let key = CacheKey::Virtual(vec!["test".to_string()]);

        let guard = registry.begin_load(&key).unwrap();
        guard.commit(Value::String("test module".to_string()));

        // After commit, should be in Loaded state
        let states = registry.load_states.borrow();
        assert!(matches!(states.get(&key), Some(LoadState::Loaded(_))));
    }

    #[test]
    fn test_load_guard_drop_cleanup() {
        let registry = ModuleRegistry::new();
        let key = CacheKey::Virtual(vec!["test".to_string()]);

        {
            let _guard = registry.begin_load(&key).unwrap();
            // Guard dropped here without commit
        }

        // Should be removed from loading state
        {
            let states = registry.load_states.borrow();
            assert!(states.get(&key).is_none());
        } // Drop the borrow before retry

        // Should be able to retry
        assert!(registry.begin_load(&key).is_ok());
    }

    #[test]
    fn test_load_guard_panic_safety() {
        let registry = ModuleRegistry::new();
        let key = CacheKey::Virtual(vec!["test".to_string()]);

        // Simulate panic during module load
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = registry.begin_load(&key).unwrap();
            panic!("Simulated panic during module load");
        }));

        // Panic should have occurred
        assert!(panic_result.is_err());

        // Guard should have been dropped and key removed from loading state
        {
            let states = registry.load_states.borrow();
            assert!(states.get(&key).is_none());
        } // Drop the borrow before retry

        // Should be able to retry after panic
        assert!(
            registry.begin_load(&key).is_ok(),
            "Should be able to retry after panic"
        );
    }

    #[test]
    fn test_with_directory_context() {
        let registry = ModuleRegistry::new();
        let key = CacheKey::Filesystem(PathBuf::from("/test/dir/module.si"));

        registry.with_directory_context(&key, || {
            // Directory context should be set
            let stack = registry.dir_stack.borrow();
            assert_eq!(stack.len(), 1);
            assert_eq!(stack[0], PathBuf::from("/test/dir"));
        });

        // Directory context should be cleaned up
        let stack = registry.dir_stack.borrow();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_with_directory_context_nested() {
        let registry = ModuleRegistry::new();
        let key1 = CacheKey::Filesystem(PathBuf::from("/test/dir1/module1.si"));
        let key2 = CacheKey::Filesystem(PathBuf::from("/test/dir2/module2.si"));

        registry.with_directory_context(&key1, || {
            registry.with_directory_context(&key2, || {
                // Both contexts should be stacked
                let stack = registry.dir_stack.borrow();
                assert_eq!(stack.len(), 2);
                assert_eq!(stack[0], PathBuf::from("/test/dir1"));
                assert_eq!(stack[1], PathBuf::from("/test/dir2"));
            });

            // Inner context cleaned up, outer still there
            let stack = registry.dir_stack.borrow();
            assert_eq!(stack.len(), 1);
            assert_eq!(stack[0], PathBuf::from("/test/dir1"));
        });

        // All contexts cleaned up
        let stack = registry.dir_stack.borrow();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_with_directory_context_virtual() {
        let registry = ModuleRegistry::new();
        let key = CacheKey::Virtual(vec!["std".to_string(), "json".to_string()]);

        registry.with_directory_context(&key, || {
            // Virtual modules don't set directory context
            let stack = registry.dir_stack.borrow();
            assert_eq!(stack.len(), 0);
        });
    }
}
