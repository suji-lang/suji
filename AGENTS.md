### AGENTS: Working with AI Coding Assistants in this Repository

This document defines how AI coding agents (and human collaborators using them) should operate in this project. It establishes expectations, workflows, coding standards, and guardrails to keep the codebase healthy and predictable.

### Scope and Principles
- **Primary goals**: correctness, clarity, maintainability, and performance where it matters.
- **Small, focused edits**: keep changes narrowly scoped; avoid broad refactors mixed with feature work.
- **Document intent**: when behavior changes, update docs and tests in the same change.
- **No silent behavior changes**: if public behavior changes, call it out.

### Repository Overview (quick map)
- **Language and runtime**: Rust
- **Entry points**: `src/main.rs` (CLI), `src/lib.rs` (library)
- **Lexer**: `src/lexer/` (`core.rs`, `states/`, `token.rs`, `utils.rs`)
- **Parser**: `src/parser/`
- **Runtime/eval**: `src/runtime/`
- **Diagnostics**: `src/diagnostics/`
- **Tests**: `tests/` (Rust integration/unit tests)
- **Language specs (nn)**: `spec/` (source-of-truth examples and expectations)
- **Examples (nn)**: `examples/`
- **Docs**: `docs/` (`NN_LANG.md`, implementation plans)

### Local Dev Basics
- **Build**:
```bash
cargo build
```
- **Run** (interpret a `.nn` program):
```bash
cargo run -- examples/hello.nn
```
- **Test** (Rust tests):
```bash
cargo test
```
- **Verify language specs** (nn spec files):
```bash
make verify_spec
```
- **Lint**:
```bash
make lint
```

### Test Conventions
- **Rust tests**: live in `tests/`. Add specific suites (e.g., `lexer_*`, `parser_*`, `eval_*`) matching the area you change.
- **Spec files (`spec/*.nn`)**: these are single-spec programs checked by the scripts above. Convention: one expectation per file with a single `println` of the result at the end (project convention) [[Spec tests: one println per file]].
- **When adding features**: add/adjust both a Rust test and a spec file (when applicable). Prefer minimal, readable inputs that isolate the behavior.

#### Spec file conventions (`spec/`)
- **One assertion per file**: keep each file laser‑focused on a single example/expectation; end the file with exactly one `println(...)`.
- **Expected output annotation**: place the expected output in a trailing comment on the final `println` line. The spec runner extracts the text after `#` on the last line and compares it to the program output.
  - Example: `println(3 |> inc) # 4`
  - This must be on the same, final line (the runner uses `tail -n 1`).
- **Import printing**: explicitly import `std:println` in each file that prints.
  - Example: `import std:println`
- **Naming**: name files by feature area with a numeric suffix: `feature_area_XX.nn`.
  - Use 2‑digit, zero‑padded counters for sequences (e.g., `pipe_apply_01.nn`, `pipe_apply_02.nn`).
  - Keep names short, descriptive, and consistent with existing files (e.g., `list_methods_07.nn`, `operator_precedence_03.nn`).
- **Content**: prefer minimal, readable inputs that isolate the behavior under test; avoid unrelated constructs.
- **Determinism**: specs must be deterministic. If a feature depends on environment or IO, stub/minimize it so the output is stable.

##### Style
- Add an empty line after all import statements.
- Add an empty line before the final `println(...)` statement.
- Add two spaces before the expected output comment in the final println statement. ("println(result) # expected_output")
- See `spec/pipe_apply_01.nn` for an example layout.

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
- **Lexer**: `src/lexer/core.rs` implements scanning; state machines live under `src/lexer/states/` (e.g., `string.rs`, `regex.rs`, `shell.rs`, `interpolation.rs`). `LexState` and `ScannerContext` carry control and position info.
- **Regex disambiguation**: consult `ScannerContext::should_parse_as_regex` and nearby logic before changing operator/regex parsing.
- **Parser**: precedence and constructs are under `src/parser/*`. Update precedence rules if new operators are introduced.

### Runtime/Eval Notes
- Runtime implementation under `src/runtime/` mirrors language features. Changing evaluation often requires updating both the parser output and runtime handlers (expressions, statements, methods, etc.).

### Performance Guidelines
- Keep hot paths allocation-lean and branch-predictable (lexer, parser inner loops, runtime tight loops).
- When optimizing, include a microbenchmark or measurement if the change is non-trivial. Use `benches/` and Criterion when appropriate.

### Documentation Expectations
- **Language behavior**: update `docs/NN_LANG.md` when syntax/semantics change.
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
  - Run: `make lint`, `make test`, `make verify_spec`, `make verify_examples`.
  - Update docs as needed.

### Guardrails
- Do not change public-facing behavior without tests demonstrating the change.
- Avoid cross-cutting refactors in the same PR as feature work.
- Avoid adding new dependencies without justification.
- Keep unsafe Rust out unless strictly necessary and reviewed.
- Do not run any git commands; git workflow is user-owned.

### Where to Add What
- **New token or operator**: `src/lexer/*`, adjust states, add `tests/lexer_*`, update `parser/precedence.rs` and parser modules, then runtime handling.
- **New standard function/module**: `src/runtime/builtins/`, add tests under `tests/` and examples/specs.
- **Diagnostics**: prefer `src/diagnostics/*` helpers; keep user messages consistent.

### CI/Verification (manual today)
- Ensure all local checks pass:
```bash
make lint
make test
make verify_spec
make verify_examples
```

### Contact and Ownership
- If a change spans lexer, parser, and runtime, break it into reviewable steps where possible.
- Unsure about language design? Propose the change in a doc under `docs/` and link it from your PR.

---
If anything here conflicts with existing code conventions or tests, prefer existing behavior and update this document in a follow-up.
