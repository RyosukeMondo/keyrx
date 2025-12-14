# Testing & Coverage (Flutter UI)

This guide explains how to run Flutter UI tests with coverage, enforce thresholds, and generate HTML reports using the shared scripts under `ui/scripts`.

## Prerequisites
- Flutter SDK installed and on `PATH`.
- Coverage tooling installed (run `just setup` or `./scripts/install-coverage-tools.sh` to install `dart coverage` + `lcov`/`genhtml`).
- From repo root, run commands inside `ui/` unless noted.

## Fast coverage check (pre-commit mirror)
- The git hook calls `ui/scripts/check_coverage.sh` against a fast Dart subset to keep commits quick.
- To reproduce locally with the default full suite and a 60% gate:
  ```bash
  cd ui
  COVERAGE_THRESHOLD=60 ./scripts/check_coverage.sh
  ```
- Override knobs when needed:
  - `TEST_FILES` to limit scope (e.g., `TEST_FILES=test/services/keymap_converter_test.dart`).
  - `EXCLUDE_TAGS` (default `slow,integration`) and `CONCURRENCY` (default `4`).
  - `COVERAGE_THRESHOLD` set to `0` to collect without gating. Raise toward the 80% target as the suite matures.

## Collect normalized coverage (lcov)
Use `collect_coverage.sh` to run tests and filter generated code:
```bash
cd ui
./scripts/collect_coverage.sh
```
- Output: `ui/coverage/lcov.info` (filtered) and `ui/coverage/lcov.info.raw` (unfiltered).
- Filters drop `*.g.dart`, `*.freezed.dart`, `generated/`, `*.mocks.dart`, and `l10n/` files by default; override with `FILTER_PATTERNS`.
- Key envs: `COVERAGE_DIR` (default `coverage`), `TEST_RANDOM_SEED` (`random`), `EXCLUDE_TAGS`, `TEST_FILES`, `CONCURRENCY`.

## Enforce thresholds
`check_coverage.sh` wraps collection plus optional gating:
```bash
cd ui
COVERAGE_THRESHOLD=60 ./scripts/check_coverage.sh
```
- Set `COLLECT_COVERAGE=0` to reuse an existing `LCOV_FILE`.
- Prints line coverage summary and exits non-zero if the threshold is not met.

## Generate HTML report
- Quick path: `just ui-coverage` (runs collection + HTML generation).
- Script path: `ui/scripts/generate_coverage_report.sh`
  ```bash
  cd ui
  ./scripts/generate_coverage_report.sh
  # or reuse existing lcov
  COLLECT_COVERAGE=0 ./scripts/generate_coverage_report.sh
  ```
- Output: `ui/coverage/html/index.html` (safe to open locally; ignored by git). Override with `HTML_OUTPUT_DIR`, `COVERAGE_DIR`, and `LCOV_FILE`.

## Recommended workflow
1) Install tools once: `just setup`.
2) During development: run `COVERAGE_THRESHOLD=60 ./scripts/check_coverage.sh` before committing; bump toward 80% as coverage improves.
3) For reports: `just ui-coverage` and open `ui/coverage/html/index.html`.
4) Keep the fast pre-commit subset healthy; expand `TEST_FILES` as new suites are added.

## Widget test harness (Riverpod + provider)
- Use `WidgetTestHarness` from `ui/test/helpers/widget_test_harness.dart` to wrap widgets in `ProviderScope` + `MultiProvider` with fake services (FakeBridge, fake registry, MockKeyrxFacade, AppState).
- Quick start:
  ```dart
  final harness = WidgetTestHarness();
  await harness.pumpPage(
    tester,
    Consumer(
      builder: (context, ref, _) =>
          Text('Bridge: ${ref.read(bridgeProvider).version}'),
    ),
  );
  await tester.tapAndSettle(find.text('Submit'));
  tester.expectOne(find.text('Bridge: fake-1.0.0'));
  ```
- Override dependencies with `overrides` or `providerOverrides` when a test needs custom fakes.
- `WidgetTesterX` adds `tapAndSettle`, `enterTextAndSettle`, `expectOne`, and `expectNone` to keep assertions terse.
- The harness defaults to `MaterialApp + Scaffold`; pass `wrapWithScaffold: false` when testing fragments that already provide layout chrome.

## Golden tests (golden_toolkit)
- Global config: `ui/flutter_test_config.dart` loads fonts once and standardizes device names (phone, iPhone 11, tablet landscape) with real shadows enabled.
- Harness: use `GoldenTestHarness` (`ui/test/helpers/golden_test_harness.dart`) to reuse the widget harness and consistent surface sizes.
- Commands:
  - Verify: `just ui-golden-verify` (runs `flutter test test/widgets/golden`).
  - Update baselines: `just ui-golden-update` (runs with `--update-goldens`).
- Example:
  ```dart
  testGoldens('renders dialog', (tester) async {
    final harness = GoldenTestHarness();
    await harness.pump(tester, ConfirmationDialog(title: 'Confirm', message: 'Proceed?'));
    await harness.expectGolden(tester, 'goldens/confirmation_dialog_basic');
  });
  ```
