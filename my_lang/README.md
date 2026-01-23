# my_lang

A programming language implementation that will eventually compile to LLVM.

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
2. **Parser** (`src/parser.rs`): Builds an Abstract Syntax Tree (AST) from tokens
3. **Code Generator** (future): Compiles the AST to LLVM IR

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
- ⏳ Parser: Placeholder implementation
- ⏳ Code Generator: Not yet implemented

## Getting Started

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
