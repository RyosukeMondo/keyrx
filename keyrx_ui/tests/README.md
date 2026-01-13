# Test Strategy Documentation

## Test Pyramid Overview

This project follows the **test pyramid** strategy to maintain a sustainable and efficient test suite. The pyramid prioritizes fast, focused unit tests while using slower, more comprehensive integration and E2E tests strategically.

```
        /\
       /  \    E2E (10%)
      /____\   ~12 tests
     /      \  Browser automation, critical user flows
    /________\
   /          \ Integration (20%)
  /____________\ ~25 tests
 /              \ Component interactions, page-level behavior
/________________\
      Unit (70%)
    ~90 tests
    Individual functions, components, hooks
```

### Target Test Distribution

| Test Type | Percentage | Count Target | Current |
|-----------|-----------|--------------|---------|
| Unit | 70% | ~90 tests | 68 |
| Integration | 20% | ~25 tests | 3 |
| E2E | 10% | ~12 tests | 14 |

**Current Status**: Good E2E coverage, need more integration tests to balance the pyramid.

---

## Test Type Guidelines

### 1. Unit Tests (70%)

**Purpose**: Test individual units of code in isolation (functions, components, hooks)

**When to use**:
- Testing pure functions and utilities
- Testing component rendering with minimal dependencies
- Testing custom hooks in isolation
- Testing business logic and calculations
- Testing error handling and edge cases

**Characteristics**:
- **Fast**: < 50ms per test (target), < 1s max (warning threshold)
- **Isolated**: No network, filesystem, or database access
- **Focused**: Test one thing at a time
- **Mocked**: External dependencies are mocked

**Naming convention**:
- File: `*.test.ts` or `*.test.tsx`
- Location: Co-located with source (`src/components/Button.test.tsx`)
- Test name: `describe('ComponentName', () => { it('should do X when Y', ...) })`

**Example structure**:
```typescript
// src/utils/formatTimestamp.test.ts
import { formatTimestamp } from './formatTimestamp';

describe('formatTimestamp', () => {
  it('should format milliseconds correctly', () => {
    expect(formatTimestamp(1500)).toBe('1.50s');
  });

  it('should handle zero correctly', () => {
    expect(formatTimestamp(0)).toBe('0ms');
  });
});
```

**Coverage target**: ≥90% for critical paths (hooks, utils, core logic)

---

### 2. Integration Tests (20%)

**Purpose**: Test how multiple components work together or how a component integrates with external systems

**When to use**:
- Testing full page rendering with real state management
- Testing component interactions (parent-child communication)
- Testing API integration (with mocked backend)
- Testing WebSocket connection flows
- Testing complex user workflows across multiple components

**Characteristics**:
- **Moderate speed**: < 5s per test
- **Partial integration**: Real components, mocked external services
- **User-focused**: Test from user's perspective
- **State management**: Use real stores/contexts

**Naming convention**:
- File: `*.integration.test.ts` or `*.integration.test.tsx`
- Location: Co-located with source or in `tests/integration/`
- Test name: `describe('Page Integration', () => { it('user can complete workflow X', ...) })`

**Example structure**:
```typescript
// src/pages/ProfilesPage.integration.test.tsx
import { renderWithProviders } from '@/tests/testUtils';
import { ProfilesPage } from './ProfilesPage';
import { setupMockWebSocket } from '@/tests/helpers/websocket';

describe('ProfilesPage Integration', () => {
  it('user can create and activate a new profile', async () => {
    const { server } = setupMockWebSocket();
    const { user } = renderWithProviders(<ProfilesPage />);

    // User creates profile
    await user.click(screen.getByRole('button', { name: /new profile/i }));
    await user.type(screen.getByLabelText(/name/i), 'Gaming');
    await user.click(screen.getByRole('button', { name: /save/i }));

    // WebSocket responds
    server.send(JSON.stringify({ id: 1, result: { success: true } }));

    // Profile appears and can be activated
    expect(await screen.findByText('Gaming')).toBeInTheDocument();
  });
});
```

**Coverage target**: ≥80% for page components and critical integrations

---

### 3. E2E Tests (10%)

**Purpose**: Test complete user flows in a real browser with the full application stack

