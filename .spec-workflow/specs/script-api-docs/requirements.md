# Requirements Document

## Introduction

KeyRx uses Rhai scripting for advanced configurations, but there's zero auto-generated API documentation. Users must read Rust source code to understand available functions. This spec creates comprehensive, auto-generated Rhai API documentation with examples and type information.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **User Experience**: Users can discover and use scripting features
- **Documentation**: Self-documenting API
- **Discoverability**: IDE-like experience for script writing

Per product.md: "Powerful scripting for advanced users"

## Requirements

### Requirement 1: API Documentation Generation

**User Story:** As a script author, I want auto-generated API docs, so that I know what functions are available.

#### Acceptance Criteria

1. WHEN Rhai functions are registered THEN documentation SHALL be extractable
2. IF a function has a doc comment THEN it SHALL appear in generated docs
3. WHEN docs are generated THEN all registered functions SHALL be included
4. IF a function is deprecated THEN it SHALL be marked as such

### Requirement 2: Type Information

**User Story:** As a script author, I want type information, so that I know what parameters functions expect.

#### Acceptance Criteria

1. WHEN a function is documented THEN parameter types SHALL be shown
2. IF a function returns a value THEN the return type SHALL be shown
3. WHEN types are complex THEN they SHALL be explained
4. IF a type has methods THEN they SHALL be documented

### Requirement 3: Examples

**User Story:** As a script author, I want examples, so that I can learn by example.

#### Acceptance Criteria

1. WHEN a function is documented THEN at least one example SHALL exist
2. IF an example exists THEN it SHALL be runnable
3. WHEN examples are tested THEN they SHALL pass
4. IF an example fails THEN the build SHALL warn

### Requirement 4: Searchable Documentation

**User Story:** As a script author, I want searchable docs, so that I can find functions quickly.

#### Acceptance Criteria

1. WHEN docs are generated THEN they SHALL be in searchable format
2. IF HTML docs exist THEN search functionality SHALL work
3. WHEN searching THEN function names, parameters, and descriptions SHALL be indexed
4. IF IDE integration exists THEN autocomplete SHALL use doc data

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Source**: Docs generated from code, not manually written
- **Modular Generation**: Each module's docs generated separately
- **Testable Examples**: Examples are test cases

### Documentation Quality
- All public Rhai functions SHALL be documented
- Examples SHALL cover common use cases
- Type information SHALL be accurate
- Docs SHALL be regenerated on changes

### Integration
- Docs SHALL be viewable in browser
- Docs SHALL be exportable as markdown
- Docs SHALL be usable by IDEs (JSON schema)
