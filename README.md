# SUJI

This is **Suji**: a small, expressive language with dynamic but strong typing, higher‑order functions, pattern matching, built‑in string interpolation, regex, and seamless shell integration. It has a strong focus on pipes and pipelines with support for both shell-like `|` pipes and F#-like pipes, `|>` and `<|`.

```suji
import std:io
import std:println

producer = || {
    println("foo")
    println("bar")
    println("baz")
}

count_lines = || {
    lines = io:stdin::read_lines()
    return lines::length()
}

format = |n| "matches: ${n}"

producer() | `grep ba` | count_lines()
    |> format
    |> println
```


For the full language tour see the User Guide: [`docs/USER_GUIDE.md`](docs/USER_GUIDE.md). There are also more [`examples/`](examples/).

## Repository layout

- `crates/`
  - `suji-ast/`: AST types and helpers
  - `suji-cli/`: CLI entry point (binary)
  - `suji-diagnostics/`: Error reporter
  - `suji-interpreter/`: AST-walking interpreter
  - `suji-lexer/`: Lexical scanner
  - `suji-parser/`: Parser and precedence
  - `suji-repl/`: REPL entry point (binary)
  - `suji-runtime/`: Executor trait, module system, and builtins
  - `suji-stdlib/`: Standard library implementations
  - `suji-values/`: Value types, errors, environment, and methods
- `examples/`: Sample `suji` scripts
- `spec/`: Language specification tests
- `tests/`: Rust integration tests
- `scripts/`: Build verification helpers

Entry points:
- Library: exposed via the crate graph under `crates/`
- CLI: `crates/suji-cli/src/main.rs`

## Build

Prerequisite: Rust stable (via `rustup`).

```bash
make build    # debug
make release  # optimized
```

## Run

Run a file:

```bash
cargo run -- examples/hello.si
```

Or, after a `make release`:

```bash
target/release/suji examples/hello.si
```

Start the REPL:

```bash
cargo run
```

## Test

Tests are split into three categories:

```bash
make rust_tests       # Rust unit and integration tests
make verify_spec      # Language specification tests
make verify_examples  # Examples verification
```

Or, for short:

```bash
make test
```
