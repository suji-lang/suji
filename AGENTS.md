### AGENTS: Working with AI Coding Assistants in this Repository

This document defines how AI coding agents (and human collaborators using them) should operate in this project. It establishes expectations, workflows, coding standards, and guardrails to keep the codebase healthy and predictable.

### Scope and Principles
- **Primary goals**: correctness, clarity, maintainability, and performance where it matters.
- **Small, focused edits**: keep changes narrowly scoped; avoid broad refactors mixed with feature work.
- **Document intent**: when behavior changes, update docs and tests in the same change.
- **No silent behavior changes**: if public behavior changes, call it out.

### Crate map and key files

- **suji-ast**: AST data structures and spans
  - `crates/suji-ast/src/lib.rs` (crate exports)
  - `crates/suji-ast/src/ast/` (AST node definitions)
  - `crates/suji-ast/src/span.rs` (source span types and helpers)

- **suji-cli**: CLI entry and execution
  - `crates/suji-cli/src/main.rs` (binary entry point; parses args, runs REPL or file)

- **suji-diagnostics**: diagnostics and error reporting utilities
  - `crates/suji-diagnostics/src/lib.rs` (crate exports)
  - `crates/suji-diagnostics/src/diagnostics/` (diagnostic builders, messages, span formatting)

- **suji-lexer**: lexical scanner for Suji source
  - `crates/suji-lexer/src/lib.rs` (crate exports)
  - `crates/suji-lexer/src/lexer/core.rs` (scanner core; drives state machine)
  - `crates/suji-lexer/src/lexer/states/` (state handlers: `string.rs`, `regex.rs`, `shell.rs`, `interpolation.rs`)
  - `crates/suji-lexer/src/lexer/token.rs` (token kinds and token data)
  - `crates/suji-lexer/src/lexer/utils.rs` (helpers for scanning)

- **suji-parser**: parser and precedence rules
  - `crates/suji-parser/src/lib.rs` (crate exports)
  - `crates/suji-parser/src/parser/` (parsing modules)
  - `crates/suji-parser/src/parser/precedence.rs` (operator precedence and associativity)

- **suji-repl**: interactive REPL engine (used by CLI)
  - `crates/suji-repl/src/lib.rs` (REPL loop, line evaluation)

- **suji-runtime**: evaluator/runtime for executing AST
  - `crates/suji-runtime/src/lib.rs` (crate exports)
  - `crates/suji-runtime/src/runtime/eval/` (expression/statement evaluators)
    - Notable: `function_call.rs` (call invocation), `expressions/binary.rs` (binary ops)
  - `crates/suji-runtime/src/runtime/methods/` (methods on values; `common.rs` shared helpers)
  - Other runtime modules under `crates/suji-runtime/src/runtime/` (values, env, helpers)

- **suji-stdlib**: standard library builtins
  - `crates/suji-stdlib/src/lib.rs` (crate exports)
  - `crates/suji-stdlib/src/runtime/builtins/modules/` (module aggregators, e.g., `std.rs`, `json.rs`)
  - `crates/suji-stdlib/src/runtime/builtins/functions/` (builtin functions, e.g., `json_generate.rs`, `yaml_generate.rs`)
  - `crates/suji-stdlib/src/runtime/**/*.si` (stdlib Suji source files, when present)

- **Workspace facade**
  - `src/lib.rs` (workspace-level facade library)

- **Top-level supporting dirs**
  - `tests/` (Rust integration/unit tests)
  - `spec/` (single-assertion spec programs)
  - `examples/` (runnable examples)
  - `docs/`, `internal_docs/` (user and design docs)
  - `scripts/` (spec/examples verification scripts)
  - `Makefile` (common tasks)

### Where to find X (quick index)

- **Token kinds and tokens**: `crates/suji-lexer/src/lexer/token.rs`
- **Scanning rules by construct**:
  - Strings: `crates/suji-lexer/src/lexer/states/string.rs`
  - Regex: `crates/suji-lexer/src/lexer/states/regex.rs` (disambiguation via `ScannerContext::should_parse_as_regex` in `lexer/core.rs`)
  - Shell templates: `crates/suji-lexer/src/lexer/states/shell.rs`
  - Interpolation: `crates/suji-lexer/src/lexer/states/interpolation.rs`
- **Operator precedence/associativity**: `crates/suji-parser/src/parser/precedence.rs`
- **Expression/statement parsing**: under `crates/suji-parser/src/parser/`
- **Function/method invocation (runtime)**: `crates/suji-runtime/src/runtime/eval/function_call.rs`
- **Binary expression evaluation**: `crates/suji-runtime/src/runtime/eval/expressions/binary.rs`
- **Value methods (runtime)**: `crates/suji-runtime/src/runtime/methods/` (see `common.rs`)
- **Diagnostics helpers/messages**: `crates/suji-diagnostics/src/diagnostics/`
- **Built-in modules and functions**:
  - Modules: `crates/suji-stdlib/src/runtime/builtins/modules/` (e.g., `std.rs`, `json.rs`)
  - Functions: `crates/suji-stdlib/src/runtime/builtins/functions/`
- **REPL loop**: `crates/suji-repl/src/lib.rs`
- **CLI entry point**: `crates/suji-cli/src/main.rs`
- **Workspace facade library**: `src/lib.rs`
- **Spec tests and conventions**: `spec/*.si` (one expectation per file; import `std:println`; last line `println(...)  # expected`)
- **Verification scripts**: `scripts/verify_spec.sh`, `scripts/verify_examples.sh`
- **Examples**: `examples/*.si`