**When to use**:
- Testing critical business workflows end-to-end
- Testing authentication and authorization flows
- Testing multi-page user journeys
- Testing browser-specific behavior (navigation, local storage)
- Visual regression testing for key pages

**Characteristics**:
- **Slow**: 5-30s per test
- **Full stack**: Real browser, real backend (or staging environment)
- **User perspective**: No implementation details exposed
- **Comprehensive**: Tests entire system integration

**Naming convention**:
- File: `*.spec.ts` (Playwright convention)
- Location: `tests/e2e/` directory
- Test name: `test('user can complete checkout flow', ...) `

**Example structure**:
```typescript
// tests/e2e/profile-workflow.spec.ts
import { test, expect } from '@playwright/test';

test('user can create, edit, and activate a profile', async ({ page }) => {
  // Navigate to profiles page
  await page.goto('http://localhost:3000/profiles');

  // Create new profile
  await page.click('button:has-text("New Profile")');
  await page.fill('input[name="profileName"]', 'Work Setup');
  await page.click('button:has-text("Save")');

  // Verify profile appears
  await expect(page.locator('text=Work Setup')).toBeVisible();

  // Activate profile
  await page.click('button:has-text("Activate")');
  await expect(page.locator('.profile-active')).toBeVisible();
});
```

**Coverage target**: Cover top 10 critical user workflows

---

## Test File Organization

```
keyrx_ui/
├── src/
│   ├── components/
│   │   ├── Button.tsx
│   │   ├── Button.test.tsx              # Unit test (co-located)
│   │   └── Dialog.integration.test.tsx  # Integration test (if needed)
│   ├── hooks/
│   │   ├── useProfiles.ts
│   │   └── useProfiles.test.tsx         # Unit test
│   ├── pages/
│   │   ├── ProfilesPage.tsx
│   │   ├── ProfilesPage.test.tsx        # Unit test (basic rendering)
│   │   └── ProfilesPage.integration.test.tsx  # Integration test (workflows)
│   └── utils/
│       ├── timeFormatting.ts
│       └── timeFormatting.test.ts       # Unit test
└── tests/
    ├── e2e/
    │   ├── profile-workflow.spec.ts     # E2E test
    │   └── device-management.spec.ts    # E2E test
    ├── integration/
    │   └── websocket-reconnect.integration.test.ts  # Shared integration tests
    ├── a11y/
    │   └── colorContrast.test.tsx       # Accessibility tests (integration-level)
    ├── helpers/
    │   ├── websocket.ts                 # Test utilities
    │   └── testUtils.tsx                # Shared test helpers
    └── README.md                        # This file
```

**Organization principles**:
- **Co-location**: Unit tests live next to the code they test
- **Separation**: Integration and E2E tests in `tests/` directory
- **Categorization**: Clear naming conventions indicate test type
- **Helpers**: Shared test utilities in `tests/helpers/`

---

## Test Naming Conventions

### File Naming

| Test Type | Pattern | Example |
|-----------|---------|---------|
| Unit | `*.test.{ts,tsx}` | `Button.test.tsx` |
| Integration | `*.integration.test.{ts,tsx}` | `ProfilesPage.integration.test.tsx` |
| E2E | `*.spec.ts` | `profile-workflow.spec.ts` |
| Accessibility | `*.a11y.test.{ts,tsx}` | `colorContrast.a11y.test.tsx` |

### Test Description Naming

**Unit tests**: Focus on behavior and outcomes
```typescript
describe('formatTimestamp', () => {
  it('should format milliseconds as "Xms"', () => { ... });
  it('should format seconds as "X.XXs"', () => { ... });
  it('should throw error for negative values', () => { ... });
});
```

**Integration tests**: Focus on user workflows
```typescript
describe('ProfilesPage Integration', () => {
  it('user can create a new profile', async () => { ... });
  it('user can edit an existing profile', async () => { ... });
  it('displays error when profile name is duplicate', async () => { ... });
});
```

**E2E tests**: Focus on complete user journeys
```typescript
test('user can manage keyboard profiles end-to-end', async ({ page }) => { ... });
test('user can configure device settings and apply changes', async ({ page }) => { ... });
```

---

## Coverage Expectations

### Global Coverage Targets

- **Overall**: ≥80% line and branch coverage
- **Critical paths**: ≥90% coverage
  - `src/hooks/` - Core business logic
  - `src/api/` - Backend communication
  - `src/utils/` - Shared utilities

