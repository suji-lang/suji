# NN Language

A dynamically and strongly typed language with familiar syntax, higher-order functions, closures, built-in string interpolation, regular expressions, and seamless shell interaction.

## Table of Contents

- [Quick Start](#quick-start)
- [Language Overview](#language-overview)
- [Data Types](#data-types)
  - [Numbers](#numbers)
  - [Booleans](#booleans)
  - [Strings](#strings)
  - [Lists](#lists)
  - [Maps](#maps)
  - [Tuples](#tuples)
  - [Regular Expressions](#regular-expressions)
  - [Streams](#streams)
  - [Nil](#nil)
- [Operators](#operators)
  - [Assignment](#assignment)
  - [Arithmetic](#arithmetic)
  - [Relational](#relational)
  - [Logical](#logical)
  - [Matching](#matching)
  - [Pipe](#pipe)
  - [Pipe Apply](#pipe-apply)
  - [Function Composition](#function-composition)
- [Control Flow](#control-flow)
  - [Loops](#loops)
  - [Match Expressions](#match-expressions)
- [Functions](#functions)
- [Modules](#modules)
- [Shell Integration](#shell-integration)
- [Advanced Features](#advanced-features)
  - [String Interpolation](#string-interpolation)
  - [Deep Nesting](#deep-nesting)
  - [Optional Braces](#optional-braces)
- [Standard Library](#standard-library)
  - [env](#env)
    - [Environment variables](#environment-variables)
    - [Command-line arguments](#command-line-arguments)
  - [io (Process Streams)](#io-process-streams)
  - [JSON Module](#json-module)
  - [YAML Module](#yaml-module)
  - [TOML Module](#toml-module)
  - [Random Module](#random-module)
  - [print and println](#print-and-println)
- [Examples](#examples)
  - [Fibonacci Sequence](#fibonacci-sequence)
  - [Quicksort](#quicksort)
  - [File Processing](#file-processing)
  - [Configuration Management](#configuration-management)
- [Installation](#installation)
- [CLI & REPL Usage](#cli--repl-usage)
- [Spec & Testing](#spec--testing)
- [Language Versions](#language-versions)

## Quick Start

```nn
# Hello World
import std:println
println("Hello, World!")

# Variables and functions
name = "Alice"
greet = |n| "Hello, ${n}!"
message = greet(name)
println(message)

# Lists and loops
numbers = [1, 2, 3, 4, 5]
loop through numbers with n {
    println(n)
}

# Pattern matching
result = match 42 {
    42 => "The answer",
    _ => "Something else",
}
println(result)
```

## Language Overview

NN is a dynamically and strongly typed language designed for simplicity and expressiveness. Key features include:

- **Dynamic typing**: Variables can hold values of any type
- **Strong typing**: Type safety enforced at runtime
- **Higher-order functions**: Functions are first-class values
- **Closures**: Functions can capture variables from their lexical scope
- **Pattern matching**: Powerful control flow with `match` expressions
- **String interpolation**: Built-in `${expression}` syntax
- **Regular expressions**: Native regex support with `/pattern/` literals
- **Shell integration**: Execute commands with backticks
- **Pipelines**: Stream data between closures and shell commands with `|`; value pipelines with `|>`/`<|`

## Data Types

### Numbers

NN has one number type: 64-bit decimal numbers with precise base‑10 semantics (no IEEE-754 surprises).

```nn
x = 42
y = 3.14159
z = -10.5

# Arithmetic operations
sum = x + y
product = x * y
power = x ^ 2  # 42^2 = 1764
modulo = x % 5  # 42 % 5 = 2

# Increment/decrement
x++  # x is now 43
y--  # y is now 2.14159
```

```nn
import std:println

# Intuitive equality/ordering
println(0.1 + 0.2 == 0.3)  # true
println(1.50 > 1.5)        # false (equal values)

# Deterministic rounding helpers
println((10 / 3)::floor())  # 3
println((10 / 3)::ceil())   # 4
println((10 / 3)::round())  # 3

# Errors instead of NaN/Inf
# 1 / 0  # runtime error: division by zero
```

### Booleans

```nn
true_value = true
false_value = false

# Logical operations
result = true && false  # false
result = true || false  # true
result = !true          # false
```

### Strings

Strings are Unicode sequences with built-in interpolation:

```nn
# Basic strings
name = "Alice"
path = '/home/user'  # Single quotes also supported

# String interpolation
age = 30
message = "Hello, ${name}! You are ${age} years old."
calculation = "The result is ${10 + 5}"

# Escape sequences
escaped = "He said, \"Hello there!\""
newline = "Line 1\nLine 2"
tabbed = "Column 1\tColumn 2"

# Multiline strings
poem = """
Roses are red,
Violets are blue,
Sugar is sweet,
And so are you.
"""

# String methods
text = "hello world"
length = text::length()           # 11
words = text::split()             # ["hello", "world"]
upper = text::upper()             # "HELLO WORLD"
reversed = text::reverse()        # "dlrow olleh"
contains_world = text::contains("world")  # true
```

### Lists

Ordered, zero-based, growable sequences:

```nn
# List literals
empty = []
numbers = [1, 2, 3, 4, 5]
mixed = ["hello", 42, true, [1, 2]]

# Range literals
range = 0..5        # [0, 1, 2, 3, 4]
descending = 5..0   # [5, 4, 3, 2, 1]

# Indexing and slicing
first = numbers[0]      # 1
last = numbers[-1]      # 5
slice = numbers[1:3]    # [2, 3]
first_two = numbers[:2] # [1, 2]
from_third = numbers[2:] # [3, 4, 5]

# List methods
numbers::push(6)        # [1, 2, 3, 4, 5, 6]
last_item = numbers::pop()  # 6, list is now [1, 2, 3, 4, 5]
count = numbers::length()   # 5
joined = numbers::join(",") # "1,2,3,4,5"

# Functional methods
evens = numbers::filter(|x| x % 2 == 0)  # [2, 4]
squares = numbers::map(|x| x * x)        # [1, 4, 9, 16, 25]
total = numbers::sum()                   # 15
sorted = [3, 1, 4]::sort()              # [1, 3, 4]

# Additional helpers (v0.1.6)
avg = numbers::average()                # 3
first_or_default = empty::first("n/a") # "n/a"
last_or_zero = empty::last(0)           # 0
```

### Maps

Key-value dictionaries:

```nn
# Map literals
empty_map = {}
config = { name: "Alice", age: 30, active: true }
nested = { 
    user: { name: "Bob", settings: { theme: "dark" } }
}

# Accessing values
name = config:name        # "Alice" (dot notation)
age = config["age"]       # 30 (bracket notation)
theme = nested:user:settings:theme  # "dark" (deep access)

# Assigning values
config:email = "alice@example.com"
config["city"] = "New York"

# Map methods
has_name = config::contains("name")     # true
keys = config::keys()                   # ["name", "age", "active", "email", "city"]
values = config::values()               # ["Alice", 30, true, "alice@example.com", "New York"]
size = config::length()                 # 5
email = config::get("email", "N/A")     # "alice@example.com"
```

### Tuples

Immutable, fixed-length collections:

```nn
# Tuple literals
point = (10, 20)
data = ("hello", 42, true)
single = (42,)  # Single-element tuple

# Tuple methods
length = point::length()        # 2
as_list = point::to_list()      # [10, 20]
as_string = point::to_string()  # "(10, 20)"
```

### Regular Expressions

```nn
# Regex literals
email_pattern = /^[^@\s]+@[^@\s]+\.[^@\s]+$/
number_pattern = /\d+/

# Matching
email = "user@example.com"
is_valid = email ~ email_pattern  # true
has_numbers = "abc123" ~ number_pattern  # true
```

### Streams

Blocking I/O type for file descriptors and process streams. Methods may block; `read()` returns nil on EOF.

```nn
import std:io

# Read a single chunk (may block)
chunk = io:stdin::read()

# Read everything or lines until EOF
all   = io:stdin::read_all()
lines = io:stdin::read_lines()

# Read a single line (newline not included)
line = io:stdin::read_line()

# Check if stdin is a terminal (TTY)
interactive = io:stdin::is_terminal()

# Write to stdout/stderr
io:stdout::write("Hello, world!\n")
io:stderr::write("Warning: something happened\n")
```

### Nil

Represents the absence of a value:

```nn
nothing = nil
result = match nothing {
    nil => "No value",
    _ => "Has value",
}
```

## Operators

### Assignment

```nn
x = 42
y = "hello"

# Compound assignment
x += 5    # x = x + 5
x -= 3    # x = x - 3
x *= 2    # x = x * 2
x /= 4    # x = x / 4
x %= 7    # x = x % 7
```

### Arithmetic

```nn
a = 10
b = 3

sum = a + b      # 13
diff = a - b     # 7
product = a * b  # 30
quotient = a / b # 3.333...
modulo = a % b   # 1
power = a ^ b    # 1000
negative = -a    # -10
```

### Relational

```nn
x = 5
y = 10

x == y   # false
x != y   # true
x < y    # true
x <= y   # true
x > y    # false
x >= y   # false
```

### Logical

```nn
a = true
b = false

a && b   # false
a || b   # true
!a       # false
```

### Matching

```nn
text = "hello@example.com"
pattern = /^[^@\s]+@[^@\s]+\.[^@\s]+$/

matches = text ~ pattern    # true
no_match = text !~ pattern  # false
```

### Pipe

Connect the stdout of a source closure to the stdin of a destination closure.

```nn
import std:println
import std:io

destination = || {
    loop through io:stdin::read_lines() with line {
        match {
            line ~ /test/ => return "output received",
        }
    }
}

source = || {
    println("test")
}

out = source() | destination()
println(out)  # output received

# Shell → closure
sink = || {
    loop through io:stdin::read_lines() with line {
        match {
            line ~ /beta/ => return "beta received",
        }
    }
}
result1 = `printf "alpha\nbeta\n"` | sink
println(result1)  # beta received

# Closure → shell
producer = || {
    println("alpha")
    println("beta")
}
filtered = producer() | `grep beta`
println(filtered)  # "beta\n"

# Closure → shell → closure
collector = || {
    lines = io:stdin::read_lines()
    return lines::join(",")
}
result2 = producer() | `grep beta` | collector
println(result2)  # beta
```

### Pipe Apply

Value-to-function application pipelines (v0.1.8):

```nn
# Forward apply (left-to-right): x |> f  ==  f(x)
inc = |x| x + 1
double = |x| x * 2

result = 3 |> inc |> double
println(result)  # 8

# Backward apply (right-to-left): f <| x  ==  f(x)
result2 = double <| inc <| 3
println(result2)  # 8

# Mixing with stream pipe: stream pipe `|` operates on closures/backticks,
# while `|>`/`<|` operate on values and functions. Precedence groups so that
# `a |> g | h` parses as `(a |> g) | h`.
```

Notes:
- `|>` is left-associative; `<|` is right-associative.
- `|>` requires a function on the right; `<|` requires a function on the left.

```nn
import std:println
import std:json

# Left side can be any expression (including backticks)
data = `echo '{"name":"Ada","age":30}'` |> json:parse
println(data:name)  # Ada

# Right side can be any expression with <|
import std:json:parse
data2 = parse <| '{"x": 1, "y": 2}'
println(data2:x + data2:y)  # 3

# Backticks also work with <|
data3 = parse <| `printf '{"ok":true}'`
println(data3:ok)  # true
```

Backtick commands can participate in pipelines as sources, middles, or sinks (v0.1.8):

```nn
import std:println
import std:io

# Backtick as source → closure sink
sink = || {
    line = io:stdin::read_line()
    return line
}

out = `echo test` | sink
println(out)  # test

# Closure → backtick as middle → closure
producer = || { println("alpha\nbeta\n") }
consumer = || {
    lines = io:stdin::read_lines()
    return lines::join(",")
}

out = producer | `grep beta` | consumer
println(out)  # beta

# Note: When a backtick command is last in a pipeline, its stdout is returned
# as a string without trimming trailing newlines.
```

Associativity and precedence: `|` remains left-associative and sits between assignment and logical-or. Backticks outside of a pipeline behave as before (trimmed output when not piped).

### Function Composition

Compose unary functions into a new function:

```nn
import std:println

add2 = |n| n + 2
mul3 = |n| n * 3

add2ThenMul3 = add2 >> mul3   # x -> mul3(add2(x))
mul3ThenAdd2 = add2 << mul3   # x -> add2(mul3(x))

println(add2ThenMul3(1))  # 9
println(mul3ThenAdd2(1))  # 5
```

## Control Flow

### Loops

```nn
# Infinite loop
count = 0
loop {
    count++
    match count {
        5 => { break },
        _ => { continue },
    }
}

# Loop through lists
numbers = [1, 2, 3, 4, 5]
loop through numbers with n {
    println(n)
}

# Loop through maps
config = { name: "Alice", age: 30 }
loop through config with key, value {
    println("${key}: ${value}")
}

# Nested loops with labels
loop as outer {
    loop as inner {
        match some_condition {
            true => { break outer },  # Exit both loops
            false => { continue inner },
        }
    }
}
```

### Match Expressions

```nn
# Basic matching
x = 42
result = match x {
    42 => "The answer",
    0 => "Zero",
    _ => "Something else",
}

# Pattern matching with tuples
point = (10, 20)
description = match point {
    (0, 0) => "Origin",
    (x, 0) => "On x-axis",
    (0, y) => "On y-axis",
    (_, _) => "Somewhere else",
}

# Regex patterns
email = "user@example.com"
type = match email {
    /^admin@/ => "Admin email",
    /@company\.com$/ => "Company email",
    _ => "Other email",
}

# Conditional matching
x = 5
y = 3
status = match {
    x > 10 => "Very large",
    x > 5 => "Large",
    x > 0 => "Positive",
    _ => "Zero or negative",
}
```

#### Pattern alternation

Handle multiple patterns in one arm using `|`:

```nn
import std:println

n = 3
text = match n {
    1 => "One",
    2 | 3 | 4 => "Couple",
    _ => "Many",
}

println(text)  # Couple
```

## Functions

Functions are first-class values with closure support:

```nn
# Basic function
add = |x, y| {
    return x + y
}

# Implicit return (last expression)
multiply = |x, y| x * y

# Default parameters
greet = |name = "World"| "Hello, ${name}!"

# Closures
make_counter = |start| {
    count = start
    return || {
        count++
        return count
    }
}

counter = make_counter(10)
println(counter())  # 11
println(counter())  # 12

# Higher-order functions
numbers = [1, 2, 3, 4, 5]
doubled = numbers::map(|x| x * 2)  # [2, 4, 6, 8, 10]
evens = numbers::filter(|x| x % 2 == 0)  # [2, 4]
sum = numbers::fold(0, |acc, x| acc + x)  # 15
```

### Multiple return values and destructuring

```nn
make_pair = || { return 1, 4 }

left, right = make_pair()
println(right)  # 4

# Discard with '_'
first, _, third = || { return 10, 20, 30 }()
println(third)  # 30
```

## Modules

Modules provide namespacing and code reuse. The standard library is available under the `std` module (e.g., `import std:println`, `import std:env:var`, `import std:io`):

```nn
# math.nn
export {
    PI: 3.14159,
    add: |a, b| a + b,
    multiply: |a, b| a * b
}

# main.nn
import math:add
import math:PI
import math:multiply as mul

result = add(5, 3)  # 8
area = mul(PI, 2)   # 6.28318
```

## Shell Integration

Execute shell commands with backticks:

```nn
# Basic command execution
output = `echo "Hello, World!"`
println(output)  # "Hello, World!"

# Command with interpolation
name = "Alice"
greeting = `echo "Hello, ${name}!"`
println(greeting)  # "Hello, Alice!"

# Complex commands
files = `ls -la | grep ".txt"`
count = `wc -l < /etc/passwd`
```

## Advanced Features

### String Interpolation

```nn
name = "Alice"
age = 30
message = "Hello, ${name}! You are ${age} years old."
calculation = "The result is ${10 + 5 * 2}"

# In shell commands
output = `echo "User: ${name}, Age: ${age}"`
```

### Deep Nesting

```nn
# Deep map access
config = {
    user: {
        profile: {
            settings: {
                display: {
                    theme: "dark"
                }
            }
        }
    }
}

theme = config:user:profile:settings:display:theme
config:user:profile:settings:display:theme = "light"

# Deep list access
matrix = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]
value = matrix[0][1][0]  # 3
matrix[0][1][0] = 99
```

### Optional Braces

```nn
# Single expressions don't need braces
square = |x| x * x
result = match x {
    1 => "one",
    2 => "two",
    _ => "other",
}

# Multiple expressions need braces
process = |x| {
    doubled = x * 2
    doubled + 1
}
```

## Standard Library

### JSON Module

```nn
import std:json

# Parse JSON
json_str = '{"name": "Alice", "age": 30}'
data = json:parse(json_str)

# Generate JSON
user = { name: "Bob", age: 25 }
json_output = json:generate(user)
```

### YAML Module

```nn
import std:yaml

# Parse YAML
yaml_str = "name: Alice\nage: 30"
data = yaml:parse(yaml_str)

# Generate YAML
config = { name: "Bob", settings: { theme: "dark" } }
yaml_output = yaml:generate(config)
```

### TOML Module

```nn
import std:toml

# Parse TOML
toml_str = 'name = "Alice"\nage = 30'
data = toml:parse(toml_str)

# Generate TOML
config = { name: "Bob", active: true }
toml_output = toml:generate(config)
```

### Random Module

```nn
import std:random
import std:println

# Seed RNG for determinism (optional)
random:seed(42)

# Uniform in [0,1)
x = random:random()

# Integer in [a,b)
i = random:integer(10, 20)

# List helpers
items = ["a", "b", "c", "d"]
pick_one = random:pick(items)
shuffled = random:shuffle(items)
sampled = random:sample(items, 2)

println(i)
```

### env

#### Environment variables

Environment variables exposed as a map under the standard library at `std:env:var`. Changing `var` affects the process environment (and child processes).

```nn
import std:env
import std:println

path = env:var:PATH
home = env:var["HOME"]
println("PATH: ${path}")
println("HOME: ${home}")

# Defaults and existence
editor = env:var::get("EDITOR", "vi")
println("Editor: ${editor}")
```

#### Command-line arguments

Command-line arguments are exposed as maps under `std:env:args`.

```nn
# Access first argument (after program name)
import std:env:args

first = args::get("1", nil)
if (first != nil) {
  println("First arg: ${first}")
}
```

```nn
# Iterate all arguments by index key
import std:env:argv
import std:println

loop through argv::keys() with k {
  v = argv[k]
  println("arg[${k}] = ${v}")
}
```

### io (Process Streams)

Access standard streams as `stream` values. Operations may block.

```nn
import std:io
import std:println

input = io:stdin::read()
println("Read: ${input}")
io:stdout::write("ok\n")
io:stderr::write("err\n")

# Read a single line
line = io:stdin::read_line()

# Detect if output is a terminal
is_tty = io:stdout::is_terminal()
```

Open files are streams as well:

```nn
import std:io

# Write
out = io:open("output.txt")
out::write("Hello, world!\n")
out::close()

# Read
f = io:open("output.txt")
content = f::read_all()
f::close()
```

### print and println

Convenience output functions that write to streams. Default target is `std:io:stdout`.

```nn
import std:print
import std:println
import std:io

print("Hello, world!\n")
println("line without manual newline")
println("to stderr", io:stderr)
```

## Examples

### Fibonacci Sequence

```nn
fib = |n| {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}

# Generate first 10 Fibonacci numbers
numbers = 0..10
fibs = numbers::map(fib)
println(fibs)  # [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
```

### Quicksort

```nn
quicksort = |list| {
    match list::length() {
        0 => [],
        1 => list,
        _ => {
            pivot = list[0]
            rest = list[1:]
            left = rest::filter(|x| x < pivot)
            right = rest::filter(|x| x >= pivot)
            return quicksort(left) + [pivot] + quicksort(right)
        }
    }
}

unsorted = [64, 34, 25, 12, 22, 11, 90]
sorted = quicksort(unsorted)
println(sorted)  # [11, 12, 22, 25, 34, 64, 90]
```

### File Processing

```nn
# Read and process a file
content = `cat data.txt`
lines = content::split("\n")
processed = lines::map(|line| line::trim()::upper())
result = processed::join("\n")
`echo "${result}" > output.txt`
```

### Configuration Management

```nn
# Load configuration from YAML
import std:yaml

config_yaml = `cat config.yaml`
config = yaml:parse(config_yaml)

# Process configuration
match config::contains("database") {
    true => {
        db_config = config:database
        connection_string = "postgresql://${db_config:host}:${db_config:port}/${db_config:name}"
        println("Connecting to: ${connection_string}")
    },
    false => println("No database configuration found"),
}
```

## Installation

```bash
# Clone the repository
git clone https://github.com/ragnar-johannsson/nnlang.git
cd nnlang

# Build the project
cargo build --release

# Run examples
cargo run -- examples/hello.nn

# Start REPL
cargo run
```

## CLI & REPL Usage

Run a file:

```bash
cargo run -- examples/hello.nn
# or after building in release:
target/release/nnlang examples/hello.nn
```

Start the REPL:

```bash
cargo run
# Exit: Ctrl-D
```

## Spec & Testing

Specification tests live under `spec/`. Each spec file contains a single test and ends with one `println` outputting the result. Use the helper scripts to verify behavior:

```bash
scripts/verify_spec.sh
scripts/verify_examples.sh
```

Rust unit/integration tests:

```bash
cargo test
```

## Language Versions

- **v0.1.0**: Core language features
- **v0.1.1**: Variable scope changes, compound operators, new methods
- **v0.1.2**: String indexing, descending ranges, list concatenation
- **v0.1.3**: Conditional matching, new map methods, JSON module
- **v0.1.4**: YAML and TOML modules, deep nesting fixes
- **v0.1.5**: Comprehensive method library, multiline strings
- **v0.1.6**: ENV map, stream type, FD streams, print/println rewrite, list average, first/last defaults
- **v0.1.7**: Rename FD->std:io and ENV->std:env:var, add stream::read_line and ::is_terminal, add std:env:args/argv, introduce pipe operator
- **v0.1.8**: Backticks in `|` pipelines (source/middle/sink), pipe-apply operators `|>` and `<|`, add `std:random` module
- **v0.1.9**: Decimal number semantics; `std:io:open(path)`; multiple return values and destructuring; `|` requires invocations
- **v0.1.10**: Runtime error spans/positions; string-literal-in-interpolation fix; module method calls in match conditions fix
- **v0.1.11**: Match expressions syntax change
- **v0.1.12**: Function composition operators (`>>`, `<<`); pattern alternation in match arms; `|>`/`<|` accept expressions on the applicable side
