# File Size Audit Results

**Audit Date:** 2025-12-12
**Target Directory:** `core/src`
**Line Limit:** 500 lines
**Files Exceeding Limit:** 73

## Top 10 Largest Files

| Rank | Lines | File |
|------|-------|------|
| 1 | 1,893 | scripting/bindings.rs |
| 2 | 1,570 | engine/state/mod.rs |
| 3 | 1,403 | engine/transitions/log.rs |
| 4 | 1,382 | bin/keyrx.rs |
| 5 | 1,069 | scripting/docs/generators/html.rs |
| 6 | 968 | validation/engine.rs |
| 7 | 949 | config/loader.rs |
| 8 | 918 | registry/profile.rs |
| 9 | 906 | engine/advanced.rs |
| 10 | 899 | cli/commands/run.rs |

## Complete List (73 files over 500 lines)

| Lines | File |
|-------|------|
| 1,893 | scripting/bindings.rs |
| 1,570 | engine/state/mod.rs |
| 1,403 | engine/transitions/log.rs |
| 1,382 | bin/keyrx.rs |
| 1,069 | scripting/docs/generators/html.rs |
| 968 | validation/engine.rs |
| 949 | config/loader.rs |
| 918 | registry/profile.rs |
| 906 | engine/advanced.rs |
| 899 | cli/commands/run.rs |
| 888 | validation/safety.rs |
| 864 | ffi/marshal/callback.rs |
| 853 | ffi/domains/observability.rs |
| 843 | engine/transitions/graph.rs |
| 840 | validation/semantic.rs |
| 831 | profiling/flamegraph_diff.rs |
| 828 | engine/replay.rs |
| 810 | engine/state/layers.rs |
| 806 | cli/commands/hardware.rs |
| 781 | engine/state/persistence.rs |
| 755 | cli/commands/simulate.rs |
| 733 | scripting/docs/generators/json.rs |
| 724 | uat/report.rs |
| 692 | scripting/docs/search.rs |
| 685 | ffi/marshal/stream.rs |
| 671 | uat/fuzz.rs |
| 656 | metrics/profile.rs |
| 650 | drivers/windows/safety/thread_local.rs |
| 649 | metrics/snapshot.rs |
| 645 | profiling/alloc_report.rs |
| 637 | uat/coverage.rs |
| 636 | ffi/marshal/impls/session.rs |
| 631 | ffi/marshal/traits.rs |
| 626 | scripting/registry.rs |
| 626 | registry/device.rs |
| 624 | cli/commands/uat.rs |
| 623 | ffi/marshal/error.rs |
| 612 | uat/report_html.rs |
| 611 | drivers/linux/reader.rs |
| 608 | engine/coordinate_translator.rs |
| 605 | uat/report_markdown.rs |
| 602 | engine/decision/pending.rs |
| 601 | safety/circuit_breaker.rs |
| 601 | metrics/full_collector.rs |
| 598 | validation/detectors/conflicts.rs |
| 593 | discovery/session.rs |
| 589 | cli/commands/runtime.rs |
| 587 | ffi/tests/parallel_tests.rs |
| 584 | engine/state/snapshot.rs |
| 582 | ffi/domains/device_registry.rs |
| 576 | ffi/marshal/result.rs |
| 574 | scripting/sandbox/mod.rs |
| 573 | profiling/flamegraph.rs |
| 567 | observability/metrics_bridge.rs |
| 566 | registry/bindings.rs |
| 565 | scripting/docs/registry.rs |
| 562 | ffi/domains/profile_registry.rs |
| 560 | validation/coverage.rs |
| 558 | drivers/windows/input.rs |
| 552 | api.rs |
| 541 | errors/critical.rs |
| 539 | cli/commands/regression.rs |
| 536 | engine/event_loop.rs |
| 532 | drivers/windows/hook.rs |
| 530 | uat/golden_types.rs |
| 527 | drivers/common/recovery.rs |
| 525 | drivers/windows/safety/hook.rs |
| 525 | definitions/library.rs |
| 522 | metrics/collector.rs |
| 519 | uat/golden.rs |
| 508 | migration/v1_to_v2.rs |
| 507 | cli/commands/repl.rs |
| 505 | cli/commands/check.rs |

## Size Distribution

| Range | Count |
|-------|-------|
| 1,500+ lines | 2 |
| 1,000-1,499 lines | 3 |
| 800-999 lines | 8 |
| 600-799 lines | 22 |
| 500-599 lines | 38 |

## Summary Statistics

- **Total oversized files:** 73
- **Total lines in oversized files:** ~47,000
- **Largest file:** scripting/bindings.rs (1,893 lines)
- **Average lines in oversized files:** ~644 lines
- **Median lines:** ~601 lines

## Domains with Most Violations

| Domain | Count | Total Lines |
|--------|-------|-------------|
| engine/ | 12 | ~9,000 |
| cli/commands/ | 9 | ~6,100 |
| ffi/ | 9 | ~5,600 |
| validation/ | 5 | ~3,850 |
| scripting/ | 5 | ~4,950 |
| uat/ | 6 | ~3,900 |
| drivers/ | 4 | ~2,350 |
| metrics/ | 4 | ~2,430 |
| registry/ | 3 | ~2,110 |
| profiling/ | 3 | ~2,050 |

## Notes

- The task plan focuses on splitting the top 10 largest files first (Phase 2-4)
- Files 11-73 may be addressed in Phase 6.3 if time permits
- Some files are close to the 500-line limit and may not need immediate attention
- Priority should be given to files over 800 lines for maximum impact
