# Visual Regression Tests

This directory contains visual regression tests for the KeyRx UI. These tests capture screenshots of all pages at different viewport sizes and compare them against baseline images to detect unintended visual changes.

## What Gets Tested

### All Pages at 3 Breakpoints
- **Mobile** (375x667) - iPhone SE size
- **Tablet** (768x1024) - iPad size
- **Desktop** (1280x720) - Typical laptop size

Pages tested:
- Home page
- Devices page
- Profiles page
- Config page
- Metrics page
- Simulator page

### Additional Test Coverage
- Interactive states (pages with data loaded)
- Modal and dialog states
- Responsive layout transitions (sidebar → bottom nav)
- Theme consistency across all pages
- Navigation component rendering

## Running Visual Regression Tests

### Run All Visual Tests
```bash
npm run test:e2e -- tests/visual/
```

### Run Only Chromium (Faster)
```bash
npx playwright test tests/visual/ --project=chromium
```

### Run All Browsers
The tests will run on:
- Chromium (Chrome)
- Firefox
- WebKit (Safari)
- Mobile Chrome
- Mobile Safari
- iPad

## Updating Baseline Screenshots

When you intentionally change the UI design, you need to update the baseline screenshots:

```bash
npx playwright test tests/visual/ --update-snapshots
```

**⚠️ Warning:** Only update baselines after visually verifying the changes are correct!

## Baseline Screenshot Location

Baseline screenshots are stored in:
```
tests/visual/visual-regression.spec.ts-snapshots/
```

These files are committed to git and serve as the "source of truth" for what the UI should look like.

## Screenshot Comparison Tolerance

The tests allow for minor pixel differences to account for:
- Font rendering variations across platforms
- Anti-aliasing differences
- Minor browser quirks

Current thresholds:
- **maxDiffPixels**: 100-200 pixels (depending on test)
- **threshold**: 0.2 (20% color difference tolerance)

## What Happens When Tests Fail

If a visual regression test fails:

1. **Check the HTML report**:
   ```bash
   npx playwright show-report
   ```

2. **Review the diff images** in the report to see what changed

3. **Determine if the change is intentional**:
   - ✅ Intentional: Update baselines with `--update-snapshots`
   - ❌ Unintentional: Fix the CSS/component causing the regression

## CI/CD Integration

Visual regression tests run automatically in CI on every pull request. If they fail:

1. Download the test artifacts from the CI run
2. Review the diff images
3. Either fix the issue or update the baselines (and commit them)

## Tips for Maintaining Visual Tests

### Disable Animations
The tests automatically disable CSS animations using `animations: 'disabled'` to ensure consistent screenshots.

### Wait for Network Idle
Tests wait for `networkidle` state to ensure all content is loaded before taking screenshots.

### Full Page Screenshots
Most tests use `fullPage: true` to capture the entire page, not just the viewport.

### Test Data Stability
Visual tests work best with stable, predictable test data. Avoid:
- Timestamps that change on every run
- Random data
- Dynamic content from external APIs

## Debugging Failed Visual Tests

If tests fail unexpectedly:

1. **Run tests in headed mode** to see what's happening:
   ```bash
   npx playwright test tests/visual/ --headed
   ```

2. **Check for timing issues**: Increase wait times if content loads slowly

3. **Verify baseline screenshots**: Make sure they weren't corrupted

4. **Check for environment differences**: Font rendering can vary between OS versions

## Snapshot File Naming

Snapshots follow this naming pattern:
```
{page}-{viewport}-{browser}-{platform}.png
```

Examples:
- `home-desktop-chromium-linux.png`
- `config-mobile-firefox-darwin.png`
- `profiles-tablet-webkit-linux.png`

## Related Documentation

- [Playwright Visual Comparisons](https://playwright.dev/docs/test-snapshots)
- [E2E Tests](../e2e/README.md) - End-to-end functional tests
- [Accessibility Tests](../accessibility.spec.ts) - A11y compliance tests