### Per-Test-Type Coverage

| Test Type | Coverage Contribution | Focus |
|-----------|----------------------|-------|
| Unit | ~60-70% | Core logic, utilities, pure functions |
| Integration | ~15-20% | Component interactions, state management |
| E2E | ~5-10% | Critical paths not covered by unit/integration |

**Coverage exclusions** (configured in `vitest.config.base.ts`):
- Test files themselves (`**/*.test.{ts,tsx}`)
- Mock implementations (`tests/mocks/**`)
- Type definitions (`**/*.d.ts`)
- WASM bindings (`src/wasm/pkg/**`)

### Coverage Commands

```bash
# Run coverage for all tests
npm run test:coverage

# Run coverage for unit tests only
vitest run --coverage --config vitest.unit.config.ts

# Run coverage for integration tests only
vitest run --coverage --config vitest.integration.config.ts

# View HTML coverage report
open coverage/index.html
```

---

## Test Execution

### Running Tests

```bash
# Unit tests (default)
npm test                    # Run once
npm run test:watch          # Watch mode

# Integration tests
npm run test:integration
npm run test:integration:watch

# E2E tests
npm run test:e2e            # All E2E tests
npm run test:e2e:ui         # Interactive UI mode

# Accessibility tests
npm run test:a11y

# All tests
npm run test:all
```

### Focused Test Runs (Developer Productivity)

```bash
# Run only changed files since last commit
npm run test:changed

# Run tests related to changed files
npm run test:related

# Re-run only failed tests
npm run test:failed

# Smart watch mode (only changed files)
npm run test:watch:smart
```

### Parallel Execution

Tests run in parallel by default (configured in `vitest.config.base.ts`):
- Thread pool optimized for 75% CPU utilization
- Unit tests: Full parallelization
- Integration tests: Parallel where safe
- E2E tests: Sequential to avoid state conflicts

**CI sharding** (split tests across multiple runners):
```bash
npm run test:shard 1/3      # Run shard 1 of 3
npm run test:shard 2/3      # Run shard 2 of 3
npm run test:shard 3/3      # Run shard 3 of 3
```

---

## Test Configuration Files

### Unit Tests
**Config**: `vitest.unit.config.ts`
- Timeout: 3000ms
- Slow test threshold: 1000ms
- Includes: `src/**/*.test.{ts,tsx}`
- Excludes: Integration, E2E, accessibility tests

### Integration Tests
**Config**: `vitest.integration.config.ts`
- Timeout: 30000ms
- Includes: `**/*.integration.test.{ts,tsx}`, `tests/integration/**`
- Includes: Accessibility tests (`tests/a11y/**`)

### E2E Tests
**Config**: `playwright.e2e.config.ts`
- Timeout: 30000ms
- Browser: Chromium, Firefox, WebKit
- Base URL: http://localhost:3000
- Artifacts: Screenshots on failure, traces on failure

---

## When to Write Each Test Type

### Decision Tree

```
Is this testing a single function/component in isolation?
├─ YES → Write a UNIT test
└─ NO → Is this testing multiple components working together?
    ├─ YES → Write an INTEGRATION test
    └─ NO → Is this testing a critical end-to-end user flow?
        ├─ YES → Write an E2E test
        └─ NO → Reconsider if test is needed
```

### Examples

| Scenario | Test Type | Rationale |
|----------|-----------|-----------|
| Testing `formatTimestamp()` utility | Unit | Pure function, no dependencies |
| Testing `<Button>` component renders | Unit | Single component, isolated |
| Testing `useProfiles()` hook | Unit | Isolated logic, mocked dependencies |
| Testing ProfilesPage with API calls | Integration | Multiple components + API integration |
| Testing full profile creation workflow | Integration | Multi-step workflow, mocked backend |
| Testing keyboard configuration end-to-end | E2E | Critical business flow, real browser |

---

## Best Practices

### Unit Tests

✅ **Do**:
- Test pure functions exhaustively (happy path + edge cases)
- Use `renderWithProviders()` for React components
- Mock external dependencies (API, WebSocket, localStorage)
- Test one behavior per test
- Use descriptive test names

