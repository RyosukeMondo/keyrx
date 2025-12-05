# Tasks Document

## Phase 1: Core Formatter

- [x] 1. Create OutputFormatter trait
  - File: `core/src/cli/output/formatter.rs`
  - Define format interface
  - Add OutputFormat enum
  - Purpose: Format abstraction
  - _Requirements: 1.1_

- [x] 2. Implement JSON formatter
  - File: `core/src/cli/output/json.rs`
  - Pretty-printed JSON output
  - Handle all types
  - Purpose: JSON format
  - _Requirements: 1.2_

- [ ] 3. Implement Table formatter
  - File: `core/src/cli/output/table.rs`
  - Aligned columns
  - Handle variable widths
  - Purpose: Table format
  - _Requirements: 1.3_

- [ ] 4. Implement YAML formatter
  - File: `core/src/cli/output/yaml.rs`
  - Valid YAML output
  - Purpose: YAML format
  - _Requirements: 1.4_

## Phase 2: Integration

- [ ] 5. Add global --output-format flag
  - File: `core/src/cli/mod.rs`
  - Parse format from args
  - Pass to all commands
  - Purpose: Consistent interface
  - _Requirements: 2.1, 2.2, 2.3, 2.4_
