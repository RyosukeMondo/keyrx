# Automated API E2E Testing with Auto-Fix

## Overview
This spec implements an automated end-to-end testing system that exercises web UI backend features via REST API, validates responses against expected results, and iteratively fixes issues automatically.

## Motivation
- **Catch regressions early**: Automated testing on every PR prevents API contract breakage
- **Reduce manual testing**: Auto-fix common issues (network, schema, data) without human intervention
- **Improve confidence**: Comprehensive API coverage with 30+ test cases
- **Enable iteration**: Fix-retry loop allows tests to self-heal and unblock CI

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Test Runner (CLI)                        │
│  - Launch daemon                                            │
│  - Execute test suite                                       │
│  - Orchestrate fix-retry loop                               │
└─────────────┬───────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Test Executor                              │
│  - Run 30+ API test cases (sequential)                      │
│  - Collect results with timing                              │
└─────────────┬───────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────────┐
│               Result Comparator                             │
│  - Deep diff: actual vs expected                            │
│  - Ignore fields (timestamps, IDs)                          │
│  - Generate actionable diffs                                │
└─────────────┬───────────────────────────────────────────────┘
              │
              ├──► [PASS] → Report Success
              │
              └──► [FAIL] ──────┐
                                 │
                                 ▼
              ┌─────────────────────────────────────────────────┐
              │           Issue Classifier                      │
              │  - Network errors → restart daemon              │
              │  - Schema mismatches → update expected results  │
              │  - Data errors → reseed fixtures                │
              └─────────────┬───────────────────────────────────┘
                            │
                            ▼
              ┌─────────────────────────────────────────────────┐
              │          Auto-Fix Strategies                    │
              │  - RestartDaemonStrategy                        │
              │  - UpdateExpectedResultStrategy                 │
              │  - ReseedFixtureStrategy                        │
              │  - RetryTestStrategy                            │
              └─────────────┬───────────────────────────────────┘
                            │
                            ▼
              ┌─────────────────────────────────────────────────┐
              │           Fix Orchestrator                      │
              │  - Apply fixes in priority order                │
              │  - Retry test after each fix                    │
              │  - Track fix history (max 3 iterations)         │
              └─────────────┬───────────────────────────────────┘
                            │
                            ├──► [FIXED] → Report Success
                            │
                            └──► [STILL FAILING] → Report Failure
```

## Key Features

### 1. Comprehensive API Coverage
- **30+ test cases** covering all REST endpoints
- **Type-safe validation** using Zod schemas from `keyrx_ui/src/api/schemas.ts`
- **Multiple scenarios** per endpoint (success, empty, error cases)

### 2. Intelligent Auto-Fix
- **Network issues**: Restart daemon, wait longer, retry requests
- **Schema mismatches**: Update expected results (with human approval in CI)
- **Data issues**: Re-seed test fixtures, clean up stale data
- **Transient failures**: Retry with exponential backoff

### 3. Detailed Reporting
- **Human-readable console output**: Color-coded pass/fail, diffs
- **JSON report**: Machine-parseable for CI integrations
- **HTML report**: Visual inspection with syntax highlighting
- **Fix attempt history**: Track what was attempted and why

### 4. CI Integration
- **GitHub Actions workflow**: Run on every PR
- **Artifact upload**: Test results, HTML reports, daemon logs
- **PR comments**: Summary of test results
- **Metrics collection**: Track pass rate, duration, fix success rate over time

## Quick Start

### Local Development
```bash
# Build daemon
make build

# Run automated e2e tests with auto-fix
cd keyrx_ui
npm run test:e2e:auto

# Generate HTML report
npm run test:e2e:auto:report

# Open report in browser
open report.html
```

### CI/CD
Tests run automatically on every PR via `.github/workflows/e2e-auto.yml`

## File Structure

```
keyrx/
├── scripts/
│   ├── automated-e2e-test.ts             # CLI test runner
│   ├── fixtures/
│   │   ├── daemon-fixture.ts             # Daemon lifecycle management
│   │   └── expected-results.json         # Expected API responses
│   ├── api-client/
│   │   └── client.ts                     # Typed API client
│   ├── test-cases/
│   │   └── api-tests.ts                  # Test case definitions
│   ├── test-executor/
│   │   └── executor.ts                   # Test orchestration
│   ├── comparator/
│   │   ├── response-comparator.ts        # Deep diff comparison
│   │   └── validation-reporter.ts        # Human/JSON reports
│   ├── auto-fix/
│   │   ├── issue-classifier.ts           # Classify failure types
│   │   ├── fix-strategies.ts             # Fix implementations
│   │   └── fix-orchestrator.ts           # Fix-retry loop
│   ├── reporters/
│   │   └── html-reporter.ts              # Visual HTML reports
│   ├── metrics/
│   │   └── test-metrics.ts               # Historical metrics
│   └── dashboard/
│       └── e2e-dashboard.html            # Real-time monitoring
├── .github/workflows/
│   └── e2e-auto.yml                      # CI integration
└── .spec-workflow/specs/automated-api-e2e-testing/
    ├── README.md                          # This file
    └── tasks.md                           # Implementation tasks
