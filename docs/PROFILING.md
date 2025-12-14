# Flutter DevTools & Profiling Guide

Use these steps to stand up Flutter DevTools, capture useful traces, and validate performance with automated tests.

## Quick Setup
- Install DevTools: `flutter pub global activate devtools` (one-time).
- Start DevTools: `flutter pub global run devtools --machine --port 9100` (or `dart run devtools` if using the bundled tool).
- Run the app with VM service enabled: `cd ui && flutter run --profile --trace-skia --vm-service-port 9101`.
- Open the DevTools URL printed by the server (default `http://localhost:9100`) and paste the VM service URI from `flutter run`.

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
- Run it alone: `cd ui && flutter test test/performance/startup_timeline_test.dart`
- Include in quality suite: `just ui-test` (runs all widget/unit tests).

## Tips for Faster Investigations
- Add one-off spans: `PerformanceTimelineTracer.instant('event', context: {'detail': 'value'})`.
- Record startup: `flutter run --profile --trace-startup` writes `startup_trace.json` you can load in Chrome tracing.
- Keep background work off the UI thread; long-running tasks should use isolates or awaitable services to keep the timeline flat.
