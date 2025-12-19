#!/bin/bash
# Test script for start-a-new-task.yml workflow

set -euo pipefail

echo "Testing start-a-new-task.yml workflow..."

# Test 1: Check if workflow file exists
WORKFLOW_FILE=".github/workflows/start-a-new-task.yml"
if [ ! -f "$WORKFLOW_FILE" ]; then
    echo "❌ FAIL: Workflow file not found: $WORKFLOW_FILE"
    exit 1
fi
echo "✅ PASS: Workflow file exists"

# Test 2: Check if agent prompt file exists
PROMPT_FILE=".github/agent-prompts/start-a-new-task.md"
if [ ! -f "$PROMPT_FILE" ]; then
    echo "❌ FAIL: Agent prompt file not found: $PROMPT_FILE"
    exit 1
fi
echo "✅ PASS: Agent prompt file exists"

# Test 3: Validate YAML syntax using Python (available in GitHub Actions)
if command -v python3 &> /dev/null; then
    if python3 -c "import yaml; yaml.safe_load(open('$WORKFLOW_FILE'))" 2>&1; then
        echo "✅ PASS: Workflow YAML syntax is valid"
    else
        echo "❌ FAIL: Workflow YAML syntax is invalid"
        exit 1
    fi
else
    echo "⚠️  SKIP: Python not available to validate YAML syntax"
fi

# Test 4: Check workflow uses gh issue create
if grep -q "gh issue create" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow uses gh issue create"
else
    echo "❌ FAIL: Workflow does not use gh issue create"
    exit 1
fi

# Test 5: Check workflow assigns issue to @copilot
if grep -q 'assignee.*"@copilot"' "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow assigns issue to @copilot"
else
    echo "❌ FAIL: Workflow does not assign issue to @copilot"
    exit 1
fi

# Test 6: Verify workflow trigger is still pull_request closed
if grep -q "pull_request:" "$WORKFLOW_FILE" && grep -q "types: \[closed\]" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow trigger is correct (pull_request closed)"
else
    echo "❌ FAIL: Workflow trigger is not configured correctly"
    exit 1
fi

# Test 7: Check that issues permission is present (required for creating issues)
if grep -q "issues: write" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow has 'issues: write' permission"
else
    echo "❌ FAIL: Workflow missing 'issues: write' permission (required for creating issues)"
    exit 1
fi

# Test 8: Check workflow uses correct Copilot user login
if grep -q "user.login == 'Copilot'" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow uses correct Copilot user login"
else
    echo "❌ FAIL: Workflow does not use correct Copilot user login (should be 'Copilot', not 'copilot-swe-agent[bot]')"
    exit 1
fi

# Test 9: Check workflow uses GITHUB_TOKEN (standard for issue creation)
if grep -q 'GH_TOKEN:.*secrets\.GITHUB_TOKEN' "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow uses GITHUB_TOKEN for issue creation"
else
    echo "⚠️  WARNING: Workflow should use GITHUB_TOKEN for issue creation"
fi

# Test 10: Check workflow uses body-file flag
if grep -q "body-file" "$WORKFLOW_FILE" || grep -q "--body-file" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow uses --body-file flag to read from agent prompt"
else
    echo "⚠️  WARNING: Workflow should use --body-file flag to read issue body from file"
fi

echo ""
echo "All tests passed! ✅"
