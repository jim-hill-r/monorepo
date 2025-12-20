#!/bin/bash
# Build script for agent-copilot Linux x86_64 binary
# This script builds the agent-copilot binary for Linux and copies it to the artifacts directory

set -e

echo "Building agent-copilot for Linux x86_64..."
cargo build --release --target x86_64-unknown-linux-gnu

echo "Copying binary to artifacts directory..."
mkdir -p artifacts/x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/agent-copilot artifacts/x86_64-unknown-linux-gnu/

echo "Verifying binary..."
file artifacts/x86_64-unknown-linux-gnu/agent-copilot
ls -lh artifacts/x86_64-unknown-linux-gnu/agent-copilot

echo "Build complete! Binary saved to artifacts/x86_64-unknown-linux-gnu/agent-copilot"
