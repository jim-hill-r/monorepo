# my_lang

A programming language implementation that will eventually compile to LLVM.

## Overview

This project provides the foundation for building a new programming language. The lexer has been implemented with support for basic token types.

## Architecture

The language implementation consists of three main phases:

1. **Lexer** (`src/lexer.rs`): ✅ **Implemented** - Tokenizes source code into a stream of tokens
2. **Parser** (`src/parser.rs`): Builds an Abstract Syntax Tree (AST) from tokens
3. **Code Generator** (future): Compiles the AST to LLVM IR

## Lexer Features

The lexer supports the following token types:

### Keywords
- `let`, `fn`, `if`, `else`, `return`, `while`, `for`, `true`, `false`

### Identifiers
- Variable and function names (starting with letter or underscore)
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

let input = "let x = 42;".to_string();
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

- ✅ Lexer: Fully implemented with comprehensive tests
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
