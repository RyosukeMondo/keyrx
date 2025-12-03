# Tasks Document

## Phase 1: Stack Sampling

- [x] 1. Create Profiler struct
  - File: `core/src/profiling/profiler.rs`
  - Configuration
  - Lifecycle management
  - Purpose: Core profiling
  - _Requirements: 1.1_

- [x] 2. Implement stack sampler
  - File: `core/src/profiling/sampler.rs`
  - Configurable rate
  - Low overhead collection
  - Purpose: Stack collection
  - _Requirements: 1.1_

## Phase 2: Flame Graph

- [x] 3. Create FlameGraphGenerator
  - File: `core/src/profiling/flamegraph.rs`
  - SVG generation
  - Color schemes
  - Purpose: Visualization
  - _Requirements: 1.2, 1.3_

- [x] 4. Add diff flame graphs
  - File: `core/src/profiling/flamegraph_diff.rs`
  - Baseline comparison
  - Regression highlighting
  - Purpose: A/B comparison
  - _Requirements: 1.4_

## Phase 3: Allocation Tracking

- [ ] 5. Implement allocation tracker
  - File: `core/src/profiling/allocations.rs`
  - Global allocator wrapper
  - Site tracking
  - Purpose: Memory profiling
  - _Requirements: 2.1, 2.2_

- [ ] 6. Create allocation report
  - File: `core/src/profiling/alloc_report.rs`
  - Hot spot detection
  - JSON output
  - Purpose: Memory analysis
  - _Requirements: 2.3, 2.4_

## Phase 4: Integration

- [ ] 7. Extend bench command
  - File: `core/src/cli/commands/bench.rs`
  - --flamegraph flag
  - --allocations flag
  - Purpose: CLI access
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 8. Add UI visualization
  - File: `ui/lib/pages/developer/profiler_page.dart`
  - Flame graph viewer
  - Allocation report
  - Purpose: Visual interface
  - _Requirements: 3.4_
