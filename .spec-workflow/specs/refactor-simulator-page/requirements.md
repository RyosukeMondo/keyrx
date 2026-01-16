# Requirements: Refactor SimulatorPage Component

## Overview
Break down the monolithic SimulatorPage component (712 code lines) into smaller, focused components following Single Responsibility Principle. The current component violates code quality standards (max 500 lines per file, max 50 lines per function) by combining event simulation, display, filtering, and controls.

## User Stories

### 1. As a developer, I want separate components for simulation controls
**EARS Format**: WHEN using simulator controls, THEN I see them in a dedicated component, SO THAT control logic is isolated.

**Acceptance Criteria**:
- Simulation controls extracted to `SimulationControls` component
- Start/stop, clear, injection controls grouped logically
- Component is ≤200 lines of code

### 2. As a developer, I want event display logic separated
**EARS Format**: WHEN viewing keyboard events, THEN the event list is a dedicated component, SO THAT display logic is reusable.

**Acceptance Criteria**:
- Event list extracted to `EventList` component
- Filtering and sorting logic encapsulated
- Virtualization for performance with large event lists

### 3. As a developer, I want event injection UI separated
**EARS Format**: WHEN injecting test events, THEN injection UI is in a dedicated component, SO THAT injection logic is isolated.

**Acceptance Criteria**:
- Event injection form extracted to `EventInjectionForm` component
- Key selection and event type selection encapsulated
- Form validation logic separated

### 4. As a developer, I want simulation state managed by custom hook
**EARS Format**: WHEN managing simulation state, THEN I use a dedicated hook, SO THAT state logic is testable and reusable.

**Acceptance Criteria**:
- Simulation state extracted to `useSimulation` hook
- Events array, running state, filters managed
- Hook has >80% test coverage

## Technical Requirements

### TR-1: Code Quality Compliance
- All files ≤500 lines (excluding comments/blanks)
- All functions ≤50 lines
- ESLint passes with 0 errors
- Prettier formatting applied

### TR-2: Test Coverage
- Each extracted component has unit tests
- Custom hook has unit tests
- Minimum 80% line/branch coverage

### TR-3: Backward Compatibility
- User-facing behavior identical
- Existing tests pass

### TR-4: Performance
- Event list virtualization for 1000+ events
- No performance degradation

## Success Metrics
- SimulatorPage.tsx reduced from ~712 to ≤300 code lines
- 3-4 new focused components created
- All code quality gates pass
- Test coverage maintained or improved
