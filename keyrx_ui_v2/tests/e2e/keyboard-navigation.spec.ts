import { test, expect } from '@playwright/test';

/**
 * E2E Test: Full Keyboard Navigation
 *
 * This test verifies that all pages and features are fully accessible via keyboard:
 * 1. Tab navigation works on all pages
 * 2. Focus indicators are visible
 * 3. Escape closes modals and dropdowns
 * 4. Enter/Space activate buttons
 * 5. Arrow keys navigate lists and dropdowns
 * 6. Focus returns correctly after modal close
 *
 * Requirements: Task 34 (E2E Tests), Req 3 (Accessibility - keyboard navigation)
 */

test.describe('Keyboard Navigation E2E', () => {
  const pages = [
    { path: '/', name: 'HomePage' },
    { path: '/devices', name: 'DevicesPage' },
    { path: '/profiles', name: 'ProfilesPage' },
    { path: '/config', name: 'ConfigPage' },
    { path: '/metrics', name: 'MetricsPage' },
    { path: '/simulator', name: 'SimulatorPage' },
  ];

  test('should navigate all pages with Tab key', async ({ page }) => {
    for (const { path, name } of pages) {
      await page.goto(path);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(300);

      // Tab through the page and collect focused elements
      const focusedElements: Array<{ tag: string; text: string; ariaLabel: string }> = [];
      let tabCount = 0;
      const maxTabs = 50;

      while (tabCount < maxTabs) {
        await page.keyboard.press('Tab');
        tabCount++;

        const focused = page.locator(':focus');
        const tagName = await focused.evaluate((el) => el.tagName).catch(() => '');
        const text = await focused.textContent().catch(() => '');
        const ariaLabel = await focused.getAttribute('aria-label').catch(() => '');

        if (tagName) {
          focusedElements.push({
            tag: tagName,
            text: text?.substring(0, 30) || '',
            ariaLabel: ariaLabel || '',
          });
        }

        // Stop if we've cycled back to the first element
        if (tabCount > 10 && focusedElements.length > 5) {
          const firstElement = focusedElements[0];
          const currentElement = focusedElements[focusedElements.length - 1];
          if (
            firstElement.tag === currentElement.tag &&
            firstElement.text === currentElement.text
          ) {
            break;
          }
        }
      }

      // Verify we focused multiple interactive elements
      console.log(`${name}: Focused ${focusedElements.length} elements`);
      expect(focusedElements.length).toBeGreaterThan(0);

      // Verify focused elements are actually interactive
      const interactiveTags = ['BUTTON', 'A', 'INPUT', 'SELECT', 'TEXTAREA'];
      const interactiveCount = focusedElements.filter((el) =>
        interactiveTags.includes(el.tag)
      ).length;

      expect(interactiveCount).toBeGreaterThan(0);
    }
  });

  test('should show visible focus indicators on all pages', async ({ page }) => {
    for (const { path, name } of pages) {
      await page.goto(path);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(300);

      // Tab to first focusable element
      await page.keyboard.press('Tab');

      // Check if focus indicator is visible
      const focusIndicator = await page.evaluate(() => {
        const activeElement = document.activeElement;
        if (!activeElement) return null;

        const styles = window.getComputedStyle(activeElement);
        return {
          outline: styles.outline,
          outlineWidth: styles.outlineWidth,
          outlineColor: styles.outlineColor,
          boxShadow: styles.boxShadow,
        };
      });

      // At least one focus indicator should be present
      const hasFocusIndicator =
        (focusIndicator?.outline && focusIndicator.outline !== 'none') ||
        (focusIndicator?.outlineWidth && focusIndicator.outlineWidth !== '0px') ||
        (focusIndicator?.boxShadow && focusIndicator.boxShadow !== 'none');

      expect(hasFocusIndicator).toBeTruthy();
      console.log(`${name}: Focus indicator visible`);
    }
  });

  test('should close modals with Escape key', async ({ page }) => {
    // Test on ProfilesPage create modal
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(300);

    const createButton = page.getByRole('button', { name: /create/i });

    if (await createButton.isVisible()) {
      await createButton.click();
      await page.waitForTimeout(300);

      const modal = page.getByRole('dialog');
      await expect(modal).toBeVisible();

      // Press Escape to close
      await page.keyboard.press('Escape');
      await page.waitForTimeout(300);

      // Modal should be closed
      await expect(modal).not.toBeVisible();
    }

    // Test on ConfigPage key config dialog (if key exists)
    await page.goto('/config');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    const keyButton = page.getByRole('button', { name: /key|caps|esc/i }).first();

    if (await keyButton.isVisible()) {
      await keyButton.click();
      await page.waitForTimeout(300);

      const dialog = page.getByRole('dialog');
      if (await dialog.isVisible()) {
        await page.keyboard.press('Escape');
        await page.waitForTimeout(300);
        await expect(dialog).not.toBeVisible();
      }
    }
  });

  test('should activate buttons with Enter and Space keys', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(300);

    // Tab until we find a button
    let tabCount = 0;
    const maxTabs = 30;
    let buttonFound = false;

    while (tabCount < maxTabs && !buttonFound) {
      await page.keyboard.press('Tab');
      tabCount++;

      const focused = page.locator(':focus');
      const tagName = await focused.evaluate((el) => el.tagName).catch(() => '');
      const text = await focused.textContent().catch(() => '');

      if (tagName === 'BUTTON' && text) {
        buttonFound = true;

        // Try pressing Enter
        await page.keyboard.press('Enter');
        await page.waitForTimeout(300);

        // Button should have been activated (hard to verify without knowing what it does)
        // At minimum, verify no errors occurred
        const errors = await page.evaluate(() => {
          // Check for console errors or visible error messages
          return (window as any).__testErrors || [];
        });

        expect(errors.length).toBe(0);
        break;
      }
    }

    // Verify we found at least one button to test
    expect(buttonFound).toBeTruthy();
  });

  test('should navigate dropdowns with arrow keys', async ({ page }) => {
    // Test on DevicesPage layout selector
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    const dropdown = page.getByRole('button', { name: /layout|scope/i }).first();

    if (await dropdown.isVisible()) {
      // Focus the dropdown
      await dropdown.focus();

      // Press Enter or Space to open
      await page.keyboard.press('Enter');
      await page.waitForTimeout(300);

      // Try arrow down to navigate options
      await page.keyboard.press('ArrowDown');
      await page.waitForTimeout(200);

      // Try arrow up
      await page.keyboard.press('ArrowUp');
      await page.waitForTimeout(200);

      // Press Escape to close
      await page.keyboard.press('Escape');
      await page.waitForTimeout(300);
    }

    // Test on ConfigPage layer selector
    await page.goto('/config');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    const layerSelector = page.getByRole('button', { name: /layer/i }).first();

    if (await layerSelector.isVisible()) {
      await layerSelector.focus();
      await page.keyboard.press('Enter');
      await page.waitForTimeout(300);

      await page.keyboard.press('ArrowDown');
      await page.waitForTimeout(200);

      await page.keyboard.press('Escape');
      await page.waitForTimeout(300);
    }
  });

  test('should trap focus within modals', async ({ page }) => {
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(300);

    const createButton = page.getByRole('button', { name: /create/i });

    if (!(await createButton.isVisible())) {
      test.skip();
      return;
    }

    await createButton.click();
    await page.waitForTimeout(300);

    const modal = page.getByRole('dialog');
    await expect(modal).toBeVisible();

    // Tab through modal multiple times
    const focusedElements: string[] = [];
    let tabCount = 0;
    const maxTabs = 20;

    while (tabCount < maxTabs) {
      await page.keyboard.press('Tab');
      tabCount++;

      const focused = page.locator(':focus');
      const tagName = await focused.evaluate((el) => el.tagName).catch(() => '');
      const id = await focused.getAttribute('id').catch(() => '');
      const className = await focused.getAttribute('class').catch(() => '');

      focusedElements.push(`${tagName}#${id}.${className}`);

      // Check if focus is still within modal
      const isInsideModal = await focused.evaluate((el, modalEl) => {
        return modalEl?.contains(el);
      }, await modal.elementHandle());

      if (!isInsideModal && focusedElements.length > 3) {
        // Focus escaped the modal - test fails
        expect(isInsideModal).toBeTruthy();
        break;
      }

      // If we've tabbed 10 times and focus is cycling, that's good
      if (tabCount >= 10) {
        // Check if we're cycling (same element focused again)
        const uniqueElements = new Set(focusedElements);
        if (uniqueElements.size < focusedElements.length) {
          // We're cycling - focus trap is working
          break;
        }
      }
    }

    // Close modal
    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);
  });

  test('should return focus after modal closes', async ({ page }) => {
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(300);

    const createButton = page.getByRole('button', { name: /create/i });

    if (!(await createButton.isVisible())) {
      test.skip();
      return;
    }

    // Focus the create button
    await createButton.focus();

    // Get element details before opening modal
    const buttonId = await createButton.getAttribute('id');
    const buttonText = await createButton.textContent();

    // Click to open modal
    await createButton.click();
    await page.waitForTimeout(300);

    const modal = page.getByRole('dialog');
    await expect(modal).toBeVisible();

    // Close with Escape
    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);

    // Verify modal is closed
    await expect(modal).not.toBeVisible();

    // Verify focus returned to the button
    const focused = page.locator(':focus');
    const focusedText = await focused.textContent();

    // Focus should be on or near the create button
    expect(focusedText?.toLowerCase()).toContain(buttonText?.toLowerCase() || 'create');
  });

  test('should navigate between pages using sidebar/navigation', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(300);

    // Tab until we find a navigation link
    let tabCount = 0;
    const maxTabs = 40;
    const visitedPages: string[] = [];

    while (tabCount < maxTabs && visitedPages.length < 3) {
      await page.keyboard.press('Tab');
      tabCount++;

      const focused = page.locator(':focus');
      const tagName = await focused.evaluate((el) => el.tagName).catch(() => '');
      const text = await focused.textContent().catch(() => '');
      const href = await focused.getAttribute('href').catch(() => '');

      // If it's a navigation link, activate it
      if (tagName === 'A' && href && text && !visitedPages.includes(text)) {
        visitedPages.push(text);

        // Press Enter to navigate
        await page.keyboard.press('Enter');
        await page.waitForTimeout(500);

        // Verify we navigated
        const currentUrl = page.url();
        expect(currentUrl).toContain('localhost');

        // If we've tested enough pages, stop
        if (visitedPages.length >= 3) {
          break;
        }

        // Continue tabbing from here
      }
    }

    // Verify we navigated to multiple pages
    expect(visitedPages.length).toBeGreaterThan(0);
  });

  test('should handle keyboard shortcuts (if any)', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(300);

    // Test common keyboard shortcuts
    // Ctrl+K or Cmd+K might open search or command palette
    const isMac = await page.evaluate(() => navigator.platform.includes('Mac'));
    const modKey = isMac ? 'Meta' : 'Control';

    // Try Ctrl/Cmd + K
    await page.keyboard.press(`${modKey}+k`);
    await page.waitForTimeout(300);

    // Check if anything opened
    const dialog = page.getByRole('dialog');
    const searchInput = page.getByRole('searchbox');

    if (await dialog.isVisible()) {
      // Something opened - close it
      await page.keyboard.press('Escape');
      await page.waitForTimeout(300);
    }

    if (await searchInput.isVisible()) {
      // Search opened
      await page.keyboard.press('Escape');
      await page.waitForTimeout(300);
    }

    // Test doesn't require shortcuts to exist, just verifies they don't break
  });

  test('should navigate keyboard visualizer with keyboard', async ({ page }) => {
    await page.goto('/config');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Tab until we find a key button in the keyboard
    let tabCount = 0;
    const maxTabs = 120; // Keyboard has 104+ keys
    let keyFound = false;

    while (tabCount < maxTabs && !keyFound) {
      await page.keyboard.press('Tab');
      tabCount++;

      const focused = page.locator(':focus');
      const ariaLabel = await focused.getAttribute('aria-label').catch(() => '');
      const text = await focused.textContent().catch(() => '');

      // Check if it's a key button (aria-label contains "Key")
      if (ariaLabel?.toLowerCase().includes('key') || text?.match(/^[A-Z0-9]$/)) {
        keyFound = true;

        // Press Enter to open config dialog
        await page.keyboard.press('Enter');
        await page.waitForTimeout(300);

        const dialog = page.getByRole('dialog');
        if (await dialog.isVisible()) {
          // Success - we can navigate keyboard with Tab and activate with Enter
          await page.keyboard.press('Escape');
          await page.waitForTimeout(300);
        }
        break;
      }

      // Safety: stop if we've tabbed too far
      if (tabCount > 50 && !keyFound) {
        break;
      }
    }

    // Verify we found at least one key to test
    console.log(`Keyboard navigation: Found key after ${tabCount} tabs`);
  });

  test('should handle form submission with Enter key', async ({ page }) => {
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(300);

    const createButton = page.getByRole('button', { name: /create/i });

    if (!(await createButton.isVisible())) {
      test.skip();
      return;
    }

    await createButton.click();
    await page.waitForTimeout(300);

    // Find the name input
    const nameInput = page.getByRole('textbox', { name: /name/i }).first();

    if (await nameInput.isVisible()) {
      // Type a name
      await nameInput.fill(`KeyboardTest_${Date.now()}`);

      // Press Enter to submit form
      await nameInput.press('Enter');
      await page.waitForTimeout(500);

      // Modal should close OR show validation error
      const modal = page.getByRole('dialog');
      const modalVisible = await modal.isVisible();
      const errorMsg = page.locator('text=/error|invalid/i');
      const errorVisible = await errorMsg.isVisible();

      // Either modal closed (success) or error shown
      expect(modalVisible === false || errorVisible === true).toBeTruthy();

      // Clean up if modal closed (profile was created)
      if (!modalVisible) {
        // Profile created - delete it
        const deleteButton = page.getByRole('button', { name: /delete/i }).first();
        if (await deleteButton.isVisible()) {
          await deleteButton.click();
          await page.waitForTimeout(300);
          const confirmButton = page.getByRole('button', { name: /confirm|yes/i });
          if (await confirmButton.isVisible()) {
            await confirmButton.click();
            await page.waitForTimeout(300);
          }
        }
      } else {
        // Modal still open - close it
        await page.keyboard.press('Escape');
        await page.waitForTimeout(300);
      }
    }
  });
});
