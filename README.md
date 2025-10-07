# SUJI

This is **suji**, a small, expressive language with dynamic but strong typing, higher‑order functions, pattern matching, built‑in string interpolation and regex, and seamless shell integration. It has a strong focus on pipes and pipelines with support for both shell-like `|` pipes as well as F#-like pipe-apply pipes, `|>` and `<|`.

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
  - `suji-lexer/`: Lexical scanner
  - `suji-parser/`: Parser and precedence
  - `suji-repl/`: REPL entry point (binary)
  - `suji-runtime/`: Evaluator/runtime
  - `suji-stdlib/`: Standard library implementations
- `examples/`: Sample `suji` scripts
- `spec/`: Language specification tests
- `tests/`: Rust integration tests
- `scripts/`: Build verification helpers

Entry points:
- Library: exposed via the crate graph under `crates/`
- CLI: `crates/suji-cli/src/main.rs`
- REPL: `crates/suji-repl/src/main.rs`

## Build

Prerequisite: Rust stable (via `rustup`).

```bash
cargo build            # debug
cargo build --release  # optimized
```

## Run

Interpret a program file:

```bash
cargo run -- examples/hello.si
# or after building in release:
target/release/suji examples/hello.si
```

Start the REPL:

```bash
cargo run
# Exit: Ctrl-D
```

## Test

Rust tests only:

```bash
cargo test
```

Full suite (Rust + specs + examples):

```bash
make test
# or individually
scripts/verify_spec.sh
scripts/verify_examples.sh
```

## Lint

```bash
make lint
```