❌ **Don't**:
- Test implementation details (internal state, private methods)
- Make network requests
- Test third-party libraries
- Write slow tests (> 1s triggers warning)

### Integration Tests

✅ **Do**:
- Test from user's perspective (click buttons, type input)
- Use real state management (stores, contexts)
- Mock external services (API, WebSocket)
- Test error states and loading states
- Use `waitFor` and `findBy` queries for async behavior

❌ **Don't**:
- Test every component combination (too many tests)
- Make real API calls
- Test what unit tests already cover
- Rely on arbitrary timeouts

### E2E Tests

✅ **Do**:
- Focus on critical business workflows
- Test with realistic data
- Verify complete user journeys
- Use Page Object Model for reusability
- Capture screenshots/videos on failure

❌ **Don't**:
- Test every edge case (unit tests cover those)
- Write brittle selectors (prefer accessible queries)
- Test non-critical flows
- Skip error handling (use retries for flaky tests)

---

## Test Utilities and Helpers

### Shared Test Utilities

Located in `tests/testUtils.tsx`:

```typescript
import { renderWithProviders } from '@/tests/testUtils';

// Wraps component with necessary providers (Router, WebSocket, State)
const { user } = renderWithProviders(<MyComponent />);
```

### WebSocket Test Helpers

Located in `tests/helpers/websocket.ts`:

```typescript
import { setupMockWebSocket } from '@/tests/helpers/websocket';

const { server, cleanup } = setupMockWebSocket();

// Send RPC response
server.send(JSON.stringify({ id: 1, result: { success: true } }));

// Cleanup automatically in afterEach
cleanup();
```

### Accessibility Helpers

Located in `tests/AccessibilityTestHelper.ts`:

```typescript
import { checkColorContrast } from '@/tests/AccessibilityTestHelper';

await checkColorContrast(container);
```

---

## Troubleshooting

### Common Issues

**"Test timeout exceeded"**
- Check if using correct config (unit vs integration timeout)
- Ensure async operations use `await` and `waitFor`
- Check for missing cleanup causing hanging promises

**"Unable to find element"**
- Use `findBy*` queries for async elements (instead of `getBy*`)
- Wrap in `waitFor(() => { ... })` for delayed renders
- Check if component is wrapped with necessary providers

**"WebSocket mock not working"**
- Ensure all messages are `JSON.stringify()`'d before sending
- Use `setupMockWebSocket()` helper for standardized setup
- Check cleanup is called in `afterEach`

**"Coverage not updating"**
- Run `npm run test:coverage` (not just `npm test`)
- Ensure test actually executes the code path
- Check coverage exclusions in `vitest.config.base.ts`

### Getting Help

1. Check this README for test strategy
2. Review existing tests for patterns
3. Check test configuration files for timeout/setup issues
4. Use `npm run test:ui` for interactive debugging
5. Check CI logs for detailed error messages

---

## Maintenance Guidelines

### Adding New Tests

1. Determine test type using decision tree above
2. Follow naming conventions for file and test names
3. Co-locate unit tests with source code
4. Use shared test utilities for consistency
5. Verify coverage impact with `npm run test:coverage`

### Refactoring Tests

1. Keep test pyramid balance (70/20/10)
2. Consolidate duplicate test setup into helpers
3. Update tests when component APIs change
4. Remove obsolete tests for deleted features
5. Monitor slow tests and optimize (threshold: 1s for unit, 5s for integration)

### Test Metrics Monitoring

**Check test balance**:
```bash
# Unit tests
find src -name "*.test.ts" -o -name "*.test.tsx" | grep -v integration | wc -l

# Integration tests
find . -name "*.integration.test.*" | wc -l

# E2E tests
find tests/e2e -name "*.spec.ts" | wc -l
```

**Target ratios**: Maintain 70% unit, 20% integration, 10% E2E

---

## CI/CD Integration

Tests run automatically in CI pipeline (`.github/workflows/ci.yml`):

1. **Unit tests**: Run in parallel (3 shards)
2. **Integration tests**: Run sequentially (state-dependent)
3. **Accessibility tests**: Run in parallel with unit tests
4. **E2E tests**: Run on Ubuntu and Windows
5. **Coverage**: Collected and uploaded as artifact

