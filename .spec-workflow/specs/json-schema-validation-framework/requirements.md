# Requirements Document

## Introduction

Device profiles, configurations, and validation results are serialized to JSON with no schema enforcement. Manual parsing with downstream validation allows silent schema drift. This spec adds JSON Schema validation and generation from Rust types.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Reliability**: Catch config errors early
- **Maintainability**: Schema as documentation
- **Developer Experience**: Clear validation errors

Per tech.md: "Validate at entry, reject invalid immediately"

## Requirements

### Requirement 1: Schema Generation

**User Story:** As a developer, I want schemas from types, so that they stay in sync.

#### Acceptance Criteria

1. WHEN Rust type has JsonSchema derive THEN schema SHALL generate
2. IF schema generated THEN it SHALL match type structure
3. WHEN type changes THEN schema SHALL update
4. IF schema is invalid THEN build SHALL fail

### Requirement 2: Schema Validation

**User Story:** As a user, I want config validation, so that errors are caught early.

#### Acceptance Criteria

1. WHEN config loaded THEN it SHALL validate against schema
2. IF validation fails THEN clear error SHALL show
3. WHEN profile loaded THEN it SHALL validate
4. IF schema missing THEN fallback validation SHALL occur

### Requirement 3: Migration Support

**User Story:** As a user, I want config migration, so that upgrades don't break configs.

#### Acceptance Criteria

1. WHEN schema version changes THEN migration SHALL be possible
2. IF old config loaded THEN migration SHALL be attempted
3. WHEN migration fails THEN backup SHALL be preserved
4. IF migration succeeds THEN new version SHALL be stored

## Non-Functional Requirements

### Maintainability
- Schemas SHALL be auto-generated
- Validation errors SHALL be actionable
- Migration code SHALL be versioned
