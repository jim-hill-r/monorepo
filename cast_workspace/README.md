# Cast Workspace

This workspace contains the Cast monorepo tooling projects.

## Projects

- **core** (`cast_core`) - Core Cast library for monorepo operations
- **cli** (`cast_cli`) - Cast command-line interface
- **vscode_ext** (`cast_vscode`) - VSCode extension for Cast (to be migrated)

## Building

Build all workspace members:
```bash
cargo build --workspace
```

Build specific member:
```bash
cargo build -p cast_core
cargo build -p cast_cli
```

## Testing

Test all workspace members:
```bash
cargo test --workspace
```

## Development

The Cast CLI is the primary development tool:
```bash
cd cli
cargo build --release
./target/release/cast --help
```
