# my_lang Tests

This directory contains tests for the my_lang programming language project.

## Running Tests

Tests are run using Cargo:

```bash
cargo test
```

Or using mise:

```bash
mise run test
```

## Test Organization

Tests are currently organized as:
- Unit tests in each module (`src/*.rs`)
- `integration_tests.rs`: the lexer → parser → codegen pipeline and the JIT runner
- `aot_tests.rs`: AOT compilation; builds real executables and object files and
  runs them, so it needs a system C compiler (`cc`)

## Writing Tests

When adding new features to the lexer, parser, or code generator, please add corresponding tests following Rust testing best practices.

For more information on Rust testing, see the [Rust Book](https://doc.rust-lang.org/book/ch11-00-testing.html).
