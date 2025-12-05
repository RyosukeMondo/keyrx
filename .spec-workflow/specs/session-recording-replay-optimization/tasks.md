# Tasks Document

## Phase 1: Format

- [x] 1. Design optimized file format
  - File: `core/src/engine/recording/format.rs`
  - Header with index, compressed blocks
  - Version for future compatibility
  - Purpose: File format spec
  - _Requirements: 1.1, 3.1_

- [ ] 2. Implement compression
  - File: `core/src/engine/recording/compression.rs`
  - Gzip block compression
  - Streaming decompression
  - Purpose: Size reduction
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

## Phase 2: Recording

- [ ] 3. Update SessionRecorder
  - File: `core/src/engine/recording/recorder.rs`
  - Block-based recording
  - Index generation
  - Purpose: Optimized recording
  - _Requirements: 1.1, 3.1_

## Phase 3: Replay

- [ ] 4. Implement streaming replay
  - File: `core/src/engine/replay.rs`
  - Streaming decompression
  - Block-based iteration
  - Purpose: Memory-efficient replay
  - _Requirements: 2.1, 2.2, 2.4_
