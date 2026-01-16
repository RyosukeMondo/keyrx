# Design: Fix Code Formatting

## Approach
1. Run `npm run format` to auto-fix all files
2. Verify no functionality broken (run tests)
3. Commit formatted code

## Files to Format
- All 174 files identified by Prettier check
- Includes src/, tests/, config files

## Verification
- Run `npm run format:check` → should pass
- Run `npm test` → all tests pass
- Review git diff to ensure only formatting changes
