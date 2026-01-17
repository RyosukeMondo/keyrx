# Requirements: Remaining Code Quality Fixes

## Overview
Address final code quality violations to achieve full compliance: refactor 3 remaining files exceeding 500-line limit (KeyConfigModal: 641 lines, KeyConfigPanel: 634 lines, MetricsPage: 532 lines).

## User Stories

### 1. Refactor KeyConfigModal Component
**EARS**: WHEN editing key mappings in modal, THEN configuration logic is in modular components, SO THAT modal orchestration is simplified.

**Acceptance**:
- KeyConfigModal.tsx reduced from 641 to ≤500 lines
- Mapping type selector extracted to reusable component
- Key selection UI extracted to component
- All functions ≤50 lines

### 2. Refactor KeyConfigPanel Component
**EARS**: WHEN editing key mappings inline, THEN configuration logic is in modular components, SO THAT panel code is maintainable.

**Acceptance**:
- KeyConfigPanel.tsx reduced from 634 to ≤500 lines
- Shared components with KeyConfigModal (mapping type selector, key selection)
- All functions ≤50 lines

### 3. Refactor MetricsPage Component
**EARS**: WHEN viewing metrics dashboard, THEN each metric section is a component, SO THAT page is composable.

**Acceptance**:
- MetricsPage.tsx reduced from 532 to ≤500 lines
- Metrics cards extracted to components
- Latency chart extracted to component
- Event log list extracted to component
- All functions ≤50 lines

## Technical Requirements

### TR-1: Code Quality Compliance
- All files ≤500 lines (excluding comments/blanks)
- All functions ≤50 lines
- ESLint passes with 0 errors
- Prettier formatting applied

### TR-2: Test Coverage
- Each extracted component has unit tests
- Minimum 80% line/branch coverage
- All existing tests pass

### TR-3: Backward Compatibility
- User-facing behavior unchanged
- Existing tests pass
- No breaking API changes

### TR-4: Code Reuse
- Shared components between KeyConfigModal and KeyConfigPanel
- DRY principle for mapping type selection
- Consistent patterns across codebase

## Success Metrics
- KeyConfigModal.tsx: ≤500 lines (from 641, reduce by 141+)
- KeyConfigPanel.tsx: ≤500 lines (from 634, reduce by 134+)
- MetricsPage.tsx: ≤500 lines (from 532, reduce by 32+)
- 8-10 new focused components created
- All code quality gates pass
- Test coverage >80%
