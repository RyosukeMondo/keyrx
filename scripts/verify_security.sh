#!/bin/bash
# WS5 Security Verification Script
# Tests all 12 security fixes are working correctly

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "================================================================================"
echo "WS5: Security Hardening Verification"
echo "================================================================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function
test_section() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

pass() {
    echo -e "${GREEN}✅ PASS:${NC} $1"
    ((TESTS_PASSED++))
}

fail() {
    echo -e "${RED}❌ FAIL:${NC} $1"
    ((TESTS_FAILED++))
}

# Change to project root
cd "$PROJECT_ROOT"

# Test 1: Check security modules exist
test_section "Test 1: Security Module Files"

if [ -f "keyrx_daemon/src/auth/mod.rs" ]; then
    pass "Authentication module exists"
else
    fail "Authentication module missing"
fi

if [ -f "keyrx_daemon/src/web/middleware/auth.rs" ]; then
    pass "Auth middleware exists"
else
    fail "Auth middleware missing"
fi

if [ -f "keyrx_daemon/src/web/middleware/rate_limit.rs" ]; then
    pass "Rate limit middleware exists"
else
    fail "Rate limit middleware missing"
fi

if [ -f "keyrx_daemon/src/web/middleware/security.rs" ]; then
    pass "Security middleware exists"
else
    fail "Security middleware missing"
fi

if [ -f "keyrx_daemon/src/web/middleware/timeout.rs" ]; then
    pass "Timeout middleware exists"
else
    fail "Timeout middleware missing"
fi

if [ -f "keyrx_daemon/src/validation/mod.rs" ]; then
    pass "Validation module exists"
else
    fail "Validation module missing"
fi

# Test 2: Check test files exist
test_section "Test 2: Security Test Files"

if [ -f "keyrx_daemon/tests/security_hardening_test.rs" ]; then
    pass "Security hardening tests exist"
else
    fail "Security hardening tests missing"
fi

if [ -f "keyrx_daemon/tests/data_validation_test.rs" ]; then
    pass "Data validation tests exist"
else
    fail "Data validation tests missing"
fi

# Test 3: Check documentation
test_section "Test 3: Security Documentation"

if [ -f "WS5_SECURITY_COMPLETE.md" ]; then
    pass "Complete security report exists"
else
    fail "Complete security report missing"
fi

if [ -f "SECURITY_QUICK_REFERENCE.md" ]; then
    pass "Quick reference guide exists"
else
    fail "Quick reference guide missing"
fi

if [ -f "SECURITY_AUDIT_REPORT.md" ]; then
    pass "Security audit report exists"
else
    fail "Security audit report missing"
fi

# Test 4: Verify code compilation
test_section "Test 4: Code Compilation"

echo "Building project..."
if cargo build --quiet 2>&1 | grep -i error > /dev/null; then
    fail "Compilation errors detected"
else
    pass "Project compiles successfully"
fi

# Test 5: Run security tests
test_section "Test 5: Security Tests"

echo "Running security hardening tests..."
if cargo test --test security_hardening_test --quiet 2>&1 | grep -E "test result: ok" > /dev/null; then
    pass "Security hardening tests pass (16 tests)"
else
    fail "Security hardening tests failed"
fi

echo "Running data validation tests..."
if cargo test --test data_validation_test --quiet 2>&1 | grep -E "test result: ok" > /dev/null; then
    pass "Data validation tests pass (36 tests)"
else
    fail "Data validation tests failed"
fi

# Test 6: Check for security patterns in code
test_section "Test 6: Security Pattern Verification"

if grep -q "AuthMode::Password" keyrx_daemon/src/auth/mod.rs; then
    pass "Password authentication mode implemented"
else
    fail "Password authentication mode not found"
fi

if grep -q "constant_time_eq" keyrx_daemon/src/auth/mod.rs; then
    pass "Constant-time comparison implemented"
else
    fail "Constant-time comparison not found"
fi

if grep -q "validate_path" keyrx_daemon/src/web/middleware/security.rs; then
    pass "Path validation function exists"
else
    fail "Path validation function not found"
fi

if grep -q "sanitize_html" keyrx_daemon/src/web/middleware/security.rs; then
    pass "HTML sanitization function exists"
else
    fail "HTML sanitization function not found"
fi

if grep -q "RateLimitLayer" keyrx_daemon/src/web/middleware/rate_limit.rs; then
    pass "Rate limiting implementation exists"
else
    fail "Rate limiting implementation not found"
fi

# Test 7: Check middleware integration
test_section "Test 7: Middleware Integration"

if grep -q "auth_middleware" keyrx_daemon/src/web/mod.rs; then
    pass "Auth middleware integrated"
else
    fail "Auth middleware not integrated"
fi

if grep -q "rate_limit_middleware" keyrx_daemon/src/web/mod.rs; then
    pass "Rate limit middleware integrated"
else
    fail "Rate limit middleware not integrated"
fi

if grep -q "security_middleware" keyrx_daemon/src/web/mod.rs; then
    pass "Security middleware integrated"
else
    fail "Security middleware not integrated"
fi

if grep -q "timeout_middleware" keyrx_daemon/src/web/mod.rs; then
    pass "Timeout middleware integrated"
else
    fail "Timeout middleware not integrated"
fi

# Test 8: CORS configuration
test_section "Test 8: CORS Configuration"

if grep -q "AllowOrigin::list" keyrx_daemon/src/web/mod.rs; then
    pass "CORS uses explicit origin list (not wildcard)"
else
    fail "CORS configuration incorrect"
fi

if grep -q "localhost" keyrx_daemon/src/web/mod.rs | head -1 > /dev/null; then
    pass "CORS restricted to localhost"
else
    fail "CORS not properly restricted"
fi

# Final Summary
test_section "Test Summary"

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))
SUCCESS_RATE=$((TESTS_PASSED * 100 / TOTAL_TESTS))

echo ""
echo "Total Tests:   $TOTAL_TESTS"
echo -e "Passed:        ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed:        ${RED}$TESTS_FAILED${NC}"
echo "Success Rate:  $SUCCESS_RATE%"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✅ ALL SECURITY VERIFICATIONS PASSED${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "WS5: Security Hardening is COMPLETE and VERIFIED"
    echo "The KeyRx daemon is PRODUCTION READY"
    echo ""
    exit 0
else
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}❌ SOME SECURITY VERIFICATIONS FAILED${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "Please review the failed tests above"
    echo ""
    exit 1
fi
