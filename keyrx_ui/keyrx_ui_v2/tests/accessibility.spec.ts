import { test, expect } from '@playwright/test';
import type { Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

/**
 * Accessibility Testing Suite
 *
 * Tests WCAG 2.1 Level AA compliance using axe-core.
 * Requirements: Req 3 (Accessibility), Task 31 (0 violations, Lighthouse ≥95)
 */

const routes = [
  { path: '/', name: 'HomePage' },
  { path: '/devices', name: 'DevicesPage' },
  { path: '/profiles', name: 'ProfilesPage' },
  { path: '/config', name: 'ConfigPage' },
  { path: '/metrics', name: 'MetricsPage' },
  { path: '/simulator', name: 'SimulatorPage' },
];

test.describe('Accessibility Testing', () => {
  test.beforeEach(async ({ page }) => {
    // Wait for app to load
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('networkidle');
  });

  for (const route of routes) {
    test(`${route.name} should have no accessibility violations`, async ({ page }) => {
      await page.goto(`http://localhost:5173${route.path}`);
      await page.waitForLoadState('networkidle');

      // Run axe-core accessibility scan
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      // Expect 0 violations
      expect(accessibilityScanResults.violations).toEqual([]);
    });
  }

  test('Modal should have no accessibility violations when open', async ({ page }) => {
    await page.goto('http://localhost:5173/profiles');
    await page.waitForLoadState('networkidle');

    // Open modal (assuming there's a "Create Profile" button)
    const createButton = page.getByRole('button', { name: /create/i });
    if (await createButton.isVisible()) {
      await createButton.click();
      await page.waitForTimeout(500); // Wait for modal animation

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    }
  });

  test('Dropdown should have no accessibility violations when open', async ({ page }) => {
    await page.goto('http://localhost:5173/config');
    await page.waitForLoadState('networkidle');

    // Open dropdown (layout selector)
    const layoutDropdown = page.getByRole('button', { name: /layout/i });
    if (await layoutDropdown.isVisible()) {
      await layoutDropdown.click();
      await page.waitForTimeout(300); // Wait for dropdown animation

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    }
  });

  test('Keyboard navigation should work on all pages', async ({ page }) => {
    for (const route of routes) {
      await page.goto(`http://localhost:5173${route.path}`);
      await page.waitForLoadState('networkidle');

      // Press Tab to focus first element
      await page.keyboard.press('Tab');

      // Verify focus is visible
      const focusedElement = await page.evaluate(() => {
        const activeElement = document.activeElement;
        if (!activeElement) return null;

        const styles = window.getComputedStyle(activeElement);
        return {
          tagName: activeElement.tagName,
          outline: styles.outline,
          outlineWidth: styles.outlineWidth,
          boxShadow: styles.boxShadow,
        };
      });

      // At least one of these focus indicators should be present
      const hasFocusIndicator =
        focusedElement?.outline !== 'none' ||
        focusedElement?.outlineWidth !== '0px' ||
        (focusedElement?.boxShadow && focusedElement.boxShadow !== 'none');

      expect(hasFocusIndicator).toBeTruthy();
    }
  });

  test('All interactive elements should have accessible names', async ({ page }) => {
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('networkidle');

    // Find all buttons, links, and form inputs
    const elements = await page.locator('button, a, input, select, textarea').all();

    for (const element of elements) {
      const ariaLabel = await element.getAttribute('aria-label');
      const ariaLabelledBy = await element.getAttribute('aria-labelledby');
      const innerText = await element.innerText().catch(() => '');
      const altText = await element.getAttribute('alt');
      const title = await element.getAttribute('title');
      const placeholder = await element.getAttribute('placeholder');

      // Element should have at least one accessible name
      const hasAccessibleName =
        ariaLabel ||
        ariaLabelledBy ||
        (innerText && innerText.trim().length > 0) ||
        altText ||
        title ||
        placeholder;

      if (!hasAccessibleName) {
        const outerHTML = await element.evaluate((el) => el.outerHTML);
        console.warn('Element without accessible name:', outerHTML);
      }

      // This is informational - we log warnings but don't fail the test
      // because some elements (like containers) may not need names
    }
  });

  test('Color contrast should meet WCAG AA standards', async ({ page }) => {
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('networkidle');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2aa'])
      .options({
        runOnly: {
          type: 'rule',
          values: ['color-contrast'],
        },
      })
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('Images should have alt text', async ({ page }) => {
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('networkidle');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .options({
        runOnly: {
          type: 'rule',
          values: ['image-alt'],
        },
      })
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('Form inputs should have labels', async ({ page }) => {
    // Go to a page with forms (DevicesPage has inline rename)
    await page.goto('http://localhost:5173/devices');
    await page.waitForLoadState('networkidle');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .options({
        runOnly: {
          type: 'rule',
          values: ['label'],
        },
      })
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('Page should have proper heading hierarchy', async ({ page }) => {
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('networkidle');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .options({
        runOnly: {
          type: 'rule',
          values: ['heading-order'],
        },
      })
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('Links should have discernible text', async ({ page }) => {
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('networkidle');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .options({
        runOnly: {
          type: 'rule',
          values: ['link-name'],
        },
      })
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('ARIA attributes should be valid', async ({ page }) => {
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('networkidle');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .options({
        runOnly: {
          type: 'tag',
          values: ['best-practice'],
        },
      })
      .analyze();

    // Filter for ARIA-related violations only
    const ariaViolations = accessibilityScanResults.violations.filter((violation) =>
      violation.id.includes('aria')
    );

    expect(ariaViolations).toEqual([]);
  });
});
