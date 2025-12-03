# Tasks Document

## Phase 1: Metrics Collection

- [ ] 1. Create MetricsCollector
  - File: `core/src/metrics/collector.rs`
  - Counter, histogram, gauge setup
  - OTEL integration
  - Purpose: Metrics collection
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [ ] 2. Instrument engine
  - File: `core/src/engine/mod.rs`
  - Key event counters
  - Latency histograms
  - Purpose: Data collection
  - _Requirements: 1.1, 1.2_

- [ ] 3. Add error metrics
  - File: `core/src/metrics/errors.rs`
  - Error type counters
  - Error rate calculation
  - Purpose: Error tracking
  - _Requirements: 1.4_

## Phase 2: Export

- [ ] 4. Implement Prometheus exporter
  - File: `core/src/metrics/prometheus.rs`
  - /metrics endpoint
  - Label configuration
  - Purpose: Prometheus scraping
  - _Requirements: 2.1_

- [ ] 5. Add OTLP metrics export
  - File: `core/src/metrics/otlp.rs`
  - Batch export
  - Endpoint configuration
  - Purpose: Collector export
  - _Requirements: 2.2_

- [ ] 6. Create local metrics store
  - File: `core/src/metrics/local_store.rs`
  - Time-series storage
  - Query interface
  - Purpose: Local access
  - _Requirements: 2.3_

## Phase 3: Dashboard

- [ ] 7. Create Grafana dashboard JSON
  - File: `core/src/metrics/grafana.rs`
  - Panel definitions
  - Query templates
  - Purpose: Grafana import
  - _Requirements: 3.1_

- [ ] 8. Add embedded dashboard
  - File: `ui/lib/pages/metrics_dashboard.dart`
  - Real-time charts
  - Time range selection
  - Purpose: Local visualization
  - _Requirements: 3.2, 3.4_

- [ ] 9. Implement alerting
  - File: `core/src/metrics/alerts.rs`
  - Threshold configuration
  - Alert callbacks
  - Purpose: Proactive monitoring
  - _Requirements: 3.3_
