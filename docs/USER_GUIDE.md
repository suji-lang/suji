# SUJI User Guide

This guide covers the SUJI language: core concepts, syntax, standard library, and examples.

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
  - [Multiple return values and destructuring](#multiple-return-values-and-destructuring)
- [Modules](#modules)
- [Shell Integration](#shell-integration)
- [Advanced Features](#advanced-features)
  - [String Interpolation](#string-interpolation)
  - [Deep Nesting](#deep-nesting)
  - [Optional Braces](#optional-braces)
- [Standard Library](#standard-library)
  - [JSON Parsing and Generation (`std:json`)](#json-parsing-and-generation-stdjson)
  - [YAML Parsing and Generation (`std:yaml`)](#yaml-parsing-and-generation-stdyaml)
  - [TOML Parsing and Generation (`std:toml`)](#toml-parsing-and-generation-stdtoml)
  - [Random Number Generation (`std:random`)](#random-number-generation-stdrandom)
  - [Time and Date Functions (`std:time`)](#time-and-date-functions-stdtime)
  - [UUID Generation and Validation (`std:uuid`)](#uuid-generation-and-validation-stduuid)
  - [Text Encoding and Decoding (`std:encoding`)](#text-encoding-and-decoding-stdencoding)
  - [Mathematical Functions (`std:math`)](#mathematical-functions-stdmath)
  - [Cryptographic Hashing (`std:crypto`)](#cryptographic-hashing-stdcrypto)
  - [Operating System (`std:os`)](#operating-system-stdos)
  - [Path Utilities (`std:path`)](#path-utilities-stdpath)
  - [Environment File Loading (`std:dotenv`)](#environment-file-loading-stddotenv)
  - [CSV Parsing and Generation (`std:csv`)](#csv-parsing-and-generation-stdcsv)
  - [Environment Variables (`std:env`)](#environment-variables-stdenv)
  - [I/O and Streams (`std:io`)](#io-and-streams-stdio)
  - [Print Functions (`std:print`, `std:println`)](#print-functions-stdprint-stdprintln)
- [Examples](#examples)
- [Installation](#installation)
- [CLI & REPL Usage](#cli--repl-usage)
- [Spec & Testing](#spec--testing)
- [Language Versions](#language-versions)

## Quick Start

```suji
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

SUJI is a dynamically and strongly typed language designed for simplicity and expressiveness. Key features include:

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

SUJI has one number type: 64-bit decimal numbers with precise base‑10 semantics (no IEEE-754 surprises).

```suji
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

```suji
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

**Available Methods:**
- `to_string()` → Converts number to string
- `is_int()` → Returns `true` if number is an integer
- `abs()` → Returns absolute value
- `ceil()` → Rounds up to nearest integer
- `floor()` → Rounds down to nearest integer
- `round()` → Rounds to nearest integer
- `sqrt()` → Returns square root
- `pow(exponent)` → Raises number to power
- `min(other)` → Returns minimum of two numbers
- `max(other)` → Returns maximum of two numbers

### Booleans

```suji
true_value = true
false_value = false

# Logical operations
result = true && false  # false
result = true || false  # true
result = !true          # false
```

**Available Methods:**
- `to_string()` → Converts boolean to string ("true" or "false")

### Strings

Strings are Unicode sequences with built-in interpolation:

```suji
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

# Trim with optional custom character set
padded = "***hello***world***"
trimmed = padded::trim("*")       # "hello***world" (only trims edges)
whitespace = "  hello  "::trim()  # "hello" (default whitespace trim)
```

**Available Methods:**
- `length()` → Returns string length
- `split(separator)` → Splits string into list (default separator: space)
- `to_number()` → Converts string to number
- `to_list()` → Converts string to list of characters
- `index_of(substring)` → Returns index of substring (-1 if not found)
- `contains(substring)` → Returns `true` if string contains substring
- `starts_with(prefix)` → Returns `true` if string starts with prefix
- `ends_with(suffix)` → Returns `true` if string ends with suffix
- `replace(old, new)` → Replaces all occurrences of old with new
- `trim(chars)` → Trims characters from edges (default: whitespace)
- `upper()` → Converts to uppercase
- `lower()` → Converts to lowercase
- `reverse()` → Reverses the string
- `repeat(count)` → Repeats string count times
- `to_string()` → Returns the string itself

### Lists

Ordered, zero-based, growable sequences:

```suji
# List literals
empty = []
numbers = [1, 2, 3, 4, 5]
mixed = ["hello", 42, true, [1, 2]]

# Range literals
range = 0..5        # [0, 1, 2, 3, 4]
descending = 5..0   # [5, 4, 3, 2, 1]

# Inclusive ranges (include the end value)
inclusive = 0..=5      # [0, 1, 2, 3, 4, 5]
inclusive_desc = 10..=5  # [10, 9, 8, 7, 6, 5]

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

**Available Methods:**
- `push(item)` → Appends item to end of list
- `pop()` → Removes and returns last item
- `length()` → Returns number of items
- `join(separator)` → Joins items into string (default separator: space)
- `index_of(elem)` → Returns index of element (-1 if not found)
- `filter(closure)` → Returns new list with elements matching closure
- `map(closure)` → Transforms each element using closure
- `fold(initial, closure)` → Reduces list to single value
- `sum()` → Returns sum of numbers in list
- `product()` → Returns product of numbers in list
- `contains(elem)` → Returns `true` if list contains element
- `reverse()` → Returns reversed list
- `sort()` → Returns sorted list
- `min()` → Returns minimum number (numbers only)
- `max()` → Returns maximum number (numbers only)
- `first(default)` → Returns first element or default
- `last(default)` → Returns last element or default
- `average()` → Returns average of numbers (nil if empty)
- `to_string()` → Converts list to string representation

### Maps

Key-value dictionaries:

```suji
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

**Available Methods:**
- `delete(key)` → Removes key and returns `true` if it existed
- `contains(key)` → Returns `true` if map contains key
- `keys()` → Returns list of all keys
- `values()` → Returns list of all values
- `to_list()` → Returns list of [key, value] tuples
- `length()` → Returns number of key-value pairs
- `get(key, default)` → Returns value for key or default (nil if omitted)
- `merge(other_map)` → Merges other map into this map (mutates)
- `to_string()` → Converts map to string representation

### Tuples

Immutable, fixed-length collections:

```suji
# Tuple literals
point = (10, 20)
data = ("hello", 42, true)
single = (42,)  # Single-element tuple

# Tuple methods
length = point::length()        # 2
as_list = point::to_list()      # [10, 20]
as_string = point::to_string()  # "(10, 20)"
```

**Available Methods:**
- `length()` → Returns number of elements
- `to_list()` → Converts tuple to list
- `to_string()` → Converts tuple to string representation

### Regular Expressions

```suji
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

```suji
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

**Available Methods:**
- `read(chunk_kb)` → Reads chunk from stream (default: 8KB, returns nil on EOF)
- `read_line()` → Reads single line (returns nil on EOF)
- `read_all()` → Reads all content until EOF
- `read_lines()` → Reads all lines as list
- `write(text)` → Writes text to stream
- `is_terminal()` → Returns `true` if stream is a terminal
- `close()` → Closes the stream
- `to_string()` → Returns string representation of stream

### Nil

Represents the absence of a value:

```suji
nothing = nil
result = match nothing {
    nil => "No value",
    _ => "Has value",
}
```

### Type Checking Methods

All values support type checking methods that return `true` if the value is of the specified type, and `false` otherwise:

```suji
import std:println

# Number type checking
x = 42
println(x::is_number())    # true
println(x::is_string())    # false
println(x::is_list())      # false
```

**Available Methods:**
- `is_number()` → Returns `true` if value is a number
- `is_bool()` → Returns `true` if value is a boolean
- `is_string()` → Returns `true` if value is a string
- `is_list()` → Returns `true` if value is a list
- `is_map()` → Returns `true` if value is a map
- `is_stream()` → Returns `true` if value is a stream
- `is_function()` → Returns `true` if value is a function
- `is_tuple()` → Returns `true` if value is a tuple
- `is_regex()` → Returns `true` if value is a regex

**Notes:**
- All type checking methods are available on all values, including `nil`
- Each method returns `true` only when called on its corresponding type
- `nil` returns `false` for all type checking methods
- Useful for runtime type validation and conditional processing


## Operators

### Assignment

```suji
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

```suji
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

```suji
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

```suji
a = true
b = false

a && b   # false
a || b   # true
!a       # false
```

### Matching

```suji
text = "hello@example.com"
pattern = /^[^@\s]+@[^@\s]+\.[^@\s]+$/

matches = text ~ pattern    # true
no_match = text !~ pattern  # false
```

### Pipe

Connect the stdout of a source closure to the stdin of a destination closure.
Each stage must be a function invocation (e.g., `stage()`), not a function value (`stage`).

```suji
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
result1 = `printf "alpha\nbeta\n"` | sink()
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
result2 = producer() | `grep beta` | collector()
println(result2)  # beta
```

### Pipe Apply

Value-to-function application pipelines (v0.1.8):

```suji
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
# `a | b |> c` parses as `(a | b) |> c`.
```

Notes:
- `|>` is left-associative; `<|` is right-associative.
- `|>` requires a function on the right; `<|` requires a function on the left.

```suji
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

```suji
import std:println
import std:io

# Backtick as source → closure sink
sink = || {
    line = io:stdin::read_line()
    return line
}

out = `echo test` | sink()
println(out)  # test

# Closure → backtick as middle → closure
producer = || { println("alpha\nbeta\n") }
consumer = || {
    lines = io:stdin::read_lines()
    return lines::join(",")
}

out = producer() | `grep beta` | consumer()
println(out)  # beta

# Note: When a backtick command is last in a pipeline, its stdout is returned
# as a string without trimming trailing newlines.
```

Associativity and precedence: `|` remains left-associative and sits between assignment and logical-or. Backticks outside of a pipeline behave as before (trimmed output when not piped).

### Function Composition

Compose unary functions into a new function:

```suji
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

```suji
import std:println

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

### Guard Clauses with Short-Circuit

Use logical operators to short-circuit into control flow for concise early exits:

```suji
# Early exit in loops
count = 0
loop {
    count++
    count == 5 && break
}

# Guard clause in functions
validate = |x| {
    x < 0 && return "negative"
    x > 100 && return "too large"
    "valid"
}

# Inverse with ||: execute right side only when left is false
done = false
done || return "wasn't done"
```

### Match Expressions

```suji
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

```suji
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

```suji
import std:println

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

```suji
import std:println

make_pair = || { return 1, 4 }

left, right = make_pair()
println(right)  # 4

# Discard with '_'
first, _, third = || { return 10, 20, 30 }()
println(third)  # 30
```

## Modules

Modules provide namespacing and code reuse. The standard library is available under the `std` module (e.g., `import std:println`, `import std:env:var`, `import std:io`):

```suji
# math.si
export {
    PI: 3.14159,
    add: |a, b| a + b,
    multiply: |a, b| a * b
}

# main.si
import math:add
import math:PI
import math:multiply as mul

result = add(5, 3)  # 8
area = mul(PI, 2)   # 6.28318
```

Note: Modules are loaded lazily on first access and cached. This behavior is transparent to users and improves startup performance.

## Shell Integration

Execute shell commands with backticks:

```suji
import std:println

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

```suji
name = "Alice"
age = 30
message = "Hello, ${name}! You are ${age} years old."
calculation = "The result is ${10 + 5 * 2}"

# In shell commands
output = `echo "User: ${name}, Age: ${age}"`
```

### Deep Nesting

```suji
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

```suji
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

### JSON Parsing and Generation (`std:json`)

```suji
import std:json

# Parse JSON
json_str = '{"name": "Alice", "age": 30}'
data = json:parse(json_str)

# Generate JSON
user = { name: "Bob", age: 25 }
json_output = json:generate(user)
```

**Available Functions:**
- `parse(text)` → Parses JSON string into SUJI values (maps, lists, strings, numbers, booleans, nil)
- `generate(value)` → Converts SUJI value to JSON string

**Notes:**
- Preserves number precision using decimal semantics
- `nil` maps to JSON `null`
- Maps become JSON objects; lists become JSON arrays
- Raises a `RuntimeError` on malformed JSON

### YAML Parsing and Generation (`std:yaml`)

```suji
import std:yaml

# Parse YAML
yaml_str = "name: Alice\nage: 30"
data = yaml:parse(yaml_str)

# Generate YAML
config = { name: "Bob", settings: { theme: "dark" } }
yaml_output = yaml:generate(config)
```

**Available Functions:**
- `parse(text)` → Parses YAML string into SUJI values
- `generate(value)` → Converts SUJI value to YAML string

**Notes:**
- Supports nested structures and lists
- `nil` maps to YAML `null`
- More lenient parsing than JSON (unquoted strings, comments)
- Raises a `RuntimeError` on malformed YAML

### TOML Parsing and Generation (`std:toml`)

```suji
import std:toml

# Parse TOML
toml_str = 'name = "Alice"\nage = 30'
data = toml:parse(toml_str)

# Generate TOML
config = { name: "Bob", active: true }
toml_output = toml:generate(config)
```

**Available Functions:**
- `parse(text)` → Parses TOML string into SUJI values
- `generate(value)` → Converts SUJI value to TOML string

**Notes:**
- TOML is designed for configuration files
- Supports tables (maps) and arrays (lists)
- Keys must be valid TOML identifiers
- Raises a `RuntimeError` on malformed TOML

### Random Number Generation (`std:random`)

```suji
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

**Available Functions:**
- `seed(value)` → Seeds the RNG with a number for deterministic results
- `random()` → Returns random number in [0, 1)
- `integer(min, max)` → Returns random integer in [min, max)
- `pick(list)` → Returns random element from list
- `shuffle(list)` → Returns new list with elements in random order
- `sample(list, n)` → Returns list of n random elements (without replacement)
- `string(allowed_chars, length)` → Generates random string from character set
- `hex_string(length = 16)` → Generates random hexadecimal string (lowercase)
- `alpha_string(length = 16, capitals = true)` → Generates random alphabetic string
- `numeric_string(length = 16)` → Generates random numeric string (digits only)
- `alphanumeric_string(length = 16, capitals = true)` → Generates random alphanumeric string

**String Generation Examples:**

```suji
import std:random
import std:println

# Generate random string from custom character set
chars = "abcdef0123456789"
token = random:string(chars, 10)
println(token)  # e.g., "3a7f2b9c1e"

# Generate hexadecimal string (default length: 16)
id = random:hex_string()
println(id)  # e.g., "3a7f2b9c1e4d8f6a"

short_id = random:hex_string(8)
println(short_id)  # e.g., "7c3e1a5f"

# Generate alphabetic string
code = random:alpha_string()  # 16 chars, mixed case
println(code)  # e.g., "aBcDeFgHiJkLmNoP"

lowercase_code = random:alpha_string(10, false)
println(lowercase_code)  # e.g., "abcdefghij"

# Generate numeric string
pin = random:numeric_string(4)
println(pin)  # e.g., "7392"

# Generate alphanumeric string
session_id = random:alphanumeric_string()
println(session_id)  # e.g., "aB3dE7gH9jK2mN5p"

api_key = random:alphanumeric_string(32, false)
println(api_key)  # e.g., "a3b7c2d9e4f1g8h5i0j6k3l7m2n9o4p1"
```

**Notes:**
- RNG is thread-local and isolated per execution context
- Without `seed()`, uses non-deterministic randomness
- `pick()` raises error on empty list
- `sample(list, n)` raises error if n > list length
- `shuffle()` and `sample()` return new lists (original unchanged)
- String generation functions are implemented in pure SUJI using `random()` and `pick()`
- Character selection is uniformly random; each position is independently chosen
- Default length for string functions is 16 characters

### Time and Date Functions (`std:time`)

Work with time, dates, and sleep:

```suji
import std:time
import std:println

# Get current time
current = time:now()
println("Current time: ${current:iso}")
println("Epoch ms: ${current:epoch_ms}")
println("Timezone: ${current:tz}")

# Sleep for 1 second
println("Sleeping...")
time:sleep(1000)
println("Done!")

# Parse ISO-8601 string
iso_string = "2024-03-15T14:30:00Z"
parsed = time:parse_iso(iso_string)
println("Parsed: ${parsed:epoch_ms}")

# Format epoch to ISO-8601
epoch = 1710512400000
formatted = time:format_iso(epoch, "Z")
println("Formatted: ${formatted}")
```

**Available Functions:**
- `now()` → Returns map with current time information:
  - `iso` - ISO-8601 formatted string
  - `epoch_ms` - Milliseconds since Unix epoch
  - `tz` - Timezone offset string (e.g., "+00:00")
- `sleep(milliseconds)` → Pauses execution for given duration, returns nil
- `parse_iso(iso_string)` → Parses ISO-8601 string, returns map like `now()`
- `format_iso(epoch_ms, timezone)` → Formats epoch milliseconds as ISO-8601 string

**Notes:**
- All timestamps are in UTC unless otherwise specified
- `sleep()` blocks the current execution thread
- ISO-8601 format: `YYYY-MM-DDTHH:MM:SS.sssZ`
- Timezone can be "Z" (UTC) or offset like "+05:30"
- Raises a `RuntimeError` on invalid ISO-8601 strings

### UUID Generation and Validation (`std:uuid`)

Generate and validate UUIDs:

```suji
import std:uuid
import std:println

# Generate random UUID (v4)
id = uuid:v4()
println("Random UUID: ${id}")

# Generate namespaced UUID (v5)
dns_namespace = "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
domain_uuid = uuid:v5(dns_namespace, "example.com")
println("Domain UUID: ${domain_uuid}")

# Validate UUID strings
println(uuid:is_valid(id))                    # true
println(uuid:is_valid("not-a-uuid"))          # false
```

**Available Functions:**
- `v4()` → Generates random UUID (version 4)
- `v5(namespace, name)` → Generates deterministic UUID (version 5) from namespace and name
- `is_valid(uuid_string)` → Returns true if string is valid UUID format

**Notes:**
- UUIDs are returned as lowercase strings with hyphens
- v4 uses cryptographically secure randomness
- v5 is deterministic: same namespace+name always produces same UUID
- Common namespaces: DNS (`6ba7b810-9dad-11d1-80b4-00c04fd430c8`), URL, OID, X.500
- `is_valid()` accepts UUIDs with or without hyphens

### Text Encoding and Decoding (`std:encoding`)

Encode and decode text in various formats:

```suji
import std:encoding
import std:println

# Base64 encoding
original = "Hello, World!"
encoded = encoding:base64_encode(original)
decoded = encoding:base64_decode(encoded)
println("Base64: ${encoded}")

# Hexadecimal encoding
hex = encoding:hex_encode("Hello")
println("Hex: ${hex}")  # "48656c6c6f"

# Percent encoding (URL encoding)
query = "hello world & stuff"
encoded_url = encoding:percent_encode(query)
println("URL: ${encoded_url}")  # "hello%20world%20%26%20stuff"
```

**Available Functions:**
- `base64_encode(text)` → Encodes string to Base64
- `base64_decode(encoded)` → Decodes Base64 string
- `hex_encode(text)` → Encodes string to hexadecimal
- `hex_decode(encoded)` → Decodes hexadecimal string (case-insensitive)
- `percent_encode(text)` → URL/percent-encodes string (RFC 3986)
- `percent_decode(encoded)` → Decodes percent-encoded string

**Notes:**
- All encoding functions work with UTF-8 strings
- Base64 uses standard alphabet (not URL-safe variant)
- Hex encoding produces lowercase output
- Percent encoding encodes all non-alphanumeric characters except `-_.~`
- Raises `RuntimeError::InvalidOperation` on malformed encoded input

### Mathematical Functions (`std:math`)

Mathematical constants and functions:

```suji
import std:math
import std:println

# Constants
println(math:PI)  # 3.14159265358979323846...
println(math:E)   # 2.71828182845904523536...

# Trigonometric functions (input in radians)
println(math:sin(0))           # 0
println(math:cos(0))           # 1
println(math:sin(math:PI / 2)) # 1

# Logarithmic and exponential
println(math:log(math:E))      # 1
println(math:log10(100))       # 2
println(math:exp(1))           # 2.718...

# Convert degrees to radians
degrees = 45
radians = degrees * math:PI / 180
println(math:sin(radians))     # 0.7071... (sin of 45 degrees)
```

**Available Constants:**
- `PI` → π (3.14159265358979323846...)
- `E` → Euler's number (2.71828182845904523536...)

**Available Functions:**
- `sin(x)` → Sine of x (x in radians)
- `cos(x)` → Cosine of x (x in radians)
- `tan(x)` → Tangent of x (x in radians)
- `asin(x)` → Arcsine of x (returns radians), domain: [-1, 1]
- `acos(x)` → Arccosine of x (returns radians), domain: [-1, 1]
- `atan(x)` → Arctangent of x (returns radians)
- `log(x)` → Natural logarithm (base e), domain: x > 0
- `log10(x)` → Base-10 logarithm, domain: x > 0
- `exp(x)` → e^x (exponential function)
- `sqrt(x)` → Square root, domain: x ≥ 0

**Notes:**
- All trigonometric functions use radians (not degrees)
- To convert degrees to radians: `radians = degrees * math:PI / 180`
- Domain violations raise a `RuntimeError`
- Results are decimal numbers (not IEEE-754 floats)

### Cryptographic Hashing (`std:crypto`)

Cryptographic hashing and HMAC:

```suji
import std:crypto
import std:println

# Hash functions
text = "Hello, World!"
println("MD5: ${crypto:md5(text)}")
println("SHA-1: ${crypto:sha1(text)}")
println("SHA-256: ${crypto:sha256(text)}")
println("SHA-512: ${crypto:sha512(text)}")

# HMAC for message authentication
secret_key = "my-secret-key"
message = "authenticated message"
signature = crypto:hmac_sha256(secret_key, message)
println("HMAC: ${signature}")

# Verify message integrity
received_msg = "authenticated message"
computed_sig = crypto:hmac_sha256(secret_key, received_msg)
match computed_sig == signature {
    true => println("Message is authentic"),
    false => println("Message has been tampered with"),
}
```

**Available Functions:**
- `md5(text)` → MD5 hash (32-character hex string)
- `sha1(text)` → SHA-1 hash (40-character hex string)
- `sha256(text)` → SHA-256 hash (64-character hex string)
- `sha512(text)` → SHA-512 hash (128-character hex string)
- `hmac_sha256(key, message)` → HMAC-SHA256 (64-character hex string)

**Notes:**
- All hash functions return lowercase hexadecimal strings
- MD5 and SHA-1 are cryptographically weak; use SHA-256 or SHA-512 for security
- HMAC provides message authentication with a secret key
- All functions accept UTF-8 strings as input
- Deterministic: same input always produces same output

### Operating System (`std:os`)

Access operating system information and process utilities:

```suji
import std:os
import std:println

# Get OS name
os_name = os:name()  # "linux", "darwin", or "windows"
println("Running on: ${os_name}")

# Get hostname
hostname = os:hostname()
println("Hostname: ${hostname}")

# Get process information
pid = os:pid()
ppid = os:ppid()
println("PID: ${pid}, Parent PID: ${ppid}")

# Get directories
tmp = os:tmp_dir()
home = os:home_dir()
work = os:work_dir()
println("Temp dir: ${tmp}")
println("Home dir: ${home}")
println("Working dir: ${work}")

# Get system uptime
uptime = os:uptime_ms()
uptime_seconds = uptime / 1000
println("System uptime: ${uptime_seconds} seconds")

# Get user/group IDs (Unix-like systems)
uid = os:uid()
gid = os:gid()
println("UID: ${uid}, GID: ${gid}")

# Get file/directory metadata
stat = os:stat("data.txt")
println("Size: ${stat:size} bytes")
println("Modified: ${stat:mtime}")
println("Is directory: ${stat:is_directory}")
println("Is symlink: ${stat:is_symlink}")

# Follow symlinks to get target metadata
target_stat = os:stat("/usr/bin/python3", true)
println("Target size: ${target_stat:size}")

# Filesystem operations
os:mkdir("data/output/reports")  # Creates directory (with parents by default)
os:rm("temporary.txt")           # Removes a file
os:rmdir("empty_folder")         # Removes an empty directory
```

**Available Functions:**
- `name()` → Returns OS name: `"linux"`, `"darwin"`, or `"windows"`
- `hostname()` → Returns system hostname as string
- `uptime_ms()` → Returns system uptime in milliseconds since boot
- `tmp_dir()` → Returns path to system temporary directory
- `home_dir()` → Returns path to user's home directory
- `work_dir()` → Returns current working directory path
- `exit(code)` → Terminates process with given exit code (never returns)
- `pid()` → Returns current process ID
- `ppid()` → Returns parent process ID
- `uid()` → Returns user ID (Unix/macOS: actual UID; Windows: returns 0)
- `gid()` → Returns group ID (Unix/macOS: actual GID; Windows: returns 0)
- `stat(path, follow_symlinks = false)` → Returns file/directory metadata map
- `rm(path)` → Removes a file (not directories)
- `mkdir(path, create_all = true)` → Creates a directory
- `rmdir(path)` → Removes an empty directory

**File Metadata (`stat`):**

The `stat()` function returns a map with the following fields:
- `size` → File size in bytes (number)
- `is_directory` → `true` if path is a directory (boolean)
- `is_symlink` → `true` if path is a symbolic link (boolean)
- `link` → Symlink target path if `is_symlink` is `true`, otherwise `nil`
- `mode` → File permissions as a number
- `inode` → Inode number (Unix) or 0 (Windows)
- `uid` → Owner user ID (number)
- `gid` → Owner group ID (number)
- `atime` → Last access time in milliseconds since Unix epoch (number)
- `mtime` → Last modification time in milliseconds since Unix epoch (number)
- `ctime` → Last status change time in milliseconds since Unix epoch (number)

```suji
import std:os
import std:println

# Get file metadata
info = os:stat("data.txt")
println("Size: ${info:size} bytes")
println("Modified: ${info:mtime}")
println("Is directory: ${info:is_directory}")

# Check if path is a directory
check_dir = |path| {
    stat = os:stat(path)
    stat:is_directory
}

# Get file size
get_size = |path| {
    stat = os:stat(path)
    stat:size
}
```

**Filesystem Operations:**

```suji
import std:os
import std:println

# Create directory (with intermediate directories by default)
os:mkdir("data/output/reports")
println("Directory created")

# Create directory without creating parents
os:mkdir("logs/daily", false)  # Requires parent to exist

# Remove a file
os:rm("temporary.txt")
println("File removed")

# Remove an empty directory
os:rmdir("empty_folder")
println("Directory removed")
```

**Notes:**
- `uid()` and `gid()` return `0` on Windows (platform placeholder)
- `exit(code)` terminates the entire process immediately
- All directory paths are returned with platform-appropriate separators
- `stat()` raises an error if path doesn't exist or cannot be accessed
- When `follow_symlinks = false` (default), returns metadata for symlink itself
- When `follow_symlinks = true`, follows symlink and returns target metadata
- `rm()` only removes files; use `rmdir()` for directories
- `mkdir()` with `create_all = true` (default) creates all intermediate directories
- `rmdir()` only removes empty directories; raises error if directory contains files

### Path Utilities (`std:path`)

Cross-platform path manipulation utilities:

```suji
import std:path
import std:println

# Join path components
path = path:join(["home", "user", "documents", "file.txt"])
println(path)  # "home/user/documents/file.txt" (Unix) or "home\user\documents\file.txt" (Windows)

# Get directory and filename
full_path = "/home/user/documents/report.pdf"
dir = path:dirname(full_path)   # "/home/user/documents"
file = path:basename(full_path)  # "report.pdf"
println("Directory: ${dir}")
println("Filename: ${file}")

# Get file extension
ext = path:extname("report.pdf")  # ".pdf"
println("Extension: ${ext}")

# Check if path is absolute
is_abs_unix = path:is_abs("/home/user")     # true (Unix)
is_abs_win = path:is_abs("C:\\Users\\user")  # true (Windows)
is_abs_rel = path:is_abs("documents/file")  # false
println("Is absolute: ${is_abs_unix}")

# Normalize paths (resolve . and ..)
normalized = path:normalize("a/b/../c/./d")  # "a/c/d"
println("Normalized: ${normalized}")

# Handle edge cases
hidden_ext = path:extname(".bashrc")  # "" (hidden files have no extension)
no_ext = path:extname("README")       # ""
multi_ext = path:extname("archive.tar.gz")  # ".gz"
```

**Available Functions:**
- `join(parts_list)` → Joins path components with platform separator
- `dirname(path)` → Returns directory portion of path
- `basename(path)` → Returns filename portion of path
- `extname(path)` → Returns file extension (including dot), or empty string
- `normalize(path)` → Resolves `.` and `..`, removes redundant separators
- `is_abs(path)` → Returns true if path is absolute

**Notes:**
- Automatically detects platform using `std:os:name()`
- Unix/macOS use `/` separator; Windows uses `\` separator
- `extname()` returns empty string for hidden files (e.g., `.bashrc`)
- `normalize()` is purely lexical; does not resolve symlinks or check filesystem
- Empty path `""` is treated as current directory where applicable

### Environment File Loading (`std:dotenv`)

Load environment variables from `.env` files into the process environment:

```suji
import std:dotenv
import std:env
import std:println

# Load .env file (default: ".env" in current directory)
loaded = dotenv:load(nil, nil)  # Uses defaults: path=".env", override=false
println("Loaded keys: ${loaded::keys()}")

# Access loaded variables
db_url = env:var::get("DATABASE_URL", "not set")
println("Database URL: ${db_url}")

# Load from custom path
config = dotenv:load(".env.production", nil)
println("Loaded ${config::length()} variables")

# Override existing environment variables
env:var["EXISTING_KEY"] = "original_value"
dotenv:load(".env", false)  # Will NOT override EXISTING_KEY
println(env:var["EXISTING_KEY"])  # "original_value"

dotenv:load(".env", true)   # WILL override EXISTING_KEY
println(env:var["EXISTING_KEY"])  # New value from .env file
```

**Function:**
- `load(path, override)` → Loads environment variables from file
  - `path` (default: `".env"`) - Path to env file
  - `override` (default: `false`) - Whether to override existing variables
  - Returns a map of keys that were loaded

**Parsing Rules:**
- Format: `KEY=VALUE` (one per line)
- Lines starting with `#` are comments (ignored)
- Blank lines are ignored
- Whitespace around keys and values is trimmed
- Lines without `=` are silently skipped
- Empty keys (after trimming) are skipped
- Values are raw strings (no quote parsing or variable expansion)

**Error Handling:**
- Missing file returns empty map `{}` (graceful degradation)
- File read errors raise `RuntimeError::StreamError`
- Malformed lines are silently skipped

**Example .env file:**
```
# Database configuration
DATABASE_URL=postgresql://localhost:5432/mydb
DATABASE_POOL_SIZE=10

# API settings
API_KEY=secret-key-here
API_TIMEOUT=30
```

### CSV Parsing and Generation (`std:csv`)

Parse and generate CSV data with proper handling of quotes, escapes, and custom delimiters:

```suji
import std:csv
import std:println

# Parse CSV string
csv_text = "name,age,city\nAlice,30,NYC\nBob,25,LA"
rows = csv:parse(csv_text, nil)  # nil uses default delimiter ","
println(rows::length())  # 3
println(rows[0])  # ["name", "age", "city"]
println(rows[1])  # ["Alice", "30", "NYC"]

# Generate CSV from data
data = [
    ["name", "age", "city"],
    ["Alice", "30", "NYC"],
    ["Bob", "25", "LA"]
]
csv_output = csv:generate(data, nil)  # nil uses default delimiter ","
println(csv_output)
# name,age,city
# Alice,30,NYC
# Bob,25,LA

# Round-trip example
original = "a,b,c\n1,2,3"
parsed = csv:parse(original, nil)
regenerated = csv:generate(parsed, nil)
# regenerated matches original (modulo trailing newline)

# Custom delimiter (pipe-separated)
psv_text = "name|age|city\nAlice|30|NYC"
rows = csv:parse(psv_text, "|")
println(rows[0])  # ["name", "age", "city"]

# Handle quoted fields with embedded delimiters and newlines
complex = '"Smith, John",42,"New\nYork"'
parsed = csv:parse(complex, nil)
println(parsed[0][0])  # "Smith, John" (comma preserved)
println(parsed[0][2])  # "New\nYork" (newline preserved)

# Generate with proper quoting
data_with_commas = [["Last, First", "30", "City"]]
output = csv:generate(data_with_commas, nil)
# Output: "Last, First",30,City
```

**Available Functions:**
- `parse(text, delimiter)` → Parses CSV text into list of rows
  - `text` - CSV string to parse
  - `delimiter` (default: `","`) - Single-character delimiter
  - Returns list of lists (rows of string fields)

- `generate(rows, delimiter)` → Generates CSV text from data
  - `rows` - List of lists (each inner list is a row)
  - `delimiter` (default: `","`) - Single-character delimiter
  - Returns CSV string

**Notes:**
- All parsed values are strings; use `string::to_number()` for numeric conversion
- `generate()` requires all rows to be lists of strings (raises error otherwise)
- Handles quoted fields, escaped quotes, and newlines within fields correctly
- Delimiter must be a single character string
- Empty input: `parse("")` returns `[]`, `generate([])` returns `""`

**Error Cases:**
- Malformed CSV raises `RuntimeError::InvalidOperation`
- Non-list rows raise `RuntimeError::TypeError`
- Non-string cells raise `RuntimeError::TypeError`
- Invalid delimiter (non-string or multi-char) raises `RuntimeError::TypeError`

### Environment Variables (`std:env`)

#### Environment variables

Environment variables exposed as a map under the standard library at `std:env:var`. Changing `var` affects the process environment (and child processes).

```suji
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

```suji
# Access first argument (after program name)
import std:env:args

first = args::get("1", nil)
if (first != nil) {
  println("First arg: ${first}")
}
```

```suji
# Iterate all arguments by index key
import std:env:argv
import std:println

loop through argv::keys() with k {
  v = argv[k]
  println("arg[${k}] = ${v}")
}
```

### I/O and Streams (`std:io`)

Access standard streams as `stream` values. Operations may block.

```suji
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

```suji
import std:io

# Open with create and truncate controls (defaults: create=false, truncate=false)
# Create file if it doesn't exist
out = io:open("output.txt", true)
out::write("Hello, world!\n")
out::close()

# Read existing file (will error if file doesn't exist since create defaults to false)
f = io:open("output.txt")
content = f::read_all()
f::close()

# Truncate existing file (empty it before writing)
out2 = io:open("output.txt", false, true)
out2::write("New content\n")
out2::close()
```

### Print Functions (`std:print`, `std:println`)

Convenience output functions that write to streams. Default target is `std:io:stdout`.

```suji
import std:print
import std:println
import std:io

print("Hello, world!\n")
println("line without manual newline")
println("to stderr", io:stderr)
```

## Examples

### Fibonacci Sequence

```suji
import std:println

fib = |n| {
    match n {
        0 | 1 => n,
        _ => fib(n - 1) + fib(n - 2),
    }
}

# Generate first 10 Fibonacci numbers
numbers = 0..10
fibs = numbers::map(fib)
println(fibs)  # [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
```

### Quicksort

```suji
import std:println

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
        },
    }
}

unsorted = [64, 34, 25, 12, 22, 11, 90]
sorted = quicksort(unsorted)
println(sorted)  # [11, 12, 22, 25, 34, 64, 90]
```

### File Processing

```suji
# Read and process a file
content = `cat data.txt`
lines = content::split("\n")
processed = lines::map(|line| line::trim()::upper())
result = processed::join("\n")
`echo "${result}" > output.txt`
```

### Configuration Management

```suji
# Load configuration from YAML
import std:yaml
import std:println

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
git clone https://github.com/suji-lang/suji.git
cd suji

# Build the project
cargo build --release

# Run examples
cargo run -- examples/hello.si

# Start REPL
cargo run
```

## CLI & REPL Usage

Run a file:

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
- **v0.1.13**: Official name: suji. Logical split into crates.
- **v0.1.14**: Stream pipe fixes and precedence. `|` pipelines require invoked closures and bind tighter than `|>`/`<|`.
- **v0.1.15**: Error codes for all error types and better error handling. Various bugfixes.
- **v0.1.16**: Export expressions (maps and leaf values), import path resolution (files and directories), special `__builtins__` import object, standard library directory (`std/`) with delegation to builtins.
- **v0.1.17**: Match syntax changes (optional trailing commas for braced arms), new standard library modules: `std:time`, `std:uuid`, `std:encoding`, `std:math`, `std:crypto`.
- **v0.1.18**: Lazy module loading; inclusive ranges (`..=`); complex indexing expressions; short-circuit to `break`/`continue`/`return` with `&&`/`||`.
- **v0.1.19**: New standard library modules: `std:os` (OS/process information), `std:path` (cross-platform path utilities), `std:dotenv` (.env file loader), `std:csv` (CSV parsing/generation).
- **v0.1.20**: String::`trim()` with optional custom character set; negative integer literals in match patterns; `std:io:open(path, create=false, truncate=false)`.
- **v0.1.21**: Type checking methods (`is_number()`, `is_bool()`, `is_string()`, `is_list()`, `is_map()`, `is_stream()`, `is_function()`, `is_tuple()`, `is_regex()`); `std:os:stat(path, follow_symlinks)` for file metadata; filesystem operations (`os:rm()`, `os:mkdir()`, `os:rmdir()`); random string generation functions (`random:string()`, `random:hex_string()`, `random:alpha_string()`, `random:numeric_string()`, `random:alphanumeric_string()`).