### Local Dev Basics
- **Build**:
```bash
cargo build
```
- **Run** (interpret a `.si` program):
```bash
cargo run -- examples/hello.si
```
- **Test (Rust tests only)**:
```bash
make rust_tests
# or
cargo test
```
- **Test (full suite: Rust + specs + examples)**:
```bash
make test
```
- **Lint**:
```bash
make lint
```

### Test Conventions
- **Rust tests**: live in `tests/`. Add specific suites (e.g., `lexer_*`, `parser_*`, `eval_*`) matching the area you change.
- **Spec files (`spec/*.si`)**: these are single-spec programs checked by the scripts above. Convention: one expectation per file with a single `println` of the result at the end (project convention) [[Spec tests: one println per file]].
- **When adding features**: add/adjust both a Rust test and a spec file (when applicable). Prefer minimal, readable inputs that isolate the behavior.

#### Spec file conventions (`spec/`)
- **One assertion per file**: keep each file laser‑focused on a single example/expectation; end the file with exactly one `println(...)`.
- **Expected output annotation**: place the expected output in a trailing comment on the final `println` line. The spec runner extracts the text after `#` on the last line and compares it to the program output.
  - Example: `println(3 |> inc) # 4`
  - This must be on the same, final line (the runner uses `tail -n 1`).
- **Import printing**: explicitly import `std:println` in each file that prints.
  - Example: `import std:println`
- **Naming**: name files by feature area with a numeric suffix: `feature_area_XX.si`.
  - Use 2‑digit, zero‑padded counters for sequences (e.g., `pipe_apply_01.si`, `pipe_apply_02.si`).
  - Keep names short, descriptive, and consistent with existing files (e.g., `list_methods_07.si`, `operator_precedence_03.si`).
- **Content**: prefer minimal, readable inputs that isolate the behavior under test; avoid unrelated constructs.
- **Determinism**: specs must be deterministic. If a feature depends on environment or IO, stub/minimize it so the output is stable.

##### Style
- Add an empty line after all import statements.
- Add an empty line before the final `println(...)` statement.
- Add two spaces before the expected output comment in the final println statement. ("println(result) # expected_output")
- Don't have any trailing empty lines after the final `println(...)`. The spec harness reads only the last line; an extra blank line makes it think the expected output is empty (you'll see errors like `Expected '', got 'value'` when running `make verify_spec`). Ensure the file ends immediately after the commented `println` line.
- See `spec/pipe_apply_01.si` for an example layout.

### Coding Standards (Rust)
- **Clarity over cleverness**: favor explicit control flow and readable names.
- **Naming**: functions are verbs; variables are descriptive nouns; avoid abbreviations.
- **Control flow**: use guard clauses and early returns; avoid deep nesting.
- **Error handling**:
  - Use structured errors (e.g., `thiserror`) for domain errors.
  - Provide actionable messages; prefer precise spans via `diagnostics` helpers.
- **Comments**: explain “why”, not “how”; keep them short and above code.
- **Formatting**: match existing style; do not reformat unrelated code.
- **No binary blobs or oversized literals**: never commit generated binaries or huge inlined data.

### Lexer and Parser Notes
- **Lexer**: `crates/suji-lexer/src/lexer/core.rs` implements scanning; state machines live under `crates/suji-lexer/src/lexer/states/` (e.g., `string.rs`, `regex.rs`, `shell.rs`, `interpolation.rs`). `LexState` and `ScannerContext` carry control and position info.
- **Regex disambiguation**: consult `ScannerContext::should_parse_as_regex` and nearby logic before changing operator/regex parsing.
- **Parser**: precedence and constructs are under `crates/suji-parser/src/parser/*`. Update precedence rules if new operators are introduced.

### Runtime/Eval Notes
- Runtime implementation under `crates/suji-runtime/src/runtime/` mirrors language features. Changing evaluation often requires updating both the parser output and runtime handlers (expressions, statements, methods, etc.).

### Performance Guidelines
- Keep hot paths allocation-lean and branch-predictable (lexer, parser inner loops, runtime tight loops).
- When optimizing, include a microbenchmark or measurement if the change is non-trivial. Use `benches/` and Criterion when appropriate.

### Documentation Expectations
- **Language behavior**: update `docs/SUJI_LANG.md` when syntax/semantics change.
- **Refactors**: when restructuring modules or interfaces, record rationale in `docs/REFACTORING.md`.
- **Plans**: if delivering in stages, add/extend `docs/IMPLEMENTATION_PLAN_*.md` as needed.

### Workflow for Agents
- **Before editing**:
  - Search for existing helpers and tests in the area you’ll touch.
  - Read related docs under `docs/`.
- **During edits**:
  - Keep changes minimal and cohesive.
  - Preserve indentation and surrounding formatting; avoid drive-by style changes.
  - Add or update tests in the same change.
  - Do not perform any git operations; the user handles git (init/branch/commit/push/rebase/PR).
- **After any meaningful amount of change**:
  - Run: `make lint`, `make test`.
  - Update docs as needed.

### Guardrails
- Do not change public-facing behavior without tests demonstrating the change.
- Avoid cross-cutting refactors in the same PR as feature work.
- Avoid adding new dependencies without justification.
- If a change spans lexer, parser, and runtime, break it into reviewable steps where possible.
- Keep unsafe Rust out unless strictly necessary and reviewed.
- Do not run any git commands; git workflow is user-owned.

### CI/Verification (manual today)
- Ensure all local checks pass:
```bash
make lint
make test
```

---
If anything here conflicts with existing code conventions or tests, prefer existing behavior and update this document in a follow-up.
