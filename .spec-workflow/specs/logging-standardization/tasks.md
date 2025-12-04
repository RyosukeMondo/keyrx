# Tasks Document

## Phase 1: Infrastructure

- [x] 1. Create StructuredLogger configuration
  - File: `core/src/observability/logger.rs`
  - Configure tracing-subscriber
  - Support JSON and pretty output
  - Purpose: Logging configuration
  - _Leverage: tracing-subscriber_
  - _Requirements: 1.1, 1.3, 1.4_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer configuring logging | Task: Create StructuredLogger with tracing-subscriber | Restrictions: JSON support, configurable levels | _Leverage: tracing-subscriber | Success: Structured logging configured | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 2. Create LogEntry type
  - File: `core/src/observability/entry.rs`
  - Structured log entry with fields
  - FFI-compatible representation
  - Purpose: Log data structure
  - _Leverage: serde_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating types | Task: Create LogEntry with FFI representation | Restrictions: Serializable, C-compatible FFI | _Leverage: serde | Success: Log entries structured | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 3. Create LogBridge for FFI
  - File: `core/src/observability/bridge.rs`
  - Implement as tracing Layer
  - Buffer and callback support
  - Purpose: FFI log bridge
  - _Leverage: tracing Layer_
  - _Requirements: 4.1, 4.2_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating bridge | Task: Create LogBridge as tracing Layer | Restrictions: Non-blocking, buffered | _Leverage: tracing Layer | Success: Logs bridge to FFI | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Macros and Helpers

- [ ] 4. Create logging convenience macros
  - File: `core/src/observability/macros.rs`
  - log_event!, log_error!, timed_span!
  - Purpose: Easy structured logging
  - _Leverage: tracing macros_
  - _Requirements: 1.1, 1.3_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating macros | Task: Create logging convenience macros | Restrictions: Ergonomic, consistent fields | _Leverage: tracing | Success: Easy to log structured data | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 5. Document logging standards
  - File: `docs/logging-standards.md`
  - Define when to use each level
  - Document required fields
  - Purpose: Logging guidelines
  - _Leverage: tracing documentation_
  - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Document logging standards and levels | Restrictions: Clear guidelines, examples | _Leverage: tracing docs | Success: Developers know logging standards | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Migration

- [ ] 6. Replace println in CLI
  - Files: `core/src/bin/keyrx.rs`, `core/src/cli/*.rs`
  - Replace println/eprintln with tracing
  - Keep user-facing output as println
  - Purpose: CLI logging migration
  - _Leverage: Logging macros_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating code | Task: Replace println in CLI with tracing | Restrictions: User output stays println, rest tracing | _Leverage: Logging macros | Success: CLI uses structured logging | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 7. Replace println in engine
  - Files: `core/src/engine/*.rs`
  - Replace all debug println with tracing
  - Add structured fields
  - Purpose: Engine logging migration
  - _Leverage: Logging macros_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating code | Task: Replace println in engine with tracing | Restrictions: Structured fields, appropriate levels | _Leverage: Logging macros | Success: Engine uses structured logging | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 8. Replace println in drivers
  - Files: `core/src/drivers/*.rs`
  - Replace driver debug output
  - Add platform-specific context
  - Purpose: Driver logging migration
  - _Leverage: Logging macros_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating code | Task: Replace println in drivers with tracing | Restrictions: Platform context, appropriate levels | _Leverage: Logging macros | Success: Drivers use structured logging | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 9. Replace println in FFI
  - Files: `core/src/ffi/*.rs`
  - Replace FFI debug output
  - Never log secrets
  - Purpose: FFI logging migration
  - _Leverage: Logging macros_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating code | Task: Replace println in FFI with tracing | Restrictions: No secrets, appropriate levels | _Leverage: Logging macros | Success: FFI uses structured logging | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 10. Replace remaining println calls
  - Files: All remaining Rust files
  - Audit for any remaining println
  - Purpose: Complete migration
  - _Leverage: grep, logging macros_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer completing migration | Task: Replace all remaining println with tracing | Restrictions: Complete audit, no println remaining | _Leverage: grep | Success: No debug println in codebase | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Metrics Bridge

- [ ] 11. Create MetricsBridge
  - File: `core/src/observability/metrics_bridge.rs`
  - Connect to MetricsCollector
  - FFI callback and polling
  - Purpose: Metrics to FFI
  - _Leverage: MetricsCollector_
  - _Requirements: 3.1, 3.4, 4.1_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating bridge | Task: Create MetricsBridge for FFI export | Restrictions: Callback and polling, non-blocking | _Leverage: MetricsCollector | Success: Metrics available via FFI | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 12. Create FFI observability exports
  - File: `core/src/ffi/exports_observability.rs`
  - Export log and metrics functions
  - Callback registration
  - Purpose: FFI interface
  - _Leverage: LogBridge, MetricsBridge_
  - _Requirements: 4.1, 4.2, 4.3_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating FFI | Task: Create observability FFI exports | Restrictions: Log and metrics, callbacks | _Leverage: Bridges | Success: Flutter can access observability | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Flutter Integration

- [ ] 13. Create ObservabilityService
  - File: `ui/lib/services/observability_service.dart`
  - Fetch logs and metrics from FFI
  - Stream updates
  - Purpose: Flutter observability access
  - _Leverage: FFI exports_
  - _Requirements: 4.3, 4.4_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer creating service | Task: Create ObservabilityService for logs/metrics | Restrictions: Stream updates, efficient polling | _Leverage: FFI exports | Success: Flutter has observability access | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 14. Create LogViewer widget
  - File: `ui/lib/widgets/debug/log_viewer.dart`
  - Display logs with filtering
  - Level and target filters
  - Purpose: Log visualization
  - _Leverage: ObservabilityService_
  - _Requirements: 4.3_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer creating widget | Task: Create LogViewer widget with filtering | Restrictions: Level filter, search, auto-scroll | _Leverage: ObservabilityService | Success: Logs visible in UI | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 15. Create MetricsWidget
  - File: `ui/lib/widgets/debug/metrics_widget.dart`
  - Display key metrics
  - Real-time updates
  - Purpose: Metrics visualization
  - _Leverage: ObservabilityService_
  - _Requirements: 3.3, 4.3_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer creating widget | Task: Create MetricsWidget for dashboard | Restrictions: Real-time, key metrics | _Leverage: ObservabilityService | Success: Metrics visible in UI | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 6: Testing

- [ ] 16. Add logging tests
  - File: `core/tests/unit/observability/`
  - Test log formatting
  - Test FFI bridge
  - Purpose: Verify logging
  - _Leverage: Test fixtures_
  - _Requirements: Non-functional (reliability)_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Create logging unit tests | Restrictions: Test formatting, bridge, levels | _Leverage: Test fixtures | Success: Logging verified | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 17. Add lint to prevent println
  - File: `.cargo/config.toml` or clippy.toml
  - Warn on println in non-CLI code
  - Purpose: Prevent regression
  - _Leverage: Clippy lints_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec logging-standardization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer configuring lints | Task: Add lint to warn on println | Restrictions: Warn in non-CLI, allow in CLI | _Leverage: Clippy | Success: New println causes warning | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
