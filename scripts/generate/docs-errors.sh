#!/bin/bash
# Script to generate error documentation
# Can be called directly or via git hooks

set -e

cd "$(dirname "$0")/.."

echo "Generating error documentation..."
cd core && cargo run --bin generate_error_docs

echo "Error documentation updated successfully!"
