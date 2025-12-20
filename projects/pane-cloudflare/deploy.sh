#!/usr/bin/env bash
# Deployment script for pane to Cloudflare Pages

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PANE_DIR="$PROJECT_DIR/../pane"
DIST_DIR="$PANE_DIR/dist"

echo "=== Pane Cloudflare Pages Deployment ==="
echo ""

# Check if wrangler is installed
if ! command -v wrangler &> /dev/null; then
    echo "Error: wrangler is not installed"
    echo "Install it with: npm install -g wrangler"
    echo "Or with cargo: cargo install wrangler"
    exit 1
fi

# Check if dx is installed
if ! command -v dx &> /dev/null; then
    echo "Warning: dioxus-cli (dx) is not installed"
    echo "You may need to build pane manually"
fi

# Check if dist directory exists
if [ ! -d "$DIST_DIR" ]; then
    echo "Build artifacts not found at $DIST_DIR"
    echo ""
    echo "Building pane project..."
    cd "$PANE_DIR"
    
    if command -v dx &> /dev/null; then
        dx build --release
    else
        echo "Error: Cannot build pane without dioxus-cli"
        echo "Install it with: cargo install dioxus-cli"
        exit 1
    fi
else
    echo "Using existing build artifacts at $DIST_DIR"
fi

# Deploy to Cloudflare Pages
echo ""
echo "Deploying to Cloudflare Pages..."
cd "$PROJECT_DIR"

wrangler pages deploy "$DIST_DIR" --project-name=pane

echo ""
echo "=== Deployment Complete ==="
