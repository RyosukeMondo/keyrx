# Requirements Document

## Introduction

The `editor_widgets.dart` file is 1035 lines containing 15+ widget classes with mixed responsibilities. Large widget files hurt maintainability, testability, and make code navigation difficult. This spec decomposes the monolithic widget file into focused, single-purpose widget components following Flutter best practices.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Maintainability**: Small, focused widgets are easier to understand
- **Testability**: Isolated widgets can be unit tested
- **Performance**: Smaller widgets enable better rebuild optimization

Per tech.md: "Max 500 lines/file" - current file exceeds this by 2x.

## Requirements

### Requirement 1: Widget Extraction

**User Story:** As a developer, I want each widget in its own file, so that I can find and modify widgets quickly.

#### Acceptance Criteria

1. WHEN a widget has > 100 lines THEN it SHALL be in its own file
2. IF a widget has helper methods THEN they SHALL be private to that widget
3. WHEN widgets are extracted THEN imports SHALL be updated throughout codebase
4. IF a widget has state THEN StatefulWidget and State SHALL be in same file

### Requirement 2: Widget Organization

**User Story:** As a developer, I want widgets organized by feature, so that related widgets are discoverable.

#### Acceptance Criteria

1. WHEN widgets are extracted THEN they SHALL be in feature subdirectories
2. IF widgets share styling THEN a shared theme file SHALL exist
3. WHEN a widget is reusable THEN it SHALL be in `widgets/common/`
4. IF a widget is page-specific THEN it SHALL be in `widgets/{page}/`

### Requirement 3: Widget Composition

**User Story:** As a developer, I want widgets to be composable, so that I can build complex UIs from simple parts.

#### Acceptance Criteria

1. WHEN a widget has > 3 responsibilities THEN it SHALL be split
2. IF a widget builds other widgets THEN composition SHALL be used
3. WHEN widgets communicate THEN callbacks or providers SHALL be used
4. IF state is shared THEN a state management solution SHALL coordinate

### Requirement 4: Widget Testing

**User Story:** As a developer, I want widgets testable in isolation, so that I can verify widget behavior.

#### Acceptance Criteria

1. WHEN a widget is extracted THEN a test file SHALL be created
2. IF a widget has logic THEN unit tests SHALL cover it
3. WHEN widgets interact THEN integration tests SHALL verify
4. IF a widget renders differently THEN golden tests SHALL exist

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility**: One widget per file
- **Modular Design**: Feature-based directory structure
- **Dependency Management**: Minimal cross-widget dependencies
- **Clear Interfaces**: Widget APIs defined by constructors

### Performance
- Widget rebuilds SHALL be optimized with const constructors
- Large lists SHALL use ListView.builder
- Expensive computations SHALL be memoized

### Maintainability
- No file SHALL exceed 300 lines
- Widget names SHALL be descriptive
- Public widgets SHALL have documentation
