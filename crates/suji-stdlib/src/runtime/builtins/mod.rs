//! Builtins: functions and modules used by the runtime.

mod common;
mod functions;
mod json;
mod math;
mod random;
mod std_sources_map;
mod time;
mod toml;
mod virtual_std;
mod virtual_std_adapter;
mod yaml;

use functions::*;
use suji_values::Env;

/// Setup the global environment with built-in functions
pub fn setup_global_env(_env: &Env) {
    // Register all builtin functions with the runtime registry
    register_all_builtins();
}

/// Setup the module registry with virtual std resolver
/// Should be called before any module loading that depends on std
pub fn setup_module_registry(registry: &mut suji_runtime::ModuleRegistry) {
    // Register all builtin functions
    register_all_builtins();

    // Set up virtual std resolver
    registry.set_virtual_std_resolver(virtual_std_adapter::virtual_std_resolver);

    // Set up source evaluator
    registry.set_source_evaluator(suji_interpreter::eval_module_source_callback);
}

/// Register all builtin function implementations with the runtime
pub fn register_all_builtins() {
    use suji_runtime::register_builtin;

    // Register JSON functions
    register_builtin("json_parse", builtin_json_parse as suji_runtime::BuiltinFn);
    register_builtin(
        "json_generate",
        builtin_json_generate as suji_runtime::BuiltinFn,
    );

    // Register YAML functions
    register_builtin("yaml_parse", builtin_yaml_parse as suji_runtime::BuiltinFn);
    register_builtin(
        "yaml_generate",
        builtin_yaml_generate as suji_runtime::BuiltinFn,
    );

    // Register TOML functions
    register_builtin("toml_parse", builtin_toml_parse as suji_runtime::BuiltinFn);
    register_builtin(
        "toml_generate",
        builtin_toml_generate as suji_runtime::BuiltinFn,
    );

    // Register IO functions
    register_builtin("io_open", builtin_io_open as suji_runtime::BuiltinFn);

    // Register random functions
    register_builtin(
        "random_random",
        builtin_random_random as suji_runtime::BuiltinFn,
    );
    register_builtin(
        "random_seed",
        builtin_random_seed as suji_runtime::BuiltinFn,
    );

    // Register time functions
    register_builtin("time_now", builtin_time_now as suji_runtime::BuiltinFn);
    register_builtin("time_sleep", builtin_time_sleep as suji_runtime::BuiltinFn);
    register_builtin(
        "time_parse_iso",
        builtin_time_parse_iso as suji_runtime::BuiltinFn,
    );
    register_builtin(
        "time_format_iso",
        builtin_time_format_iso as suji_runtime::BuiltinFn,
    );

    // Register uuid functions (v5 only; v4 is SUJI)
    register_builtin("uuid_v5", builtin_uuid_v5 as suji_runtime::BuiltinFn);

    // Register encoding functions
    register_builtin(
        "encoding_base64_encode",
        builtin_encoding_base64_encode as suji_runtime::BuiltinFn,
    );
    register_builtin(
        "encoding_base64_decode",
        builtin_encoding_base64_decode as suji_runtime::BuiltinFn,
    );
    register_builtin(
        "encoding_hex_encode",
        builtin_encoding_hex_encode as suji_runtime::BuiltinFn,
    );
    register_builtin(
        "encoding_hex_decode",
        builtin_encoding_hex_decode as suji_runtime::BuiltinFn,
    );
    register_builtin(
        "encoding_percent_encode",
        builtin_encoding_percent_encode as suji_runtime::BuiltinFn,
    );
    register_builtin(
        "encoding_percent_decode",
        builtin_encoding_percent_decode as suji_runtime::BuiltinFn,
    );

    // Register math functions
    register_builtin("math_sin", builtin_math_sin as suji_runtime::BuiltinFn);
    register_builtin("math_cos", builtin_math_cos as suji_runtime::BuiltinFn);
    register_builtin("math_tan", builtin_math_tan as suji_runtime::BuiltinFn);
    register_builtin("math_asin", builtin_math_asin as suji_runtime::BuiltinFn);
    register_builtin("math_acos", builtin_math_acos as suji_runtime::BuiltinFn);
    register_builtin("math_atan", builtin_math_atan as suji_runtime::BuiltinFn);
    register_builtin("math_atan2", builtin_math_atan2 as suji_runtime::BuiltinFn);
    register_builtin("math_log", builtin_math_log as suji_runtime::BuiltinFn);
    register_builtin("math_log10", builtin_math_log10 as suji_runtime::BuiltinFn);
    register_builtin("math_exp", builtin_math_exp as suji_runtime::BuiltinFn);

    // Register crypto functions
    register_builtin("crypto_md5", builtin_crypto_md5 as suji_runtime::BuiltinFn);
    register_builtin(
        "crypto_sha1",
        builtin_crypto_sha1 as suji_runtime::BuiltinFn,
    );
    register_builtin(
        "crypto_sha256",
        builtin_crypto_sha256 as suji_runtime::BuiltinFn,
    );
    register_builtin(
        "crypto_sha512",
        builtin_crypto_sha512 as suji_runtime::BuiltinFn,
    );
    register_builtin(
        "crypto_hmac_sha256",
        builtin_crypto_hmac_sha256 as suji_runtime::BuiltinFn,
    );

    // Register CSV functions
    register_builtin("csv_parse", builtin_csv_parse as suji_runtime::BuiltinFn);
    register_builtin(
        "csv_generate",
        builtin_csv_generate as suji_runtime::BuiltinFn,
    );

    // Register OS functions
    register_builtin("os_name", builtin_os_name as suji_runtime::BuiltinFn);
    register_builtin(
        "os_hostname",
        builtin_os_hostname as suji_runtime::BuiltinFn,
    );
    register_builtin(
        "os_uptime_ms",
        builtin_os_uptime_ms as suji_runtime::BuiltinFn,
    );
    register_builtin("os_tmp_dir", builtin_os_tmp_dir as suji_runtime::BuiltinFn);
    register_builtin(
        "os_home_dir",
        builtin_os_home_dir as suji_runtime::BuiltinFn,
    );
    register_builtin(
        "os_work_dir",
        builtin_os_work_dir as suji_runtime::BuiltinFn,
    );
    register_builtin("os_exit", builtin_os_exit as suji_runtime::BuiltinFn);
    register_builtin("os_pid", builtin_os_pid as suji_runtime::BuiltinFn);
    register_builtin("os_ppid", builtin_os_ppid as suji_runtime::BuiltinFn);
    register_builtin("os_uid", builtin_os_uid as suji_runtime::BuiltinFn);
    register_builtin("os_gid", builtin_os_gid as suji_runtime::BuiltinFn);
}