**Quality gates** (must pass to merge):
- All unit tests pass
- All integration tests pass
- Accessibility tests pass (WCAG 2.2 Level AA)
- Coverage ≥80% overall
- No new flaky tests introduced

---

## Automatic Test Quality Enforcement

The project includes automated enforcement tools to maintain test quality and balance:

### ESLint Rule: Test Naming Convention

**Location**: `.eslintrc.cjs` + `eslint-local-rules/test-naming-convention.cjs`

**What it does**: Validates that test files follow the correct naming conventions:
- Unit tests: `*.test.ts` or `*.test.tsx`
- Integration tests: `*.integration.test.ts` or `*.integration.test.tsx`
- E2E tests: `*.e2e.ts`

**Errors detected**:
- Using `.spec.*` instead of `.test.*`
- Missing type indicators (e.g., `.integration.`)
- Inconsistent naming patterns

**Example output**:
```
src/components/Button.spec.tsx
  1:1  warning  Use ".test.*" instead of ".spec.*" for test files  local-rules/test-naming-convention
```

**To run**:
```bash
npm run lint           # Check all files
npm run lint:fix       # Auto-fix naming issues where possible
```

### Vitest Reporter: Test Balance Reporter

**Location**: `vitest-reporters/test-balance-reporter.ts`

**What it does**: Monitors test distribution across categories and warns if the pyramid balance is off.

**Target ranges** (warning if outside):
- Unit: 65-85% (target: 70%)
- Integration: 15-30% (target: 20%)
- E2E: 5-15% (target: 10%)

**Example output**:
```
═══════════════════════════════════════════════════════
  Test Pyramid Balance Report
═══════════════════════════════════════════════════════

  Category       Count    Percentage    Target    Status
  ───────────────────────────────────────────────────────
  Unit             90     71.4%    70%       ✓ OK
  Integration      25     19.8%    20%       ✓ OK
  E2E              11      8.7%    10%       ✓ OK
  ───────────────────────────────────────────────────────
  Total           126

  ✓ Test distribution is within acceptable ranges

═══════════════════════════════════════════════════════
```

**When warnings appear**:
```
  Test Balance Warnings:

  ⚠️  Unit test percentage (85.2%) is above target (65-85%)
  ⚠️  Integration test percentage (10.1%) is below target (15-30%)

  Recommendation: Review tests/README.md for test pyramid guidelines
```

**To view**: The reporter runs automatically with every test execution:
```bash
npm test              # See balance report at end
npm test -- --run     # Non-watch mode
```

**Why these tools exist**:
1. **Naming consistency**: Makes it easy to identify and categorize tests
2. **Balance enforcement**: Prevents test suite from becoming top-heavy with slow tests
3. **Maintainability**: Ensures test suite scales sustainably as project grows
4. **Team alignment**: Clear, automated guidelines prevent confusion

