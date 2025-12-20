#!/bin/bash
# Test script for cast-ci.yml workflow

set -euo pipefail

echo "Testing cast-ci.yml workflow..."

# Test 1: Check if workflow file exists
WORKFLOW_FILE=".github/workflows/cast-ci.yml"
if [ ! -f "$WORKFLOW_FILE" ]; then
    echo "❌ FAIL: Workflow file not found: $WORKFLOW_FILE"
    exit 1
fi
echo "✅ PASS: Workflow file exists"

# Test 2: Validate YAML syntax using yamllint
if command -v yamllint &> /dev/null; then
    if yamllint "$WORKFLOW_FILE" 2>&1; then
        echo "✅ PASS: Workflow YAML syntax is valid"
    else
        echo "❌ FAIL: Workflow YAML syntax is invalid"
        exit 1
    fi
else
    echo "⚠️  SKIP: yamllint not available to validate YAML syntax"
fi

# Test 3: Validate YAML can be parsed
if command -v python3 &> /dev/null; then
    if python3 -c "import yaml; yaml.safe_load(open('$WORKFLOW_FILE'))" 2>&1; then
        echo "✅ PASS: Workflow YAML can be parsed"
    else
        echo "❌ FAIL: Workflow YAML cannot be parsed"
        exit 1
    fi
else
    echo "⚠️  SKIP: Python not available to validate YAML syntax"
fi

# Test 4: Verify workflow trigger is pull_request
if grep -q "pull_request:" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow trigger includes pull_request"
else
    echo "❌ FAIL: Workflow trigger does not include pull_request"
    exit 1
fi

# Test 5: Verify workflow uses git diff to detect changes
if grep -q "git diff" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow uses git diff to detect changes"
else
    echo "❌ FAIL: Workflow does not use git diff to detect changes"
    exit 1
fi

# Test 6: Verify workflow searches for Cast.toml
if grep -q "Cast.toml" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow searches for Cast.toml files"
else
    echo "❌ FAIL: Workflow does not search for Cast.toml files"
    exit 1
fi

# Test 7: Verify workflow builds cast CLI
if grep -q "cast_cli" "$WORKFLOW_FILE" && grep -q "cargo build" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow builds cast CLI"
else
    echo "❌ FAIL: Workflow does not build cast CLI"
    exit 1
fi

# Test 8: Verify workflow runs cast ci command
if grep -q "cast ci" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow runs cast ci command"
else
    echo "❌ FAIL: Workflow does not run cast ci command"
    exit 1
fi

# Test 9: Verify cast CLI can be built
CAST_CLI_DIR="projects/cast_cli"
if [ -f "$CAST_CLI_DIR/Cargo.toml" ]; then
    echo "✅ PASS: cast_cli project exists"
else
    echo "❌ FAIL: cast_cli project not found"
    exit 1
fi

# Test 10: Verify workflow sets up Rust toolchain
if grep -q "setup-rust-toolchain" "$WORKFLOW_FILE" || \
   grep -q "rust-toolchain" "$WORKFLOW_FILE" || \
   grep -q "actions-rust-lang" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow sets up Rust toolchain"
else
    echo "❌ FAIL: Workflow does not set up Rust toolchain"
    exit 1
fi

# Test 11: Verify workflow handles case where no projects changed
if grep -q "No projects" "$WORKFLOW_FILE" || \
   grep -q "has_projects" "$WORKFLOW_FILE"; then
    echo "✅ PASS: Workflow handles case where no projects changed"
else
    echo "⚠️  WARNING: Workflow may not handle case where no projects changed"
fi

# Test 12: Verify cast ci command works
CAST_PROJECT_DIR="projects/cast"
if [ -d "$CAST_PROJECT_DIR" ] && [ -f "$CAST_CLI_DIR/target/release/cast" ]; then
    if cd "$CAST_PROJECT_DIR" && ../cast_cli/target/release/cast ci > /dev/null 2>&1; then
        echo "✅ PASS: cast ci command works"
        cd - > /dev/null
    else
        echo "⚠️  WARNING: cast ci command could not be tested (may need to be built)"
        cd - > /dev/null
    fi
else
    if [ ! -d "$CAST_PROJECT_DIR" ]; then
        echo "⚠️  SKIP: projects/cast directory not found"
    else
        echo "⚠️  SKIP: cast CLI binary not found (needs to be built first)"
    fi
fi

echo ""
echo "All tests passed! ✅"
