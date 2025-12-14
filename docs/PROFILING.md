# Flutter DevTools & Profiling Guide

Use these steps to stand up Flutter DevTools, capture useful traces, and validate performance with automated tests.

## Quick Setup
- Install DevTools: `flutter pub global activate devtools` (one-time).
- Start DevTools: `flutter pub global run devtools --machine --port 9100` (or `dart run devtools` if using the bundled tool).
- Run the app with VM service enabled: `cd ui && flutter run --profile --trace-skia --vm-service-port 9101`.
- Open the DevTools URL printed by the server (default `http://localhost:9100`) and paste the VM service URI from `flutter run`.

## Performance budgets (targets)
- App startup to interactive: <500ms.
- TradeOffChart render: <350ms; MigrationPromptPage build: <300ms on 1280x900.
- Async FFI trace (mock 12ms call): <40ms wall clock; sync trace overhead <10ms.
- Keep main isolate free of heavy work; move parsing/IO to isolates or async services.

## Profiling Workflow
1) **Record a timeline trace**
   - DevTools → Performance tab → Record.
   - Exercise the flow (app startup, load Devices page, run a bridge call).
   - Stop recording → inspect Flutter frames, shader compilation, and our custom spans.
2) **Inspect CPU & Memory**
   - CPU Profiler: capture a 10–20s sample around the hot path, sort by Self Time.
   - Memory: take a Heap Snapshot → filter by `Keyrx` / `bridge` classes, look for leaks.
3) **Export artifacts**
   - Performance tab → `Export` for sharing in PRs.
   - CPU tab → save CPU profile (JSON) for regression baselines.

## Timeline Instrumentation
We emit Timeline events that show up directly in DevTools:
- `app.startup` – wraps Flutter bootstrap; closes after first frame.
- `services.bootstrap` – lifecycle for building the service registry.
- `ffi.bridge.open` – native library load attempt (includes load failure context).
- `ffi.bridge.initialize` – engine init duration and success flag.
- Existing FFI traces: `FfiDebugTools.trace*` spans for individual native calls.

Use these markers to line up UI frames with engine initialization and to spot slow FFI boundaries.

## Performance Tests
- Example benchmark: `ui/test/performance/startup_timeline_test.dart`
  - Uses the widget test harness + `PerformanceTimelineTracer` to assert the Profiler page builds under 500ms.
- New baselines (fail the build if exceeded):
  - Widget render: `ui/test/performance/widget_render_benchmark_test.dart`
    - TradeOffChart <350ms to render (Material3 + chart draw).
    - MigrationPromptPage <300ms to build on 1280x900 viewport.
  - FFI latency: `ui/test/performance/ffi_latency_benchmark_test.dart`
    - Async trace (12ms mock call) <40ms total wall clock.
    - Sync trace overhead <10ms.
- Run a single benchmark: `cd ui && flutter test test/performance/widget_render_benchmark_test.dart`
- Run the whole suite: `cd ui && flutter test test/performance`
- Include in quality suite: `just ui-test` (runs all widget/unit tests).

## Tips for Faster Investigations
- Add one-off spans: `PerformanceTimelineTracer.instant('event', context: {'detail': 'value'})`.
- Record startup: `flutter run --profile --trace-startup` writes `startup_trace.json` you can load in Chrome tracing.
- Keep background work off the UI thread; long-running tasks should use isolates or awaitable services to keep the timeline flat.

## Memory Profiling & Leak Detection
- Capture a heap snapshot in DevTools: Run `flutter run --profile`, open DevTools → Memory tab → Take Heap Snapshot → Filter by `Keyrx` classes and look for growing retainers (especially controllers, streams, and FFI handles).
- Live allocations view: Use DevTools `Allocation Profile` to watch live growth while exercising console input and monitor mode; flatten spikes before merging.
- Automated leak checks: `cd ui && flutter test test/memory/memory_profiling_test.dart` runs leak-tracked widget tests for Console and Monitor pages (uses `leak_tracker_*`), asserting controllers/subscriptions are released and monitor event buffers stay bounded.
- Track RSS deltas: Memory test also asserts resident set size growth stays under 40MB after heavy monitor traffic to catch runaway buffers early.
- When investigating a leak: Re-run with `--enable-heap-snapshot` and take snapshots before/after the suspect flow; diff by retaining path to find the root owner, then add a regression test under `ui/test/memory/`.

## Regression guardrails
- Before merging UI/FFI changes: run `cd ui && flutter test test/performance` and `cd ui && flutter test test/memory`.
- For visual regressions: `just ui-golden-verify` (goldens live under `test/widgets/golden`).
- For quality gates: use `just ui-quality` or `just ui-coverage` to keep coverage near the 80% target while profiling changes.