**Configuration**:
- ESLint rule: Warning level (won't block commits)
- Vitest reporter: Always runs, informational only
- Both can be temporarily disabled if needed, but this is NOT recommended

---

## Flaky Test Quarantine System

### Overview

The **test quarantine system** identifies and isolates flaky tests (tests that fail intermittently) to prevent them from blocking CI while fixes are developed. Quarantined tests run separately and are tracked for resolution.

**Key principles**:
- 🚫 **Don't block CI**: Flaky tests shouldn't prevent valid code from merging
- 📊 **Track explicitly**: Every quarantined test has a GitHub issue and owner
- ⏱️ **Time-boxed**: Tests auto-remove after 30 days if not fixed
- 📉 **Minimize size**: Quarantine should shrink over time, not grow

### What Makes a Test Flaky?

A test is flaky if it:
- ✅ Passes on retry but fails initially
- 🎲 Has inconsistent results (passes/fails randomly)
- ⏰ Depends on timing or race conditions
- 🌐 Relies on external state not properly isolated

**Common causes**:
- Race conditions in async code
- Timing assumptions (`setTimeout` without proper `waitFor`)
- Improper cleanup between tests
- Shared state pollution
- Network/WebSocket timing issues

### Quarantine Configuration

**File**: `tests/quarantine.json`

```json
{
  "version": "1.0.0",
  "quarantine": [
    {
      "testPath": "src/pages/ConfigPage.test.tsx > ConfigPage > should handle reconnection",
      "reason": "WebSocket mock timing race condition",
      "quarantinedAt": "2026-01-14T00:00:00.000Z",
      "issueUrl": "https://github.com/user/repo/issues/123",
      "failureRate": 0.15,
      "assignee": "developer-name"
    }
  ],
  "resolved": [],
  "metadata": {
    "maxQuarantineSize": 10,
    "alertThreshold": 5,
    "autoRemoveAfterDays": 30
  }
}
```

**Fields explained**:
- `testPath`: Full test path (file > describe > test name)
- `reason`: Why the test is flaky (helps with fix prioritization)
- `quarantinedAt`: When it was quarantined (ISO 8601)
- `issueUrl`: GitHub issue tracking the fix
- `failureRate`: Percentage of runs that fail (0.0 to 1.0)
- `assignee`: Person responsible for fixing the test

### Commands

**Run quarantined tests separately**:
```bash
npm run test:quarantine         # Run once
npm run test:quarantine:watch   # Watch mode
```

**Check quarantine status**:
```bash
npm run test:quarantine:status
```

**Detect new flaky tests**:
```bash
# Run tests with JSON output, then analyze for flaky patterns
npm test -- --reporter=json --outputFile=test-results.json
../scripts/detect-flaky-tests.sh test-results.json
```

### Adding a Test to Quarantine

**Step 1**: Identify the flaky test
```bash
# Test passes on retry = flaky
npm test -- --retry=2
# Check for tests that fail initially but pass on retry
```

**Step 2**: Create a GitHub issue
- Title: "Flaky test: [test name]"
- Description: Failure pattern, logs, suspected cause
- Label: `flaky-test`, `test-infrastructure`

**Step 3**: Add to `tests/quarantine.json`
```json
{
  "testPath": "src/pages/ProfilesPage.test.tsx > ProfilesPage > Edit button renders",
  "reason": "Async rendering race condition in Edit button",
  "quarantinedAt": "2026-01-14T12:00:00.000Z",
  "issueUrl": "https://github.com/user/repo/issues/456",
  "failureRate": 0.20,
  "assignee": "john-doe"
}
```

**Step 4**: Verify quarantine works
```bash
npm test                       # Should skip the test
npm run test:quarantine        # Should run only quarantined tests
```

### Removing a Test from Quarantine

**When the test is fixed**:

1. **Verify the fix**: Run the test 10 times to ensure stability
   ```bash
   for i in {1..10}; do npm test -- ProfilesPage.test.tsx || break; done
   ```

2. **Move to resolved array** in `tests/quarantine.json`:
   ```json
   {
     "resolved": [
       {
         "testPath": "src/pages/ProfilesPage.test.tsx > ProfilesPage > Edit button renders",
         "resolvedAt": "2026-01-15T10:00:00.000Z",
         "resolution": "Fixed async timing with findByRole query"
       }
     ]
   }
   ```

3. **Close the GitHub issue** with reference to the fix PR

4. **Verify in CI**: Ensure test passes consistently in CI environment

### Quarantine Health Monitoring

**Thresholds** (configured in `metadata`):
- `alertThreshold: 5` - Warn when quarantine size exceeds 5 tests
- `maxQuarantineSize: 10` - Critical alert at 10 tests
- `autoRemoveAfterDays: 30` - Auto-remove stale tests after 30 days

**Health status**:
```bash
npm run test:quarantine:status
```

**Example output**:
```
=== Test Quarantine Status ===

Total in quarantine: 3
Alert threshold: 5
Maximum allowed: 10

ℹ️  3 test(s) currently in quarantine

Quarantined tests:

1. src/pages/ConfigPage.test.tsx > ConfigPage > should handle reconnection
   Reason: WebSocket mock timing race condition
   Quarantined: 1/10/2026
   Issue: https://github.com/user/repo/issues/123
   Failure rate: 15.0%
   Assignee: alice

2. src/api/rpc.test.ts > RPC Client > retry logic
   Reason: Mock timing in retry sequence
   Quarantined: 1/12/2026
   Issue: https://github.com/user/repo/issues/124
   Failure rate: 25.0%
   Assignee: bob

✓ 5 test(s) previously resolved

==============================
```

**CI Integration**:
The quarantine system integrates with CI to:
- ✅ Skip quarantined tests in main test runs
- 🔍 Run quarantined tests separately (non-blocking)
- ⚠️ Alert if quarantine size exceeds threshold
- 📊 Report quarantine health in CI artifacts

### Best Practices

**✅ Do**:
- Add tests to quarantine immediately when flakiness is detected
- Always create a GitHub issue with details and assign an owner
- Investigate root cause before quarantining (don't quarantine unnecessarily)
- Set realistic `assignee` and track progress
- Run quarantined tests regularly to check if they can be unquarantined
- Keep quarantine size small (< 5 tests ideal)

**❌ Don't**:
- Use quarantine as a "ignore forever" list
- Add tests without GitHub issues
- Leave tests in quarantine > 30 days without action
- Quarantine tests due to actual bugs (fix the bug, don't quarantine)
- Let quarantine size grow beyond threshold

### Quarantine vs. Test Skipping

| Aspect | Quarantine | `test.skip()` |
|--------|-----------|---------------|
| Purpose | Isolate flaky tests | Temporarily disable broken tests |
| Duration | Tracked, time-boxed | Indefinite, forgotten |
| Tracking | GitHub issue required | No tracking |
| Visibility | Reported in CI | Hidden |
| CI runs | Separate non-blocking | Never runs |
| When to use | Intermittent failures | Code WIP, broken functionality |

**Recommendation**: Use quarantine for flaky tests, use `test.skip()` for intentionally disabled tests during development.

### Troubleshooting

**"Test still runs even though it's quarantined"**
- Check `testPath` matches exactly (file > suite > test)
- Verify `tests/quarantine.json` is valid JSON
- Ensure `RUN_QUARANTINE` env var is not set

**"Quarantined tests aren't running with test:quarantine"**
- Verify `RUN_QUARANTINE=true` is set
- Check quarantine.json has valid entries
- Look for syntax errors in quarantine.json

**"Quarantine health check fails"**
- Review quarantine size vs. thresholds
- Prioritize fixing tests to reduce size
- Consider adjusting thresholds if justified

### Flaky Test Detection Script

**Location**: `scripts/detect-flaky-tests.sh`

**What it does**: Analyzes Vitest JSON output to identify tests that passed on retry, suggesting candidates for quarantine.

**Usage**:
```bash
# Run tests with JSON reporter
cd keyrx_ui
npm test -- --reporter=json --outputFile=test-results.json

# Detect flaky tests
../scripts/detect-flaky-tests.sh test-results.json
```

**Example output**:
```
=== Flaky Test Detection ===

⚠️  Flaky test #1:
   Test: should handle WebSocket reconnection
   File: src/pages/ConfigPage.test.tsx
   Retries needed: 1
   Duration: 1523ms

Found 1 flaky test(s)

Recommendation:
  1. Add these tests to keyrx_ui/tests/quarantine.json
  2. Create GitHub issues to track fixes
  3. Run quarantined tests separately: npm run test:quarantine
```

### Technical Implementation

**Architecture**:
1. `tests/quarantine-manager.ts` - Core quarantine logic (load, validate, query)
2. `vitest-plugins/quarantine-plugin.ts` - Vitest plugin integration
3. `src/test/setup.ts` - Test setup hook to skip quarantined tests
4. `scripts/detect-flaky-tests.sh` - Automated flaky test detection

**How it works**:
1. Test setup reads `quarantine.json` before each test
2. Compares current test path against quarantine list
3. Calls `context.skip()` if test is quarantined (unless `RUN_QUARANTINE=true`)
4. Vitest plugin prints quarantine health status
5. Quarantined tests run separately with `test:quarantine` script

**Integration points**:
- `vitest.config.base.ts` - Loads quarantine plugin
- `package.json` - Defines quarantine commands
- CI workflow - Runs quarantined tests as separate job

---

## References

- [Vitest Documentation](https://vitest.dev/)
- [Playwright Documentation](https://playwright.dev/)
- [React Testing Library](https://testing-library.com/react)
- [Test Pyramid Pattern](https://martinfowler.com/articles/practical-test-pyramid.html)
- WebSocket Testing Guide: `tests/WEBSOCKET_TESTING.md`
- Accessibility Testing: `tests/a11y/README.md`

---

**Last Updated**: 2026-01-14
**Maintained By**: Test Infrastructure Team
