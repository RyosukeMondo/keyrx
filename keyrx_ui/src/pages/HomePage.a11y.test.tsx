/**
 * Accessibility tests for HomePage
 *
 * WCAG 2.2 Level AA compliance verification
 */

import { describe, test, expect, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/react';
import {
  renderWithProviders,
  setupMockWebSocket,
  cleanupMockWebSocket,
} from '../../tests/testUtils';
import {
  runA11yAudit,
} from '../../tests/AccessibilityTestHelper';
import { HomePage } from './HomePage';

describe('HomePage Accessibility', () => {
  beforeEach(async () => {
    await setupMockWebSocket();
  });

  afterEach(() => {
    cleanupMockWebSocket();
  });

  test('should have no WCAG 2.2 Level AA violations', async () => {
    const { container } = renderWithProviders(<HomePage />, {
      wrapWithRouter: true,
    });

    await new Promise((resolve) => setTimeout(resolve, 100));

    const results = await runA11yAudit(container);
    expect(results).toHaveNoViolations();
  });

  test('should have proper page structure with landmarks', async () => {
    const { container } = renderWithProviders(<HomePage />, {
      wrapWithRouter: true,
    });

    await new Promise((resolve) => setTimeout(resolve, 100));

    const main = container.querySelector('main');
    expect(main).toBeTruthy();
  });

  test('should have descriptive page title or heading', async () => {
    const { container } = renderWithProviders(<HomePage />, {
      wrapWithRouter: true,
    });

    await new Promise((resolve) => setTimeout(resolve, 100));

    const heading = container.querySelector('h1, h2');
    expect(heading).toBeTruthy();
    expect(heading?.textContent).toBeTruthy();
  });
});
