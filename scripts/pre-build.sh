#!/bin/bash
# Pre-build script - Run before Flutter build
# Catches issues during build/test phase instead of UAT

set -e

echo "🔍 Running pre-build checks..."
echo ""

# 1. API Contract Tests
./scripts/test-api-contracts.sh

# 2. Unit Tests (optional - can be enabled)
# echo "Running unit tests..."
# flutter test

# 3. Static Analysis
echo "Running static analysis..."
flutter analyze --no-fatal-infos

echo ""
echo "✅ All pre-build checks passed!"
echo "Ready to build."
