#!/bin/bash
# Test script for start-a-new-task.yml workflow

set -e

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
    python3 -c "import yaml; yaml.safe_load(open('$WORKFLOW_FILE'))" 2>&1
    if [ $? -eq 0 ]; then
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

echo ""
echo "All tests passed! ✅"
