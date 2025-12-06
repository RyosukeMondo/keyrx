# Tasks Document

_Status: Priority #4 in 2025 implementation order; all items pending. Establish OTEL tracing/metrics foundation before downstream dashboards._

## Phase 1: Setup

- [x] 1. Add OTEL dependencies
  - File: `core/Cargo.toml`
  - Add opentelemetry, tracing-opentelemetry
  - Feature-gated
  - Purpose: Dependency setup
  - _Requirements: 3.4_

- [x] 2. Create OtelConfig
  - File: `core/src/observability/otel/config.rs`
  - Environment-based configuration
  - Defaults and validation
  - Purpose: Configuration
  - _Requirements: 3.1, 3.2_

## Phase 2: Tracing

- [x] 3. Implement OTEL layer
  - File: `core/src/observability/otel/layer.rs`
  - Create tracer with batching
  - Add to subscriber
  - Purpose: Trace export
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [ ] 4. Add span instrumentation
  - Files: Engine, drivers
  - Instrument key paths
  - Add relevant attributes
  - Purpose: Instrumentation
  - _Requirements: 1.1, 1.4_

## Phase 3: Metrics

- [ ] 5. Implement metrics export
  - File: `core/src/observability/otel/metrics.rs`
  - Export histograms and counters
  - Configure buckets
  - Purpose: Metrics export
  - _Requirements: 2.1, 2.2, 2.3, 2.4_
