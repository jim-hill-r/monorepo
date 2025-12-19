# Troubleshooting GitHub Workflows

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
