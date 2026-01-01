# Visual Regression Testing

This directory contains visual regression tests for the KeyRx UI using Playwright's screenshot comparison feature.

## Overview

Visual regression tests capture screenshots of pages at different viewport sizes and compare them against baseline images. This helps prevent unintended visual changes from being introduced.

## Viewports Tested

- **Mobile**: 375x667px (iPhone SE)
- **Tablet**: 768x1024px (iPad)
- **Desktop**: 1024x768px

## Pages Tested

1. **Profiles Page** (`/`)
2. **Config Page - Visual Tab** (`/config/Default`)
3. **Config Page - Code Tab** (with Monaco editor)
4. **Dashboard Page** (`/dashboard`)
5. **Devices Page** (`/devices`)

## Running Visual Tests

### First Time Setup - Generate Baselines

When running for the first time, you need to generate baseline images:

```bash
# Generate baselines for all browsers
npm run test:e2e -- tests/e2e/visual/responsive.spec.ts --update-snapshots

# Or for a specific browser
npx playwright test tests/e2e/visual/responsive.spec.ts --project=chromium --update-snapshots
```

This will create baseline images in:
- `tests/e2e/visual/responsive.spec.ts-snapshots/`

### Running Visual Regression Tests

```bash
# Run visual tests only
npx playwright test tests/e2e/visual/responsive.spec.ts

# Run with UI mode for debugging
npx playwright test tests/e2e/visual/responsive.spec.ts --ui

# Run for specific viewport
npx playwright test tests/e2e/visual/responsive.spec.ts --grep "mobile"
```

### Reviewing Failed Tests

When a test fails due to visual differences:

1. Playwright will show the diff in the HTML report:
   ```bash
   npx playwright show-report
   ```

2. Review the differences:
   - **Expected**: Original baseline
   - **Actual**: Current screenshot
   - **Diff**: Highlighted differences

3. If the change is intentional, update the baseline:
   ```bash
   npx playwright test tests/e2e/visual/responsive.spec.ts --update-snapshots
   ```

## CI Integration

Visual regression tests run automatically in CI. If a test fails:

1. Download the test artifacts from the CI run
2. Review the diff images
3. If the changes are intentional:
   - Update baselines locally
   - Commit the new baseline images
   - Push to trigger CI again

## Baseline Management

### Baseline Location

Baselines are stored in:
```
tests/e2e/visual/responsive.spec.ts-snapshots/
├── chromium/
├── firefox/
└── webkit/
```

Each browser has its own baselines because rendering can differ slightly.

### When to Update Baselines

Update baselines when:
- ✅ You intentionally changed the UI design
- ✅ You fixed a visual bug
- ✅ You updated component styling
- ❌ Don't update to "make tests pass" without reviewing changes

### Best Practices

1. **Commit Baselines**: Always commit baseline images to git
2. **Review Diffs**: Never update baselines without reviewing the visual diff
3. **Browser-Specific**: Keep separate baselines per browser
4. **Stable Environment**: Generate baselines in a consistent environment (same OS, browser versions)
5. **Test in CI**: Ensure visual tests pass in CI before merging

## Troubleshooting

### Flaky Tests

If tests are flaky due to animations or loading states:
- Screenshots already disable animations with `animations: 'disabled'`
- Added `waitForPageStable()` helper to wait for network idle
- Increase wait times if needed

### Different Rendering

If screenshots differ across machines:
- Use Docker for consistent environment
- Run in CI for canonical baseline
- Check browser versions match

### Font Rendering

Font rendering can differ slightly across platforms:
- Playwright uses consistent font rendering per browser
- Stick to one platform for generating baselines (CI)
- Acceptable threshold is configured in Playwright config

## Test Configuration

Visual tests use these Playwright config options:

```typescript
use: {
  screenshot: 'only-on-failure',
  video: 'retain-on-failure',
}
```

Screenshot comparison threshold can be adjusted in `playwright.config.ts`:

```typescript
expect: {
  toMatchSnapshot: {
    maxDiffPixels: 100, // Allow small differences
  }
}
```
