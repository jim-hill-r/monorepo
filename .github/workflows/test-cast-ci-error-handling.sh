#!/bin/bash
# Test script to verify cast-ci.yml error handling improvements

set -euo pipefail

echo "Testing cast-ci.yml error handling improvements..."
echo ""

# Test 1: Verify the workflow contains the fetch commands
echo "Test 1: Checking for explicit git fetch commands..."
if grep -q "git fetch origin.*BASE_SHA" .github/workflows/cast-ci.yml && \
   grep -q "git fetch origin.*HEAD_SHA" .github/workflows/cast-ci.yml; then
    echo "✅ PASS: Workflow contains explicit git fetch commands"
else
    echo "❌ FAIL: Workflow missing explicit git fetch commands"
    exit 1
fi

# Test 2: Verify the workflow checks git diff exit code
echo "Test 2: Checking for git diff error handling..."
if grep -q "if \[ \$? -ne 0 \]" .github/workflows/cast-ci.yml; then
    echo "✅ PASS: Workflow checks git diff exit code"
else
    echo "❌ FAIL: Workflow does not check git diff exit code"
    exit 1
fi

# Test 3: Verify the workflow captures stderr from cast command (which includes git diff)
echo "Test 3: Checking if cast command captures stderr..."
if grep -Pzo "CAST_BIN.*project.*with-changes.*\n.*2>&1" .github/workflows/cast-ci.yml > /dev/null 2>&1 || \
   grep "with-changes.*2>&1" .github/workflows/cast-ci.yml > /dev/null 2>&1; then
    echo "✅ PASS: Workflow captures stderr from cast command"
else
    echo "❌ FAIL: Workflow does not capture stderr from cast command"
    exit 1
fi

# Test 4: Verify the workflow prints error output on failure
echo "Test 4: Checking for error output printing..."
if grep -q "echo.*output.*CHANGED_PROJECTS" .github/workflows/cast-ci.yml; then
    echo "✅ PASS: Workflow prints error output on failure"
else
    echo "❌ FAIL: Workflow does not print error output"
    exit 1
fi

# Test 5: Verify the workflow exits with error on cast command failure
echo "Test 5: Checking for exit on error..."
if grep -A 3 "if \[ \$? -ne 0 \]" .github/workflows/cast-ci.yml | grep -q "exit 1"; then
    echo "✅ PASS: Workflow exits with error on cast command failure"
else
    echo "❌ FAIL: Workflow does not exit with error on cast command failure"
    exit 1
fi

# Test 6: Verify fetch commands use || true to not fail on missing commits
echo "Test 6: Checking fetch commands don't fail on missing commits..."
if grep -q "git fetch.*|| true" .github/workflows/cast-ci.yml; then
    echo "✅ PASS: Fetch commands use || true for graceful failure"
else
    echo "❌ FAIL: Fetch commands may fail the workflow unnecessarily"
    exit 1
fi

echo ""
echo "All error handling tests passed! ✅"
echo ""
echo "Summary:"
echo "- The workflow now explicitly fetches both BASE_SHA and HEAD_SHA"
echo "- Cast command errors (including git diff errors) are caught and reported clearly"
echo "- The workflow exits with a clear error message when cast command fails"
echo "- Fetch failures don't prevent the workflow from attempting to find changed projects"
