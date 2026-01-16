/**
 * Accessibility tests for ConfigPage
 *
 * WCAG 2.2 Level AA compliance verification
 * Requirements: Task 16 (Requirement 4.1)
 */

import { describe, test, expect, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/react';
import {
  renderWithProviders,
  setupMockWebSocket,
  cleanupMockWebSocket,
  simulateConnected,
} from '../../tests/testUtils';
import {
  runA11yAudit,
  runCompleteA11yAudit,
} from '../../tests/AccessibilityTestHelper';
import ConfigPage from './ConfigPage';

describe('ConfigPage Accessibility', () => {
  beforeEach(async () => {
    await setupMockWebSocket();
  });

  afterEach(() => {
    cleanupMockWebSocket();
  });

  test('should have no WCAG 2.2 Level AA violations', async () => {
    const { container } = renderWithProviders(<ConfigPage />, {
      wrapWithRouter: true,
    });

    // Small delay to let component initialize
    await new Promise((resolve) => setTimeout(resolve, 100));

    const results = await runA11yAudit(container);
    expect(results).toHaveNoViolations();
  });

  test('should pass complete accessibility audit', async () => {
    const { container } = renderWithProviders(<ConfigPage />, {
      wrapWithRouter: true,
    });

    // Small delay to let component initialize
    await new Promise((resolve) => setTimeout(resolve, 100));

    const results = await runCompleteA11yAudit(container);

    // WCAG 2.2 Level AA compliance
    expect(results.wcag22).toHaveNoViolations();

    // Color contrast compliance (WCAG 1.4.3)
    expect(results.colorContrast).toHaveNoViolations();

    // Keyboard accessibility (WCAG 2.1.1, 2.1.2, 2.4.7)
    expect(results.keyboard).toHaveNoViolations();

    // ARIA and semantic HTML (WCAG 4.1.2)
    expect(results.aria).toHaveNoViolations();
  });

  test('should have proper page structure with landmarks', async () => {
    const { container } = renderWithProviders(<ConfigPage />, {
      wrapWithRouter: true,
    });

    // Small delay to let component initialize
    await new Promise((resolve) => setTimeout(resolve, 100));

    // Verify semantic HTML landmarks exist
    const main = container.querySelector('main');
    expect(main).toBeTruthy();
  });

  test('should have descriptive page title or heading', async () => {
    const { container } = renderWithProviders(<ConfigPage />, {
      wrapWithRouter: true,
    });

    // Small delay to let component initialize
    await new Promise((resolve) => setTimeout(resolve, 100));

    // Look for h1 or h2 heading
    const heading = container.querySelector('h1, h2');
    expect(heading).toBeTruthy();
    expect(heading?.textContent).toBeTruthy();
  });
});
