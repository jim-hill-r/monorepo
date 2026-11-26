# Troubleshooting GitHub Workflows

## Issue: Pull Request CI fails with "Cargo fmt check failed"

### Problem
The Pull Request CI workflow fails during the `cast ci` step with an error about formatting differences:
```
Error: ci error: Cargo fmt check failed: Diff in /path/to/file.rs
```

### Root Cause
The `cast ci` command runs `cargo fmt --check` as its first step to ensure code is properly formatted. If any files in the project are not formatted according to Rust's standard formatting rules, the check fails. Common issues include:
- Import statements not in alphabetical order
- Inconsistent spacing or indentation
- Line length violations

### Solution
Run `cargo fmt` in the affected project to automatically format all code according to Rust standards:

```bash
cd <project_directory>
cargo fmt
git add .
git commit -m "Fix formatting issues"
```

### Prevention
To prevent this issue:
1. Run `cargo fmt` before committing code
2. Configure your editor to format on save
3. Add a pre-commit hook that runs `cargo fmt`

### Example
The issue was found in `cast_workspace/cli/src/bin/cast.rs` where imports were not alphabetically sorted:
```rust
// Before (incorrect):
use cast::args::{Args, execute};

// After (correct):
use cast::args::{execute, Args};
```

Running `cargo fmt` automatically fixed this and all similar formatting issues.
