# Design: Fix RhaiFormatter Tests

## Failing Tests Analysis
Tests failing: edge cases with line endings, trailing whitespace

**Issues**:
- Formatter now adds `device_start("*")` and `device_end()` wrappers
- Old tests expect unwrapped output
- Line count expectations outdated

## Fix Strategy
1. Update test expectations to match current formatter behavior
2. Verify formatter output is correct for edge cases
3. Add additional edge case tests if gaps found

## Verification
- Run tests in isolation
- Manually verify formatter output for edge cases
- Ensure no regressions
