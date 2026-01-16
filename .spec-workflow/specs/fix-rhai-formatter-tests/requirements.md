# Requirements: Fix RhaiFormatter Tests

## Overview
Fix 17 failing tests in rhaiFormatter.test.ts related to edge case formatting expectations.

## User Stories

### 1. Fix edge case test expectations
**EARS**: WHEN formatter processes edge cases, THEN tests verify correct behavior, SO THAT edge cases are properly handled.

**Acceptance**: All 17 tests pass, formatter behavior matches expectations

## Technical Requirements
- All tests in rhaiFormatter.test.ts pass
- Formatter behavior is correct (verified manually)
- No test skipping

## Success Metrics
- `npm test rhaiFormatter` → 0 failures
- Test expectations align with formatter output
