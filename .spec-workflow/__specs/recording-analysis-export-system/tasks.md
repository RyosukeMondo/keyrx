# Tasks Document

_Status: Priority #9 in 2025 implementation order; all items pending. Implement analysis/export/CLI after observability/profile foundations._

## Phase 1: Analysis Engine

- [ ] 1. Create AnalysisEngine
  - File: `core/src/analysis/engine.rs`
  - Statistical calculations
  - Streaming support
  - Purpose: Core analysis
  - _Requirements: 1.1, 1.2_

- [ ] 2. Implement outlier detection
  - File: `core/src/analysis/outliers.rs`
  - IQR-based detection
  - Context extraction
  - Purpose: Anomaly detection
  - _Requirements: 1.2, 1.4_

- [ ] 3. Add comparison analysis
  - File: `core/src/analysis/compare.rs`
  - Session diff
  - Regression detection
  - Purpose: A/B comparison
  - _Requirements: 1.3_

## Phase 2: Export Formats

- [ ] 4. Implement CSV exporter
  - File: `core/src/analysis/export/csv.rs`
  - Configurable columns
  - Streaming write
  - Purpose: Spreadsheet export
  - _Requirements: 2.1_

- [ ] 5. Implement JSON exporter
  - File: `core/src/analysis/export/json.rs`
  - Full event data
  - Pretty/compact modes
  - Purpose: Programmatic access
  - _Requirements: 2.2_

- [ ] 6. Implement Parquet exporter
  - File: `core/src/analysis/export/parquet.rs`
  - Columnar format
  - Compression options
  - Purpose: Data science
  - _Requirements: 2.3_

## Phase 3: Visualization

- [ ] 7. Create heatmap generator
  - File: `core/src/analysis/viz/heatmap.rs`
  - SVG output
  - Key frequency coloring
  - Purpose: Usage visualization
  - _Requirements: 3.1_

- [ ] 8. Create timeline generator
  - File: `core/src/analysis/viz/timeline.rs`
  - Latency over time
  - Event markers
  - Purpose: Temporal analysis
  - _Requirements: 3.2_

## Phase 4: CLI Integration

- [ ] 9. Add analyze CLI command
  - File: `core/src/cli/commands/analyze.rs`
  - Subcommands: stats, export, viz
  - Batch processing
  - Purpose: User access
  - _Requirements: 1.1, 2.4_
