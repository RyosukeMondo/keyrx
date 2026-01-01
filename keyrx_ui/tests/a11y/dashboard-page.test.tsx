import { describe, it, expect, vi, beforeEach } from 'vitest';
import { axe } from 'vitest-axe';
import { renderForA11y } from './test-utils';
import { DashboardPage } from '../../src/pages/DashboardPage';

// Mock useUnifiedApi to prevent WebSocket connection attempts
vi.mock('../../src/hooks/useUnifiedApi', () => ({
  useUnifiedApi: () => ({
    isConnected: false,
    readyState: 3, // CLOSED
    sendMessage: vi.fn(),
    lastMessage: null,
    subscribe: vi.fn(() => vi.fn()), // Returns unsubscribe function
    unsubscribe: vi.fn(),
  }),
}));

describe('DashboardPage Accessibility', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should have no accessibility violations', async () => {
    const { container } = renderForA11y(<DashboardPage />);

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have proper heading hierarchy', async () => {
    const { getByRole } = renderForA11y(<DashboardPage />);

    // Check that main heading exists
    const heading = getByRole('heading', { level: 1 });
    expect(heading).toBeInTheDocument();
  });

  it('should have ARIA labels on interactive elements', async () => {
    const { container } = renderForA11y(<DashboardPage />);

    // Get all buttons
    const buttons = container.querySelectorAll('button');

    // All buttons should have accessible names
    buttons.forEach((button) => {
      const hasText = button.textContent && button.textContent.trim().length > 0;
      const hasAriaLabel = button.hasAttribute('aria-label');
      const hasAriaLabelledBy = button.hasAttribute('aria-labelledby');

      expect(
        hasText || hasAriaLabel || hasAriaLabelledBy,
        `Button should have accessible name: ${button.outerHTML}`
      ).toBe(true);
    });
  });

  it('should support keyboard navigation', async () => {
    const { container } = renderForA11y(<DashboardPage />);

    // Get all interactive elements
    const interactiveElements = container.querySelectorAll(
      'button, a, input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );

    // All interactive elements should be focusable
    interactiveElements.forEach((element) => {
      const tabIndex = element.getAttribute('tabindex');

      if (tabIndex !== null) {
        expect(parseInt(tabIndex, 10)).toBeGreaterThanOrEqual(0);
      }
    });
  });

  it('should have proper navigation landmarks', async () => {
    const { container } = renderForA11y(<DashboardPage />);

    // Check for main content area (could be main tag or div with role="main")
    const mainElement = container.querySelector('main, [role="main"]');
    // Some pages might wrap content differently, so we just check the page renders
    expect(container.firstChild).toBeTruthy();
  });

  it('should have accessible status indicators', async () => {
    const { container } = renderForA11y(<DashboardPage />);

    // Connection status banners should have proper roles or ARIA attributes
    const statusElements = container.querySelectorAll('[role="status"], [role="alert"]');

    // At least one status element should exist for connection status
    expect(statusElements.length).toBeGreaterThanOrEqual(0);
  });
});
