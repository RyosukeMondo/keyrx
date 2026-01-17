# ESLint Error Analysis Report

Generated: 2026-01-17

## Executive Summary

- **Total Files Analyzed:** 331
- **Files with Errors:** 138 (41.7%)
- **Total Errors:** 1,160
- **Total Warnings:** 67

## Top 15 Error Types

| Rule | Count | Category |
|------|-------|----------|
| `no-console` | 359 | Code Quality |
| `@typescript-eslint/no-unused-vars` | 217 | Code Quality |
| `@typescript-eslint/no-explicit-any` | 140 | Type Safety |
| `no-useless-escape` | 104 | Code Quality |
| `no-prototype-builtins` | 79 | Best Practices |
| `@typescript-eslint/no-this-alias` | 69 | Best Practices |
| `local-rules/test-naming-convention` | 58 | Testing |
| `react-hooks/rules-of-hooks` | 53 | React |
| `no-cond-assign` | 26 | Code Quality |
| `no-empty` | 26 | Code Quality |
| `no-fallthrough` | 20 | Code Quality |
| `no-case-declarations` | 13 | Best Practices |
| `react-hooks/exhaustive-deps` | 9 | React |
| `no-constant-condition` | 6 | Code Quality |
| `react-hooks/set-state-in-effect` | 6 | React |

## Top 10 Files with Most Errors (Excluding Build Artifacts)

### Source Files (src/)

| File | Errors | Warnings |
|------|--------|----------|
| `src/test/mocks/websocketHandlers.ts` | 20 | 1 |
| `src/components/DeviceListCard.test.tsx` | 18 | 0 |
| `src/hooks/useWasm.ts` | 13 | 0 |
| `src/api/websocket.test.ts` | 12 | 0 |
| `src/api/client.test.ts` | 10 | 0 |
| `src/hooks/useUnifiedApi.ts` | 9 | 2 |
| `src/api/profiles.ts` | 7 | 0 |
| `src/contexts/WasmContext.tsx` | 7 | 0 |
| `src/hooks/useProfileSelection.test.ts` | 7 | 0 |
| `src/components/KeyPalette.tsx` | 6 | 0 |

### Non-Source Files (Build/Test Artifacts)

| File | Errors | Warnings |
|------|--------|----------|
| `playwright-report/trace/assets/codeMirrorModule-55327053.js` | 217 | 0 |
| `playwright-report/trace/assets/wsPort-1c3b5f20.js` | 177 | 0 |
| `playwright-report/trace/sw.bundle.js` | 66 | 0 |
| `scripts/analyze-bundle.js` | 31 | 0 |
| `tests/performance/bundle-analysis.spec.ts` | 31 | 1 |

## Error Distribution by Category

### Type Safety Issues (140 errors)
- `@typescript-eslint/no-explicit-any`: 140 instances
- **Impact:** High - Reduces type safety benefits
- **Priority:** High
- **Effort:** Medium to High (requires proper type definitions)

### Code Quality Issues (732 errors)
- `no-console`: 359 instances
- `@typescript-eslint/no-unused-vars`: 217 instances
- `no-useless-escape`: 104 instances
- `no-empty`: 26 instances
- `no-cond-assign`: 26 instances
- **Impact:** Medium - Code maintainability and debugging
- **Priority:** Medium to High
- **Effort:** Low to Medium

### React/Hooks Issues (68 errors)
- `react-hooks/rules-of-hooks`: 53 instances
- `react-hooks/exhaustive-deps`: 9 instances
- `react-hooks/set-state-in-effect`: 6 instances
- **Impact:** High - Can cause runtime bugs
- **Priority:** High
- **Effort:** Medium

### Best Practices Issues (161 errors)
- `no-prototype-builtins`: 79 instances
- `@typescript-eslint/no-this-alias`: 69 instances
- `no-case-declarations`: 13 instances
- **Impact:** Low to Medium - Modern JS practices
- **Priority:** Medium
- **Effort:** Low

### Testing Issues (58 errors)
- `local-rules/test-naming-convention`: 58 instances
- **Impact:** Low - Consistency only
- **Priority:** Low
- **Effort:** Low

## Prioritized Fixing Plan

