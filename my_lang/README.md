# my_lang

A placeholder for a new programming language that will eventually compile to LLVM.

## Overview

This project provides the foundation for building a new programming language. It's currently in the early stages with basic placeholder implementations for the core components.

## Architecture

The language implementation consists of three main phases:

1. **Lexer** (`src/lexer.rs`): Tokenizes source code into a stream of tokens
2. **Parser** (`src/parser.rs`): Builds an Abstract Syntax Tree (AST) from tokens
3. **Code Generator** (future): Compiles the AST to LLVM IR

## Current Status

This is a placeholder implementation with basic structure in place. The actual lexer, parser, and code generation functionality will be implemented in future iterations.

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
