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

# Test 3: Check if agent-copilot binary exists
BINARY_FILE="projects/agent-copilot/artifacts/x86_64-unknown-linux-gnu/agent-copilot"
if [ ! -f "$BINARY_FILE" ]; then
    echo "❌ FAIL: agent-copilot binary not found: $BINARY_FILE"
    exit 1
fi
echo "✅ PASS: agent-copilot binary exists"

# Test 4: Validate YAML syntax using Python (available in GitHub Actions)
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

# Test 5: Check workflow uses agent-copilot binary
if grep -q "agent-copilot" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow uses agent-copilot binary"
else
    echo "❌ FAIL: Workflow does not use agent-copilot binary"
    exit 1
fi

# Test 6: Verify workflow does not use gh issue create (old method)
if grep -q 'gh issue create' "$WORKFLOW_FILE"; then
    echo "❌ FAIL: Workflow should not use gh issue create (use agent-copilot instead)"
    exit 1
else
    echo "✅ PASS: Workflow does not use gh issue create"
fi

# Test 7: Verify workflow trigger is still pull_request closed
if grep -q "pull_request:" "$WORKFLOW_FILE" && grep -q "types: \[closed\]" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow trigger is correct (pull_request closed)"
else
    echo "❌ FAIL: Workflow trigger is not configured correctly"
    exit 1
fi

# Test 8: Check that issues permission is present (for backward compatibility)
if grep -q "issues: write" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow has 'issues: write' permission"
else
    echo "⚠️  WARNING: Workflow missing 'issues: write' permission (may not be needed with direct Copilot API)"
fi

# Test 9: Check workflow uses correct Copilot user login
if grep -q "user.login == 'Copilot'" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow uses correct Copilot user login"
else
    echo "❌ FAIL: Workflow does not use correct Copilot user login (should be 'Copilot')"
    exit 1
fi

# Test 10: Check workflow uses GITHUB_TOKEN
if grep -q 'GITHUB_TOKEN:.*secrets\.GITHUB_TOKEN' "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow uses GITHUB_TOKEN"
else
    echo "⚠️  WARNING: Workflow should use GITHUB_TOKEN"
fi

# Test 11: Check workflow uses --prompt-file flag
if grep -q "prompt-file" "$WORKFLOW_FILE" || grep -q "--prompt-file" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow uses --prompt-file flag"
else
    echo "❌ FAIL: Workflow should use --prompt-file flag to read from agent prompt"
    exit 1
fi

# Test 12: Check workflow validates binary exists
if grep -q "agent-copilot binary" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow validates agent-copilot binary exists"
else
    echo "⚠️  WARNING: Workflow should validate agent-copilot binary exists"
fi

# Test 13: Check workflow has concurrency check for running agents
if grep -q "Check for running agent tasks" "$WORKFLOW_FILE" && grep -q "gh pr list" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow includes concurrency check for running agents"
else
    echo "❌ FAIL: Workflow should check for running agents before starting a new task"
    exit 1
fi

# Test 14: Check workflow has conditional steps based on running agents
if grep -q "if: steps.check_running_agents.outputs.skip_task" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow has conditional steps based on running agents check"
else
    echo "❌ FAIL: Workflow should conditionally execute steps based on running agents check"
    exit 1
fi

echo ""
echo "All tests passed! ✅"
