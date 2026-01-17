# Requirements: Refactor ConfigPage Component

## Overview
Break down the monolithic ConfigPage component (940 code lines) into smaller, focused components following Single Responsibility Principle. The current component violates code quality standards (max 500 lines per file, max 50 lines per function) and couples multiple concerns.

## User Stories

### 1. As a developer, I want separate components for each major UI section
**EARS Format**: WHEN viewing ConfigPage, THEN I see it composed of smaller, focused sub-components, SO THAT each component has a single responsibility.

**Acceptance Criteria**:
- ConfigPage orchestrates child components but contains minimal logic
- Each sub-component is ≤500 lines of code
- No function exceeds 50 lines

### 2. As a developer, I want extracted state management hooks
**EARS Format**: WHEN working with profile state, THEN I use dedicated custom hooks, SO THAT state logic is reusable and testable.

**Acceptance Criteria**:
- Profile selection logic extracted to `useProfileSelection` hook
- Code panel state extracted to `useCodePanel` hook
- Layout state extracted to `useKeyboardLayout` hook
- Each hook has unit tests with >80% coverage

### 3. As a developer, I want keyboard visualization separated
**EARS Format**: WHEN displaying the keyboard visualizer, THEN it is in a dedicated component with its own state, SO THAT visualization logic is isolated.

**Acceptance Criteria**:
- Keyboard visualization logic moved to `KeyboardVisualizerContainer`
- Layout selection and key click handlers encapsulated
- Component independently testable

### 4. As a developer, I want configuration panel extracted
**EARS Format**: WHEN managing key mappings, THEN the configuration UI is in a dedicated component, SO THAT mapping logic is separated from visualization.

**Acceptance Criteria**:
- Configuration panel moved to `ConfigurationPanel` component
- Device selector, layer switcher, and mapping summary grouped logically
- Props interface clearly defined

### 5. As a developer, I want profile management extracted
**EARS Format**: WHEN selecting or creating profiles, THEN profile operations are handled by a dedicated component, SO THAT profile UI logic is centralized.

**Acceptance Criteria**:
- Profile selector moved to `ProfileSelector` component
- Profile creation UI moved to dedicated component
- Profile state managed through custom hook

## Technical Requirements

### TR-1: Code Quality Compliance
- All files ≤500 lines (excluding comments/blanks)
- All functions ≤50 lines
- ESLint passes with 0 errors
- Prettier formatting applied

### TR-2: Test Coverage
- Each extracted component has unit tests
- Each custom hook has unit tests
- Minimum 80% line/branch coverage
- No failing tests introduced

### TR-3: Backward Compatibility
- API contracts unchanged (props interface maintained)
- Existing tests continue to pass
- User-facing behavior identical

### TR-4: Architecture Patterns
- Follow existing component patterns in codebase
- Use dependency injection for testability
- Props drilling avoided (use context or state management)
- TypeScript strict mode compliance

## Non-Functional Requirements

### NFR-1: Performance
- No performance degradation from refactoring
- Lazy loading for code panel if possible
- Memoization where appropriate

### NFR-2: Maintainability
- Clear component hierarchy and data flow
- Each component documented with JSDoc
- README updated with new component structure

### NFR-3: Testing
- Integration tests verify component composition
- Existing E2E tests pass without modification
- New components independently testable

## Success Metrics
- ConfigPage.tsx reduced from ~940 to ≤500 code lines
- ConfigPage main component function reduced from ~892 to ≤50 lines
- 5-7 new focused components created
- All code quality gates pass
- Test coverage maintained or improved
