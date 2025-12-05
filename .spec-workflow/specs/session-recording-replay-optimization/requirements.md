# Requirements Document

## Introduction

Implement a low-overhead session recording and replay optimization pipeline for KeyRx so engineers can capture real-world input traces, store them efficiently, and deterministically replay them to diagnose latency regressions and logic bugs. The feature must minimize recording impact on live typing while producing artifacts that faithfully reproduce engine decisions.

## Alignment with Product Vision

This capability reinforces the CLI-first, performance-obsessed principles in the product vision. Deterministic capture/replay strengthens trust (safety-first), accelerates debugging for power users and developers, and supports observability goals outlined in the steering docs without compromising the sub-1ms latency target.

## Requirements

### Requirement 1 — Deterministic Session Capture

**User Story:** As a developer diagnosing flaky remaps, I want to record sessions with full timing and device metadata at <0.2 ms per event overhead, so that I can reproduce issues without disturbing live typing performance.

#### Acceptance Criteria

1. WHEN session recording is enabled via CLI flag `--record <path>` THEN the system SHALL capture every input event with monotonic sequence number, microsecond timestamp, device identity, scan code, and current engine version hash.
2. IF recording buffer pressure exceeds disk throughput THEN the system SHALL apply backpressure by dropping oldest buffered sessions only after emitting a WARN log and keeping active session integrity.
3. WHEN recording is enabled in release mode THEN the system SHALL cap retention by rolling files once they exceed 250 MB and prune the oldest files to honor a configurable size budget.

### Requirement 2 — Replay Fidelity and Performance

**User Story:** As a QA engineer, I want to replay recorded sessions deterministically with verification hooks, so that I can confirm engine outputs match the original recording and detect regressions quickly.

#### Acceptance Criteria

1. WHEN replay runs with `keyrx replay <file>` THEN the system SHALL consume recorded events in order, reproducing decisions and emitted outputs within ±100 µs timing variance relative to recorded timestamps.
2. IF replay output diverges from recorded outputs (missing, extra, or reordered actions) THEN the system SHALL emit a structured diff (JSON) and exit with code 2.
3. WHEN replay is executed with `--fast` mode THEN the system SHALL ignore real-time delays and execute the full trace at ≥10x recorded speed while preserving event order and decision outcomes.

### Requirement 3 — Storage Efficiency and Indexing

**User Story:** As an observability engineer, I want recordings to be compressed and chunked with searchable indexes, so that large traces can be stored and queried without loading entire files.

#### Acceptance Criteria

1. WHEN writing recording files THEN the system SHALL chunk data into ≤10 MB blocks, compress each block (e.g., zstd level 3), and include a table of contents with byte offsets for quick seeking.
2. IF a user queries metadata with `keyrx replay --inspect <file>` THEN the system SHALL return session duration, event count, device IDs, engine version hash, and chunk table without reading full payloads.
3. WHEN replaying a segment with `--from <ms> --to <ms>` THEN the system SHALL seek using the chunk index and load only the relevant blocks before streaming events.

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Capture, compression/indexing, and replay logic must live in separate modules with clear interfaces.
- **Modular Design**: Recording format definition must be shareable between Rust core and any tooling without duplicating schemas.
- **Dependency Management**: Compression and serialization crates must be behind feature flags to keep minimal core builds lean.
- **Clear Interfaces**: Define explicit structs for recording headers, event envelopes, and chunk metadata with versioning support.

### Performance
- Recording overhead ≤0.2 ms per event and ≤5% CPU on a 4-core laptop during sustained capture.
- Replay must stream at ≥10x recorded pace in fast mode without exceeding 512 MB RSS.
- Compression should achieve at least 2:1 ratio on typical traces (mixed tap/hold, combos).

### Security
- Recording files must omit personally identifiable content; only input metadata and engine outputs are stored.
- File integrity verified via per-file checksum; corrupted files cause replay to abort with explicit error.
- Respect user filesystem permissions—no privileged writes.

### Reliability
- Deterministic replays produce identical decision sequences for identical inputs across platforms.
- Partial/corrupted chunk detection results in graceful failure with actionable error messaging.
- Backpressure policies tested with synthetic load to ensure no engine stalls under overflow scenarios.

### Usability
- Single CLI commands for record, inspect, and replay with helpful examples in `keyrx --help`.
- Clear progress output for long replays (events processed, elapsed, ETA) with optional `--quiet`.
- Inspect command outputs machine-readable JSON for integration with automation and dashboards.
