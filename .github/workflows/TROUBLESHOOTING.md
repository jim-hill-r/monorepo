# Troubleshooting GitHub Workflows

## Issue: Cast CI fails with "Cargo fmt check failed"

### Problem
The Cast CI workflow fails during the `cast ci` step with an error about formatting differences:
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
The issue was found in `cast_cli/src/bin/cast.rs` where imports were not alphabetically sorted:
```rust
// Before (incorrect):
use cast::args::{Args, execute};

// After (correct):
use cast::args::{execute, Args};
```

Running `cargo fmt` automatically fixed this and all similar formatting issues.

## Issue: "could not assign user: 'copilot-swe-agent' not found"

### Problem
When running the `start-a-new-task.yml` workflow, it failed with the error:
```
could not assign user: 'copilot-swe-agent' not found
```

### Root Cause
The workflow was attempting to assign the created issue to `@copilot` using the `--assignee "@copilot"` flag in the `gh issue create` command. However:
- GitHub Copilot is not a regular user that can be assigned issues
- The `--assignee` flag expects valid GitHub usernames that exist in the repository
- Attempting to assign to `@copilot` causes the workflow to fail

### Solution
Remove the `--assignee "@copilot"` line from the workflow. GitHub Copilot does not need to be explicitly assigned to issues to respond to them. The workflow now simply creates the issue with the appropriate content, and GitHub Copilot will automatically detect and respond to it.

### Changes Made
1. **Removed `--assignee "@copilot"` from workflow** - The workflow no longer attempts to assign the issue
2. **Updated test script** - Changed test to verify the `--assignee` flag is NOT present
3. **Updated documentation** - Clarified that issues are not explicitly assigned

### Result
The workflow now successfully creates issues without the assignment error, and GitHub Copilot continues to function as expected.

## Testing
Run the test script to verify the workflow configuration:
```bash
bash .github/workflows/test-start-a-new-task.sh
```

All tests should pass, including the new check that verifies the `--assignee` flag is not used.