```

## Compliance with CLAUDE.md

### Code Quality
- ✅ **Max 500 lines per file**: All script files modularized
- ✅ **Max 50 lines per function**: Functions extracted, SLAP applied
- ✅ **Test coverage**: Test infrastructure itself has unit tests
- ✅ **SOLID principles**: Dependency injection, single responsibility
- ✅ **Error handling**: Structured errors, fail-fast validation

### Naming Conventions
- ✅ **Files**: `snake-case.ts` (e.g., `response-comparator.ts`)
- ✅ **Functions**: `camelCase` (e.g., `executeTests`, `applyFixes`)
- ✅ **Classes**: `PascalCase` (e.g., `ApiClient`, `FixOrchestrator`)
- ✅ **Constants**: `UPPER_SNAKE_CASE` (e.g., `MAX_ITERATIONS`)

### Development
- ✅ **CLI first**: `automated-e2e-test.ts` is CLI tool with rich flags
- ✅ **Debug mode**: `--verbose` flag for detailed logging
- ✅ **Structured logging**: JSON output with `--report-json`

## Implementation Plan

See [tasks.md](./tasks.md) for detailed task breakdown (20 tasks across 7 phases).

### Phase Summary
1. **Infrastructure** (3 tasks): Test runner, daemon fixture, expected results database
2. **Test Suite** (3 tasks): API client, test cases, executor
3. **Comparison** (2 tasks): Response comparator, reporter
4. **Auto-Fix** (3 tasks): Issue classifier, fix strategies, orchestrator
5. **Integration** (3 tasks): Wire components, HTML report, npm scripts
6. **CI Integration** (3 tasks): GitHub Actions, metrics, dashboard
7. **Documentation** (3 tasks): README, dev guide, examples

**Estimated completion**: 3-5 days for experienced developer

## Benefits

### For Developers
- **Fast feedback**: Catch API breakage in < 2 minutes
- **Self-healing tests**: Auto-fix common issues, unblock development
- **Clear diagnostics**: Actionable diffs, fix attempt history
- **Easy to extend**: Add test cases with minimal boilerplate

### For QA
- **Comprehensive coverage**: 30+ test cases, all endpoints
- **Visual reports**: HTML report for manual inspection
- **Metrics tracking**: Monitor test health over time
- **CI integration**: Automated on every PR

### For Product
- **Higher confidence**: API contracts validated automatically
- **Faster iteration**: Reduce manual testing burden
- **Lower risk**: Catch regressions before production

## Success Metrics

### Functional
- ✅ 30+ API test cases (all endpoints covered)
- ✅ Auto-fix success rate > 60% for fixable issues
- ✅ Test execution time < 2 minutes
- ✅ CI integration with artifact upload

### Quality
- ✅ Type-safe API client (Zod validation)
- ✅ Detailed diffs on failure (actionable)
- ✅ Fix history tracking (prevent infinite loops)
- ✅ Metrics collection (historical trends)

### Developer Experience
- ✅ One command: `npm run test:e2e:auto`
- ✅ Clear failure messages
- ✅ HTML report for inspection
- ✅ Easy to add tests (template provided)

## Future Enhancements

### Phase 8 (Future)
- **Parallel test execution**: Speed up suite with worker threads
- **Snapshot testing**: Auto-generate expected results from actual responses
- **AI-powered fix suggestions**: Use LLM to suggest fixes for logic bugs
- **Performance regression detection**: Track response times, alert on slowdowns
- **Contract-first development**: Generate tests from OpenAPI spec

## Related Specs

- **e2e-playwright-testing**: Browser UI testing (complements this spec)
- **api-contract-testing**: Manual contract validation (replaced by this spec)
- **frontend-test-infrastructure**: UI unit tests (orthogonal to this spec)

## Contributing

See [DEV_GUIDE.md](./DEV_GUIDE.md) (created in task 7.2) for developer guide.

## License

Same as KeyRx project license.
