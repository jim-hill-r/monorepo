#!/bin/bash
# Test script to verify proper quoting of GitHub Actions expressions

set -euo pipefail

echo "Testing proper quoting of GitHub Actions expressions in cast-ci.yml..."
echo ""

# Test 1: Verify BASE_SHA is properly quoted
echo "Test 1: Checking BASE_SHA quoting..."
if grep -E 'BASE_SHA="\$\{\{ github\.event\.pull_request\.base\.sha \}\}"' .github/workflows/cast-ci.yml > /dev/null; then
    echo "✅ PASS: BASE_SHA is properly quoted"
else
    echo "❌ FAIL: BASE_SHA is not properly quoted"
    echo "Expected: BASE_SHA=\"\${{ github.event.pull_request.base.sha }}\""
    grep "BASE_SHA=" .github/workflows/cast-ci.yml || echo "(not found)"
    exit 1
fi

# Test 2: Verify HEAD_SHA is properly quoted
echo "Test 2: Checking HEAD_SHA quoting..."
if grep -E 'HEAD_SHA="\$\{\{ github\.event\.pull_request\.head\.sha \}\}"' .github/workflows/cast-ci.yml > /dev/null; then
    echo "✅ PASS: HEAD_SHA is properly quoted"
else
    echo "❌ FAIL: HEAD_SHA is not properly quoted"
    echo "Expected: HEAD_SHA=\"\${{ github.event.pull_request.head.sha }}\""
    grep "HEAD_SHA=" .github/workflows/cast-ci.yml || echo "(not found)"
    exit 1
fi

echo ""
echo "All quoting tests passed! ✅"
echo ""
echo "Summary:"
echo "- BASE_SHA and HEAD_SHA are properly quoted to prevent bash errors"
echo "- This prevents issues when the expressions evaluate to empty or special values"
