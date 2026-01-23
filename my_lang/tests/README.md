# my_lang Tests

This directory contains tests for the my_lang programming language project.

## Running Tests

Tests are run using Cargo:

```bash
cargo test
```

Or using Cast:

```bash
cast test
```

## Test Organization

Tests are currently organized as:
- Unit tests in each module (lexer.rs, parser.rs)
- Integration tests will be added here as the project develops

## Writing Tests

When adding new features to the lexer, parser, or code generator, please add corresponding tests following Rust testing best practices.

For more information on Rust testing, see the [Rust Book](https://doc.rust-lang.org/book/ch11-00-testing.html).