### Phase 1: Critical Issues (Week 1)
**Goal:** Fix type safety and React issues that can cause runtime bugs

1. **Fix `@typescript-eslint/no-explicit-any` (140 errors)**
   - Start with API/types files (src/api/, src/types/)
   - Move to components (src/components/)
   - Finish with hooks/utils
   - **Files to prioritize:**
     - `src/hooks/useWasm.ts` (13 errors)
     - `src/hooks/useUnifiedApi.ts` (9 errors)
     - `src/api/profiles.ts` (7 errors)

2. **Fix React Hooks violations (68 errors)**
   - `react-hooks/rules-of-hooks`: 53 instances
   - `react-hooks/exhaustive-deps`: 9 instances
   - **Files to prioritize:**
     - Review test files that violate rules-of-hooks
     - Fix dependencies in production components

### Phase 2: Code Cleanliness (Week 2)
**Goal:** Remove debugging artifacts and unused code

3. **Remove/Guard console statements (359 errors)**
   - Remove debug console.log statements
   - Wrap dev-only logs in `if (import.meta.env.DEV)`
   - Keep error/warn in error handlers
   - **Impact:** Reduces production bundle size and noise

4. **Fix unused variables/imports (217 errors)**
   - Remove genuinely unused code
   - Prefix intentionally unused with `_`
   - **Benefit:** Cleaner code, smaller bundle

### Phase 3: Best Practices (Week 3)
**Goal:** Modernize codebase to follow current standards

5. **Fix remaining errors by type:**
   - `no-useless-escape`: 104 instances (quick fixes)
   - `no-prototype-builtins`: 79 instances (use `Object.hasOwn` or null checks)
   - `@typescript-eslint/no-this-alias`: 69 instances (use arrow functions)
   - `local-rules/test-naming-convention`: 58 instances (rename tests)
   - Other minor issues: 59 instances

6. **Exclude build artifacts from linting**
   - Add `coverage/`, `playwright-report/`, `dist/` to `.eslintignore`
   - **Benefit:** Faster linting, cleaner reports

### Phase 4: Verification
**Goal:** Ensure all fixes are correct and stable

7. **Run full test suite**
   - Verify all tests pass
   - Check TypeScript compilation
   - Verify no new errors introduced

## Recommendations

### Immediate Actions
1. Update `.eslintignore` to exclude build artifacts (saves ~460 errors from reports)
2. Start with type safety fixes in core API/types files
3. Set up pre-commit hooks to prevent new `any` types

### Process Improvements
1. Enable ESLint in CI/CD to fail on new errors
2. Add ESLint check to pre-commit hooks
3. Consider gradual strictness increase:
   - Phase 1: Fix existing errors
   - Phase 2: Enforce on new code (eslint-changed-files)
   - Phase 3: Full enforcement

### Long-term
1. Consider TypeScript strict mode after `any` types are fixed
2. Add custom ESLint rules for project-specific patterns
3. Regular ESLint report reviews (monthly)

## Files to Prioritize (Production Code Only)

1. `src/hooks/useWasm.ts` (13 errors) - Core functionality
2. `src/hooks/useUnifiedApi.ts` (9 errors) - API layer
3. `src/api/profiles.ts` (7 errors) - Data layer
4. `src/contexts/WasmContext.tsx` (7 errors) - State management
5. `src/components/KeyPalette.tsx` (6 errors) - Main UI component
6. `src/api/rpc.ts` (5 errors) - Communication layer
7. `src/components/KeyConfigModal.tsx` (5 errors) - Configuration UI

## Success Criteria

- [ ] Zero ESLint errors in production code (src/)
- [ ] Zero `@typescript-eslint/no-explicit-any` violations
- [ ] Zero `react-hooks/*` violations
- [ ] Console statements removed or properly guarded
- [ ] All tests pass after fixes
- [ ] TypeScript compilation successful
- [ ] ESLint runtime < 10 seconds (after excluding artifacts)

## Notes

- Many errors (460+) are in build artifacts that should be excluded from linting
- Test files have specific patterns that may need different ESLint configs
- Some `any` types may require Zod schemas for runtime validation
- React hooks violations in tests may indicate test structure issues
