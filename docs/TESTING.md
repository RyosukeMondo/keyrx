# Testing & Coverage (Flutter UI)

This guide covers the full UI test stack (unit, widget, golden, performance, memory) plus coverage enforcement. Default gates mirror pre-commit: 60% minimum today, target 80%.

## Commands you’ll use most
- Run everything fast: `just ui-test` (honors `CONCURRENCY`, default 4).
- Watch mode: `just ui-test-watch`.
- Quality bundle (format + analyze + metrics + tests): `just ui-quality`.
- Goldens: `just ui-golden-verify` / `just ui-golden-update`.
- Coverage HTML: `just ui-coverage` (lcov + genhtml).

## Prerequisites
- Flutter SDK and `dart` on PATH.
- Coverage tooling installed (`just setup` or `./scripts/install-coverage-tools.sh` for `dart coverage`, `lcov`, `genhtml`).
- Run commands from repo root unless noted.

## Unit & service tests
- Standard entry: `cd ui && flutter test --concurrency ${CONCURRENCY:-4}`.
- Tag slow/integration tests; the coverage scripts default to `EXCLUDE_TAGS=slow,integration` to keep the loop tight.
- Keep helpers in `ui/test/helpers/` to reduce duplication; prefer fakes over mocks when possible.

## Widget harness (Riverpod + provider)
- Use `WidgetTestHarness` (`ui/test/helpers/widget_test_harness.dart`) to wrap widgets in `ProviderScope` + `MultiProvider` with fake services (FakeBridge, fake registry, MockKeyrxFacade, AppState).
- Quick start:
  ```dart
  final harness = WidgetTestHarness();
  await harness.pumpPage(
    tester,
    Consumer(
      builder: (context, ref, _) => Text('Bridge: ${ref.read(bridgeProvider).version}'),
    ),
  );
  await tester.tapAndSettle(find.text('Submit'));
  tester.expectOne(find.text('Bridge: fake-1.0.0'));
  ```
- Override dependencies via `overrides`/`providerOverrides` when a test needs custom fakes.
- `WidgetTesterX` adds `tapAndSettle`, `enterTextAndSettle`, `expectOne`, and `expectNone`. Harness defaults to `MaterialApp + Scaffold`; disable with `wrapWithScaffold: false` for fragments.

## Golden tests (golden_toolkit)
- Global config in `ui/flutter_test_config.dart` loads fonts and standardizes devices (phone, iPhone 11, tablet landscape) with real shadows enabled.
- Harness: `GoldenTestHarness` (`ui/test/helpers/golden_test_harness.dart`) reuses the widget harness and consistent surface sizes.
- Commands: `just ui-golden-verify` to check, `just ui-golden-update` to refresh baselines (`CONCURRENCY` defaults to 2 for stability).
- Example:
  ```dart
  testGoldens('renders dialog', (tester) async {
    final harness = GoldenTestHarness();
    await harness.pump(tester, ConfirmationDialog(title: 'Confirm', message: 'Proceed?'));
    await harness.expectGolden(tester, 'goldens/confirmation_dialog_basic');
  });
  ```

## Performance & memory suites
- Performance baselines live in `ui/test/performance/` (startup, widget render, FFI latency). Run with `cd ui && flutter test test/performance`.
- Memory leak checks live in `ui/test/memory/` (uses `leak_tracker_*` to assert controllers/subscriptions are released). Run with `cd ui && flutter test test/memory`.
- Benchmarks target: app startup <500ms to interactive, TradeOffChart render <350ms, MigrationPromptPage build <300ms, async FFI trace <40ms wall clock.

## Coverage
- Fast gate (pre-commit mirror):
  ```bash
  cd ui
  COVERAGE_THRESHOLD=60 ./scripts/check_coverage.sh
  ```
  - Tunables: `TEST_FILES`, `EXCLUDE_TAGS` (default `slow,integration`), `CONCURRENCY` (default `4`), `COVERAGE_THRESHOLD` (set to `0` to collect without gating).
- Collect normalized coverage (lcov):
  ```bash
  cd ui
  ./scripts/collect_coverage.sh
  ```
  - Outputs `coverage/lcov.info` (filtered) and `.raw`. Filters drop generated code (`*.g.dart`, `*.freezed.dart`, `*.mocks.dart`, `generated/`, `l10n/`); override via `FILTER_PATTERNS`.
- Generate HTML report:
  ```bash
  cd ui
  ./scripts/generate_coverage_report.sh
  # reuse existing lcov
  COLLECT_COVERAGE=0 ./scripts/generate_coverage_report.sh
  ```
  - Output: `ui/coverage/html/index.html` (gitignored). Override `HTML_OUTPUT_DIR`, `COVERAGE_DIR`, `LCOV_FILE`.

## Recommended workflow
1) Install tools once: `just setup`.
2) While coding: `COVERAGE_THRESHOLD=60 ./scripts/check_coverage.sh` (bump toward 80% as the suite grows).
3) Goldens before PR: `just ui-golden-verify`.
4) Performance/memory checks before risky UI or FFI changes: run the suites under `test/performance` and `test/memory`.
5) Use `just ui-quality` for a full gate before pushing.
