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

# Test 4: Check workflow doesn't reference GitHub issues
if grep -q "issues.create" "$WORKFLOW_FILE"; then
    echo "❌ FAIL: Workflow still uses issues.create (should use gh agent-task create)"
    exit 1
fi
echo "✅ PASS: Workflow does not use issues.create"

# Test 5: Check workflow uses gh agent-task create
if grep -q "gh agent-task create" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow uses gh agent-task create"
else
    echo "❌ FAIL: Workflow does not use gh agent-task create"
    exit 1
fi

# Test 6: Verify workflow trigger is still pull_request closed
if grep -q "pull_request:" "$WORKFLOW_FILE" && grep -q "types: \[closed\]" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow trigger is correct (pull_request closed)"
else
    echo "❌ FAIL: Workflow trigger is not configured correctly"
    exit 1
fi

# Test 7: Check that issues permission is removed (no longer needed)
if grep -q "issues: write" "$WORKFLOW_FILE"; then
    echo "⚠️  WARNING: Workflow still has 'issues: write' permission (may not be needed)"
else
    echo "✅ PASS: Workflow does not have unnecessary 'issues: write' permission"
fi

# Test 8: Check workflow uses correct Copilot user login
if grep -q "user.login == 'Copilot'" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow uses correct Copilot user login"
else
    echo "❌ FAIL: Workflow does not use correct Copilot user login (should be 'Copilot', not 'copilot-swe-agent[bot]')"
    exit 1
fi

# Test 9: Check workflow uses OAuth-compatible token (not GITHUB_TOKEN)
if grep -q "GH_TOKEN.*GITHUB_TOKEN" "$WORKFLOW_FILE"; then
    echo "❌ FAIL: Workflow uses GITHUB_TOKEN which doesn't have OAuth scopes for 'gh agent-task'"
    echo "   The workflow should use a Personal Access Token (PAT) stored as a secret like GH_PAT"
    exit 1
else
    echo "✅ PASS: Workflow does not use GITHUB_TOKEN (avoiding OAuth scope issue)"
fi

# Test 10: Check workflow uses a PAT secret for authentication
if grep -q "GH_TOKEN.*secrets\.GH_PAT" "$WORKFLOW_FILE" || grep -q "GH_TOKEN.*secrets\..*PAT" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow uses a PAT secret for authentication"
else
    echo "⚠️  WARNING: Workflow should use a PAT secret (e.g., secrets.GH_PAT) for gh agent-task authentication"
fi

# Test 11: Check workflow validates token existence
if grep -q "Check for required token" "$WORKFLOW_FILE" || grep -q "GH_PAT secret is not configured" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow validates token existence before use"
else
    echo "⚠️  WARNING: Workflow should validate that required PAT token is configured"
fi

echo ""
echo "All tests passed! ✅"
