# Requirements: Fix ESLint Errors

## Overview
Fix 686 ESLint errors and 68 warnings in keyrx_ui codebase to achieve 0 errors compliance.

## User Stories

### 1. Replace `any` types with proper TypeScript types
**EARS**: WHEN reviewing code, THEN all `any` types are replaced with proper types, SO THAT type safety is enforced.

**Acceptance**: 638 instances of `any` replaced with proper types (unions, unknowns with guards, or specific interfaces)

### 2. Remove console statements from production code
**EARS**: WHEN building for production, THEN no console statements exist in non-dev code, SO THAT bundles are clean.

**Acceptance**: Console statements removed or wrapped in `if (__DEV__)` guards

### 3. Fix unused variables and imports
**EARS**: WHEN compiling, THEN no unused variables or imports exist, SO THAT code is clean.

**Acceptance**: All unused vars/imports removed or prefixed with `_` if intentionally unused

## Technical Requirements
- ESLint: 0 errors, 0 warnings
- No `eslint-disable` comments added
- TypeScript strict mode maintained
- All tests pass after fixes

## Success Metrics
- ESLint passes: `npm run lint` → 0 errors/warnings
- TypeScript compiles: `tsc --noEmit` → success
- Tests pass: `npm test` → all green
