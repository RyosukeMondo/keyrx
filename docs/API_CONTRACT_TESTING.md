# API Contract Testing - Auto-Detection Setup

## Overview

This document describes the automated API contract testing system that catches mismatches between frontend and backend during **build/test phase** instead of UAT (User Acceptance Testing).

## Problem Solved

Previously, API format mismatches were discovered during UAT:
- Backend returns `{"layouts": [...]}` but frontend expected `{"data": [...]}`
- Frontend code fails at runtime with parsing errors
- Wasted time debugging issues that should have been caught earlier

## Solution

Automated contract tests verify that:
1. Backend API response formats match frontend expectations
2. Web services can correctly parse all backend responses
3. Error responses are handled properly
4. Type mismatches are caught during compilation

## Files Created

### Test Files

1. **`/packages/core_logic/test/services/api_contract_test.dart`**
   - Verifies API response parsing for all endpoints
   - Tests success and error response formats
   - Catches malformed JSON and edge cases
   - Documents expected response format for each endpoint

### Scripts

2. **`/scripts/test-api-contracts.sh`**
   - Runs API contract tests
   - Can be executed standalone
   - Returns exit code 0 on success, 1 on failure

3. **`/scripts/pre-build.sh`**
   - Complete pre-build verification
   - Runs API contract tests
   - Runs static analysis
   - Should be run before every build

## Usage

### During Development

```bash
# Run API contract tests only
./scripts/test-api-contracts.sh

# Run full pre-build checks
./scripts/pre-build.sh
```

### Before Building

```bash
# Run pre-build checks, then build
./scripts/pre-build.sh && flutter build web --release
```

### In CI/CD Pipeline

Add to your CI/CD configuration:

```yaml
# Example GitHub Actions
- name: Run API Contract Tests
  run: ./scripts/test-api-contracts.sh

- name: Build Web App
  run: flutter build web --release
  if: success()
```

```yaml
# Example GitLab CI
test:api-contracts:
  script:
    - ./scripts/test-api-contracts.sh

build:web:
  script:
    - flutter build web --release
  needs:
    - test:api-contracts
```

## API Contract Documentation

### Backend Response Formats

| Endpoint | Method | Success Response | Error Response |
|----------|--------|------------------|----------------|
| `/api/layouts` | GET | `{"layouts": [...]}` | `{"error": "message"}` |
| `/api/layouts` | POST | `{"layout": {...}}` | `{"error": "message"}` |
| `/api/layouts/:id` | DELETE | `{"success": true}` | `{"error": "message"}` |
| `/api/keymaps` | GET | `{"keymaps": [...]}` | `{"error": "message"}` |
| `/api/keymaps` | POST | `{"keymap": {...}}` | `{"error": "message"}` |
| `/api/keymaps/:id` | DELETE | `{"success": true}` | `{"error": "message"}` |
| `/api/hardware/profiles` | GET | `{"profiles": [...]}` | `{"error": "message"}` |
| `/api/hardware/profiles` | POST | `{"profile": {...}}` | `{"error": "message"}` |
| `/api/hardware/profiles/:id` | DELETE | `{"success": true}` | `{"error": "message"}` |
| `/api/devices` | GET | `{"devices": [...]}` | `{"error": "message"}` |
| `/api/runtime/config` | GET | `{"config": {...}}` | `{"error": "message"}` |

### Frontend Parsing

**Before Fix:**
```dart
// ❌ WRONG - Expected {"data": [...]}
final data = response.data as List<dynamic>;
```

**After Fix:**
```dart
// ✅ CORRECT - Extracts from {"layouts": [...]}
final json = response.data as Map<String, dynamic>;
final data = json['layouts'] as List<dynamic>;
```

## Adding New Tests

When adding a new API endpoint:

1. **Add backend response test:**
```dart
test('correctly parses new endpoint response', () {
  final response = http.Response(
    '{"field_name": [...]}',  // Actual backend format
    200,
  );

  final apiResponse = ApiResponse.fromHttpResponse(response);

  expect(apiResponse.isSuccess, isTrue);
  final data = apiResponse.data as Map<String, dynamic>;
  expect(data.containsKey('field_name'), isTrue);
});
```

2. **Update contract documentation:**
   - Add endpoint to table above
   - Document expected request/response format

3. **Implement web service method:**
   - Extract correct field from response
   - Handle errors properly

## Test Coverage

Current coverage:
- ✅ All GET endpoints (list operations)
- ✅ All POST endpoints (create/update operations)
- ✅ All DELETE endpoints
- ✅ Error responses (400, 404, 500)
- ✅ Malformed JSON handling
- ✅ Empty response handling

## Benefits

1. **Catch errors early**: API mismatches detected during build, not UAT
2. **Prevent regressions**: Tests fail if backend changes format
3. **Living documentation**: Tests document expected API contract
4. **Faster debugging**: Clear error messages when contract breaks
5. **Confidence**: Deploy knowing APIs match

## Maintenance

### When Backend Changes

If backend API format changes:

1. Tests will **fail** - this is intentional!
2. Review the change - is it intentional or a bug?
3. If intentional:
   - Update frontend parsing code
   - Update contract tests
   - Document the change
4. If unintentional:
   - Fix backend to match contract
   - Tests should pass again

### When Frontend Changes

If frontend expectations change:

1. Update contract tests first (TDD approach)
2. Implement frontend changes
3. Coordinate with backend team if response format needs to change
4. Update documentation

## Related Files

- `/packages/core_logic/lib/web/api_client.dart` - HTTP client and response parsing
- `/packages/core_logic/lib/services/*_web.dart` - Web service implementations
- `/apps/keyrx_server/lib/api/routes.dart` - Backend API routes

## Questions?

If you find an API mismatch:
1. Check if contract tests cover it - if not, add a test
2. Run `./scripts/test-api-contracts.sh` to verify
3. Fix the mismatch and verify tests pass
4. Document the expected format

The goal: **Zero UAT bugs from API format mismatches.**
