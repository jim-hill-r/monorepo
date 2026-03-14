# my_lang

A programming language implementation that compiles to LLVM IR.

## Overview

This project provides the foundation for building a new programming language with a focus on pure functions and controlled side effects.

## Language Semantics

### Keywords

- **`function`**: Defines a pure function
  - Single segment only
  - Must return a single value
  - Cannot be void
  - No side effects allowed
  
- **`input`**: Allows side effects (input operations)
  - Does NOT allow return statements
  - Used for reading external state
  
- **`output`**: Allows side effects (output operations)
  - DOES allow return statements
  - Used for writing external state

## Architecture

The language implementation consists of three main phases:

1. **Lexer** (`src/lexer.rs`): ✅ **Implemented** - Tokenizes source code into a stream of tokens
2. **Parser** (`src/parser.rs`): ✅ **Implemented** - Builds an Abstract Syntax Tree (AST) from tokens using recursive descent parsing
3. **Code Generator** (`src/codegen.rs`): ✅ **Implemented** - Compiles the AST to LLVM IR using [inkwell](https://github.com/TheDan64/inkwell)

## Code Generator

The code generator (`src/codegen.rs`) translates the AST into LLVM IR.

### Supported Constructs

- **Integer literals** — compiled to 64-bit constant values (`i64`)
- **Arithmetic operations** — `+`, `-`, `*`, `/` using LLVM integer instructions
- **Comparison operations** — `==`, `!=`, `<`, `>`, `<=`, `>=` (result is `i64`: 1 or 0)
- **Function declarations** (`function`) — compiled to LLVM functions with `i64` parameters and return type
- **Output declarations** (`output`) — same as function, return statements are allowed
- **Input declarations** (`input`) — compiled to LLVM functions without return statements
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

## Lexer Features

The lexer supports the following token types:

### Keywords
- `function`, `input`, `output`

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

- ✅ Lexer: Fully implemented with correct language keywords
- ✅ Parser: Fully implemented with recursive descent parsing for function declarations, expressions, and statements
- ✅ Code Generator: Implemented — compiles AST to LLVM IR via inkwell (integer arithmetic, comparisons, function calls, variable assignments)

## Getting Started

### Prerequisites

This project requires LLVM to be installed on your system for code generation support.

Use Cast to automatically install all required tools including LLVM 18:

```bash
cast install
```

Cast will automatically detect that this is a programming language project and install LLVM along with other required tools.

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
cast ci
```

## Future Work

See [ISSUES.md](./ISSUES.md) for planned features and improvements.

## Project Type

This project is marked as `project_type = "programming_language"` in `Cast.toml` to indicate it's a programming language implementation project in the monorepo.
