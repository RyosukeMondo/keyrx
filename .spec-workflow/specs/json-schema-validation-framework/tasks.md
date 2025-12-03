# Tasks Document

## Phase 1: Schema Generation

- [ ] 1. Add schemars derives to config types
  - Files: `core/src/config/*.rs`
  - Add JsonSchema derive to all config types
  - Purpose: Schema generation
  - _Requirements: 1.1, 1.2_

- [ ] 2. Add schemars derives to profile types
  - Files: `core/src/discovery/types.rs`
  - Add JsonSchema derive to profile types
  - Purpose: Profile schema generation
  - _Requirements: 1.1_

- [ ] 3. Create SchemaRegistry
  - File: `core/src/validation/schema.rs`
  - Embed generated schemas
  - Provide lookup by name
  - Purpose: Schema storage
  - _Requirements: 1.3_

## Phase 2: Validation

- [ ] 4. Add validation to config loader
  - File: `core/src/config/loader.rs`
  - Validate against schema before parsing
  - Clear error messages
  - Purpose: Config validation
  - _Requirements: 2.1, 2.2_

- [ ] 5. Add validation to profile loader
  - File: `core/src/discovery/storage.rs`
  - Validate profiles on load
  - Purpose: Profile validation
  - _Requirements: 2.3_

- [ ] 6. Create migration framework
  - File: `core/src/config/migration.rs`
  - Version-based migration functions
  - Backup before migration
  - Purpose: Schema migration
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
