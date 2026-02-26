#!/bin/bash

echo "Building Zed Sheets extension..."

# Build the main workspace
cargo build --workspace

echo "Build completed successfully!"
echo "Files created:"
find . -name "*.rs" -o -name "*.toml" | head -20
echo "..."
