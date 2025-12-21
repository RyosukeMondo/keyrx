#!/bin/bash
# API Contract Testing Script
# Run this before building to catch API mismatches early

set -e

echo "================================"
echo "Running API Contract Tests"
echo "================================"
echo ""

cd "$(dirname "$0")/.."

# Run core_logic API contract tests
echo "Testing core_logic API contracts..."
cd packages/core_logic
flutter test test/services/api_contract_test.dart --reporter=expanded
cd ../..

echo ""
echo "================================"
echo "✅ All API contract tests passed!"
echo "================================"
echo ""
echo "These tests verify:"
echo "  - Backend API response formats match frontend expectations"
echo "  - Web services can parse all backend responses correctly"
echo "  - Error responses are handled properly"
echo ""
echo "Run this script before every build to catch mismatches in build phase, not UAT!"
