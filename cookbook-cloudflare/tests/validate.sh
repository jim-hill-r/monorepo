#!/usr/bin/env bash
# Validation tests for cookbook-cloudflare project

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COOKBOOK_WEB_DIR="$PROJECT_DIR/../cookbook/web"

echo "=== Validating cookbook-cloudflare project ==="
echo ""

# Test 1: Check required files exist
echo "Checking required files..."
required_files=(
    "$PROJECT_DIR/Cargo.toml"
    "$PROJECT_DIR/Cast.toml"
    "$PROJECT_DIR/wrangler.toml"
    "$PROJECT_DIR/deploy.sh"
    "$PROJECT_DIR/README.md"
    "$PROJECT_DIR/ISSUES.md"
)

for file in "${required_files[@]}"; do
    if [ ! -f "$file" ]; then
        echo "Error: Required file not found: $file"
        exit 1
    fi
    echo "  ✓ $(basename "$file")"
done

# Test 2: Verify deploy.sh is executable
echo ""
echo "Checking deploy.sh permissions..."
if [ ! -x "$PROJECT_DIR/deploy.sh" ]; then
    echo "Warning: deploy.sh is not executable"
    echo "  Making it executable..."
    chmod +x "$PROJECT_DIR/deploy.sh"
fi
echo "  ✓ deploy.sh is executable"

# Test 3: Check if cookbook/web project exists
echo ""
echo "Checking cookbook/web project..."
if [ ! -d "$COOKBOOK_WEB_DIR" ]; then
    echo "Error: cookbook/web project not found at $COOKBOOK_WEB_DIR"
    exit 1
fi
echo "  ✓ cookbook/web project exists"

# Test 4: Verify wrangler.toml syntax
echo ""
echo "Validating wrangler.toml syntax..."
if ! grep -q "name = \"cookbook\"" "$PROJECT_DIR/wrangler.toml"; then
    echo "Error: wrangler.toml missing project name"
    exit 1
fi
echo "  ✓ wrangler.toml has valid syntax"

# Test 5: Verify Cast.toml configuration
echo ""
echo "Validating Cast.toml..."
if ! grep -q 'framework = "cloudflare-pages"' "$PROJECT_DIR/Cast.toml"; then
    echo "Error: Cast.toml missing or incorrect framework configuration"
    exit 1
fi
echo "  ✓ Cast.toml has valid configuration"

echo ""
echo "=== All validation tests passed ==="
