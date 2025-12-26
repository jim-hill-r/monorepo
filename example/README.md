# Example

This workspace contains exemplar projects that serve as templates for creating new projects in the monorepo.

## Exemplar Projects

### base
Basic project structure with standard directories (benches, book, docs, examples, src, tests).

### example_rust_library
Template for creating Rust libraries.

### binary
Template for creating Rust binary/CLI applications.

## Usage

These projects are marked with `exemplar = true` in their `Cast.toml` files. The Cast CLI uses them as templates when creating new projects:

```bash
cast project new <name>
```

When creating a new project, Cast copies exemplar projects in alphabetical order, with later ones overwriting files from earlier ones.

## Modifying Exemplar Projects

⚠️ **Warning**: These are templates used by the Cast CLI. Changes to these projects will affect all new projects created in the future. Only modify them when you want to update the standard project structure.
