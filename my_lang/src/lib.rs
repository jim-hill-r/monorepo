//! my_lang: A programming language implementation with LLVM backend
//!
//! This project provides a foundation for building a new programming language
//! that compiles to LLVM IR for execution on multiple platforms.
//!
//! ## Architecture
//!
//! The language implementation consists of:
//! - **Lexer**: Tokenizes source code into a stream of tokens
//! - **Parser**: Builds an Abstract Syntax Tree (AST) from tokens
//! - **Code Generator**: Compiles the AST to LLVM IR (using inkwell)
//!
//! ## LLVM Support
//!
//! This project uses [inkwell](https://github.com/TheDan64/inkwell) to safely
//! interface with LLVM. The LLVM backend enables:
//! - Cross-platform code generation
//! - Optimizations via LLVM's optimization passes
//! - JIT compilation for interactive execution
//! - AOT compilation for native binaries
//!
//! ## Supported Platforms
//!
//! - Linux x86_64 (with LLVM 18 installed)
//! - macOS ARM64 (aarch64-apple-darwin, with LLVM 18 installed)
//!
//! ## Current Status
//!
//! - ✅ Lexer: Fully implemented with language keywords
//! - ✅ Parser: Fully implemented with recursive descent parsing
//! - ✅ Code Generator: Implemented — compiles AST to LLVM IR (integer arithmetic, comparisons, functions, assignments)

pub mod codegen;
pub mod lexer;
pub mod parser;
