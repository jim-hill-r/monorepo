#!/usr/bin/env bash
# Validation tests for pane-cloudflare project

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "=== Running pane-cloudflare validation tests ==="
echo ""

# Test 1: Check required files exist
echo "Test 1: Checking required files exist..."
required_files=(
    "README.md"
    "wrangler.toml"
    "Cast.toml"
    ".gitignore"
    "deploy.sh"
    "ISSUES.md"
)

for file in "${required_files[@]}"; do
    if [ ! -f "$PROJECT_DIR/$file" ]; then
        echo "  ❌ FAIL: $file not found"
        exit 1
    else
        echo "  ✓ $file exists"
    fi
done

# Test 2: Check deploy.sh is executable
echo ""
echo "Test 2: Checking deploy.sh is executable..."
if [ ! -x "$PROJECT_DIR/deploy.sh" ]; then
    echo "  ❌ FAIL: deploy.sh is not executable"
    exit 1
else
    echo "  ✓ deploy.sh is executable"
fi

# Test 3: Validate bash script syntax
echo ""
echo "Test 3: Validating deploy.sh syntax..."
if bash -n "$PROJECT_DIR/deploy.sh"; then
    echo "  ✓ deploy.sh has valid bash syntax"
else
    echo "  ❌ FAIL: deploy.sh has syntax errors"
    exit 1
fi

# Test 4: Check wrangler.toml has required fields
echo ""
echo "Test 4: Validating wrangler.toml..."
if grep -q 'name = "pane"' "$PROJECT_DIR/wrangler.toml"; then
    echo "  ✓ wrangler.toml has project name"
else
    echo "  ❌ FAIL: wrangler.toml missing project name"
    exit 1
fi

if grep -q 'compatibility_date' "$PROJECT_DIR/wrangler.toml"; then
    echo "  ✓ wrangler.toml has compatibility_date"
else
    echo "  ❌ FAIL: wrangler.toml missing compatibility_date"
    exit 1
fi

# Test 5: Check README has required sections
echo ""
echo "Test 5: Validating README.md..."
required_sections=(
    "Prerequisites"
    "Building Pane"
    "Deploying to Cloudflare Pages"
)

for section in "${required_sections[@]}"; do
    if grep -q "$section" "$PROJECT_DIR/README.md"; then
        echo "  ✓ README has '$section' section"
    else
        echo "  ❌ FAIL: README missing '$section' section"
        exit 1
    fi
done

# Test 6: Check .gitignore has cloudflare-specific entries
echo ""
echo "Test 6: Validating .gitignore..."
if grep -q '.wrangler/' "$PROJECT_DIR/.gitignore"; then
    echo "  ✓ .gitignore has .wrangler/ entry"
else
    echo "  ❌ FAIL: .gitignore missing .wrangler/ entry"
    exit 1
fi

# Test 7: Verify relative path to pane project
echo ""
echo "Test 7: Checking pane project exists..."
PANE_DIR="$PROJECT_DIR/../pane"
if [ -d "$PANE_DIR" ]; then
    echo "  ✓ pane project exists at $PANE_DIR"
else
    echo "  ❌ FAIL: pane project not found at $PANE_DIR"
    exit 1
fi

if [ -f "$PANE_DIR/Cargo.toml" ]; then
    echo "  ✓ pane project has Cargo.toml"
else
    echo "  ❌ FAIL: pane project missing Cargo.toml"
    exit 1
fi

# Test 8: Verify README doesn't recommend cargo install for wrangler
echo ""
echo "Test 8: Checking README doesn't recommend cargo install wrangler..."
if grep -q 'cargo install wrangler' "$PROJECT_DIR/README.md"; then
    echo "  ❌ FAIL: README mentions 'cargo install wrangler' which is no longer supported"
    exit 1
else
    echo "  ✓ README does not mention deprecated cargo install wrangler"
fi

echo ""
echo "=== All tests passed! ✓ ==="
