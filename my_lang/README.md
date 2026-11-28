# my_lang

[![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badges/my_lang.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json)

A programming language implementation that compiles to LLVM IR.

## Overview

This project provides the foundation for building a new programming language with a focus on pure functions and controlled side effects.

## Language Semantics

### Keywords

- **`function`**: Defines a pure function
  - Single segment only
  - Gives a single value: the `return` it reaches, or else the expression
    its body ends with (`function add(a, b) { a + b }` gives `a + b`)
  - Cannot be void: a body with neither (empty, or ending in an assignment)
    is a compile error naming the function
  - No side effects allowed
  
- **`input`**: Allows side effects (input operations)
  - Does NOT allow return statements, and a trailing expression is only
    evaluated, so it always gives 0
  - Used for reading external state
  
- **`output`**: Allows side effects (output operations)
  - DOES allow return statements, and gives its trailing expression's value
    like a `function`
  - Used for writing external state

- **`return`**: Returns a value from the current function
  - Allowed in `function` and `output` declarations
  - Not allowed in `input` declarations

## Architecture

The language implementation consists of four main phases:

1. **Lexer** (`src/lexer.rs`): ✅ **Implemented** - Tokenizes source code into a stream of tokens
2. **Parser** (`src/parser.rs`): ✅ **Implemented** - Builds an Abstract Syntax Tree (AST) from tokens using recursive descent parsing
3. **Code Generator** (`src/codegen.rs`): ✅ **Implemented** - Compiles the AST to LLVM IR using [inkwell](https://github.com/TheDan64/inkwell)
4. **JIT Runner** (`src/jit.rs`): ✅ **Implemented** - Executes compiled functions directly in-process via LLVM's JIT engine
5. **AOT Compiler** (`src/aot.rs`): ✅ **Implemented** - Emits native object files and links standalone executables

## Code Generator

The code generator (`src/codegen.rs`) translates the AST into LLVM IR.

### Supported Constructs

- **Integer literals** — compiled to 64-bit constant values (`i64`)
- **Arithmetic operations** — `+`, `-`, `*`, `/` using LLVM integer instructions
- **Comparison operations** — `==`, `!=`, `<`, `>`, `<=`, `>=` (result is `i64`: 1 or 0)
- **Function declarations** (`function`) — compiled to LLVM functions with `i64` parameters and return type
- **Trailing expressions** — in a `function` or `output`, an expression that ends the body is returned, as if written `return <expression>`
- **Output declarations** (`output`) — same as function, return statements are allowed
- **Input declarations** (`input`) — compiled to LLVM functions without return statements; they return 0
- **Variable assignments** — stack-allocated via `alloca`/`store`/`load`
- **Function calls** — intra-module calls resolved by name

### Type System

All values are currently `i64` (64-bit signed integer). String literals parse correctly but are not yet supported by the code generator.

### Usage Example

```rust
use inkwell::context::Context;
use my_lang::codegen::CodeGenerator;
use my_lang::lexer::Lexer;
use my_lang::parser::Parser;

let source = "function add(a, b) { a + b }".to_string();
let lexer = Lexer::new(source);
let mut parser = Parser::new(lexer);
let program = parser.parse().unwrap();

let context = Context::create();
let mut codegen = CodeGenerator::new(&context, "my_module");
codegen.compile_program(&program).unwrap();

// Print generated LLVM IR
println!("{}", codegen.module().print_to_string());
```

## JIT Runner

The JIT runner (`src/jit.rs`) executes compiled my_lang functions directly in-process
using LLVM's JIT compilation engine.

### Usage Example

```rust
use inkwell::context::Context;
use my_lang::jit::JitRunner;

let context = Context::create();
let runner = JitRunner::new(&context, "function add(a, b) { return a + b }")
    .expect("compile should succeed");

let result = runner.call("add", &[3, 4]).expect("call should succeed");
assert_eq!(result, 7);
```

### API

- **`JitRunner::new(context, source)`** — Compiles `source` (lexer → parser → codegen → JIT engine).
- **`runner.call(name, args)`** — Calls the named function with up to 8 `i64` arguments and returns an `i64` result.

### Limitations

- All values are `i64` (64-bit signed integer).
- Up to 8 function arguments are supported.

## AOT Compiler

The AOT compiler (`src/aot.rs`) compiles my_lang ahead of time to native code
for the host machine.

### Usage Example

```rust
use std::path::Path;
use my_lang::aot;

// A standalone executable that prints 42.
aot::build_executable("function main() { return 6 * 7 }", Path::new("answer"))
    .expect("build should succeed");

// An object file exporting `add` for a C program to link.
aot::compile_to_object("function add(a, b) { return a + b }", Path::new("add.o"))
    .expect("compile should succeed");
```

### API

- **`aot::build_executable(source, output)`** — Builds an executable whose
  entry point is the my_lang function `main`. The executable calls `main`,
  prints its result and a newline on standard output, and exits with status 0.
- **`aot::compile_to_object(source, path)`** — Writes a host object file. Each
  my_lang function is an exported C symbol taking and returning `long long`
  (`i64`).

### Limitations

- `main` must take no parameters.
- Linking needs a system C compiler: `$CC` if set, otherwise `cc`.
- Only the host target is supported (no cross-compilation yet).

## Command Line

The `my_lang` binary compiles and runs programs without writing any Rust.
Source files can have any extension; the examples use `.my`.

```bash
cargo install --path .              # or: cargo run -- <command>

my_lang run answer.my               # JIT-runs main and prints its result
my_lang run add.my 3 4              # passes 3 and 4 to main(a, b)
my_lang build answer.my             # writes the executable ./answer
my_lang build answer.my -o bin/app  # writes bin/app
./answer                            # prints main's result
```

- **`my_lang run <file> [args...]`** compiles `file` with the JIT, calls its
  `main` with the integer arguments (one per parameter, at most 8) and prints
  the result.
- **`my_lang build <file> [-o <output>]`** compiles `file` to a native
  executable with the AOT compiler. Without `-o` the executable is named after
  the source file, minus its extension. Its `main` must take no parameters.

Errors go to standard error, prefixed `my_lang:`, and the exit status is 1.
The code is in `src/cli.rs`; `src/main.rs` only parses arguments.

## Lexer Features

The lexer supports the following token types:

### Keywords
- `function`, `input`, `output`, `return`

### Identifiers
- Function and variable names (starting with letter or underscore)
- Examples: `x`, `foo`, `_private`, `my_var`

### Literals
- Integer literals: `0`, `123`, `456789`
- String literals: `"hello"`, `"world"`

### Operators
- Arithmetic: `+`, `-`, `*`, `/`
- Assignment: `=`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`

### Delimiters
- Parentheses: `(`, `)`
- Braces: `{`, `}`
- Others: `,`, `;`

## Usage Example

```rust
use my_lang::lexer::{Lexer, Token};

let input = "function add(a, b) { a + b }".to_string();
let mut lexer = Lexer::new(input);

// Tokenize the input
while let token = lexer.next_token() {
    if token == Token::Eof {
        break;
    }
    println!("{:?}", token);
}
```

## Current Status

- ✅ Lexer: Fully implemented with correct language keywords (including `return`)
- ✅ Parser: Fully implemented with recursive descent parsing for function declarations, expressions, statements, and `return`
- ✅ Code Generator: Implemented — compiles AST to LLVM IR via inkwell (integer arithmetic, comparisons, function calls, variable assignments)
- ✅ JIT Runner: Implemented — executes compiled functions directly in-process via LLVM JIT
- ✅ AOT Compiler: Implemented — emits native object files and executables
- ✅ Command line: `my_lang build` and `my_lang run`

## Getting Started

### Prerequisites

This project requires LLVM to be installed on your system for code generation support.

Install the Rust toolchain with mise (LLVM 18 comes from your package manager; in CI the `ci:deps` task installs it):

```bash
mise install
```

`mise.toml` sets `LLVM_SYS_181_PREFIX` to `/usr/lib/llvm-18`; change it if your LLVM lives elsewhere.

#### Manual Installation (Alternative)

If you prefer to install LLVM manually:

**Linux (Ubuntu/Debian)**:
```bash
sudo apt-get update
sudo apt-get install llvm-18-dev libpolly-18-dev
```

**macOS**:
```bash
brew install llvm@18
export LLVM_SYS_180_PREFIX=$(brew --prefix llvm@18)
```

#### Verifying Installation

```bash
# Check LLVM installation
llvm-config-18 --version
```

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### CI

```bash
mise run ci
```

## Future Work

See [docs/issues/](docs/issues/) for planned features and improvements.

## Project Type

This project uses the shared `rust` task file (`standards/tasks/rust.toml`) through its `mise.toml`.
