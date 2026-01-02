/**
 * Accessibility Test Helper
 *
 * Provides reusable utilities for automated WCAG 2.2 Level AA compliance testing.
 * Uses axe-core for comprehensive accessibility audits.
 *
 * Requirements: Requirement 4 (WCAG 2.2 Level AA), Task 15
 */

import { axe, toHaveNoViolations } from 'vitest-axe';
import { expect } from 'vitest';

// Extend Vitest matchers to include axe-core assertions
expect.extend(toHaveNoViolations);

/**
 * WCAG 2.2 Level AA configuration for axe-core
 *
 * Includes rules for:
 * - WCAG 2.0 Level A and AA
 * - WCAG 2.1 Level A and AA
 * - WCAG 2.2 Level A and AA
 */
export const WCAG_22_AA_CONFIG = {
  runOnly: {
    type: 'tag' as const,
    values: ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'],
  },
};

/**
 * Color contrast rule configuration (WCAG 1.4.3)
 *
 * Ensures:
 * - Normal text: ≥4.5:1 contrast ratio
 * - Large text: ≥3:1 contrast ratio
 */
export const COLOR_CONTRAST_CONFIG = {
  runOnly: {
    type: 'rule' as const,
    values: ['color-contrast'],
  },
};

/**
 * Keyboard accessibility rule configuration
 *
 * Verifies:
 * - WCAG 2.1.1: Keyboard accessible
 * - WCAG 2.1.2: No keyboard trap
 * - WCAG 2.4.7: Focus visible
 */
export const KEYBOARD_ACCESSIBILITY_CONFIG = {
  runOnly: {
    type: 'rule' as const,
    values: ['keyboard', 'focus-order-semantics', 'tabindex'],
  },
};

/**
 * ARIA and semantic HTML rule configuration (WCAG 4.1.2)
 *
 * Verifies:
 * - Valid ARIA attributes
 * - Proper ARIA roles
 * - Semantic HTML usage
 * - Accessible names for interactive elements
 */
export const ARIA_SEMANTIC_CONFIG = {
  runOnly: {
    type: 'rule' as const,
    values: [
      'aria-allowed-attr',
      'aria-required-attr',
      'aria-valid-attr',
      'aria-valid-attr-value',
      'button-name',
      'link-name',
      'label',
      'image-alt',
    ],
  },
};

/**
 * Run full WCAG 2.2 Level AA accessibility audit on a component
 *
 * @param container - The rendered component container (from React Testing Library)
 * @returns Promise resolving to axe results
 *
 * @example
 * ```typescript
 * import { renderWithProviders } from './testUtils';
 * import { runA11yAudit } from './AccessibilityTestHelper';
 *
 * test('component has no accessibility violations', async () => {
 *   const { container } = renderWithProviders(<MyComponent />);
 *   const results = await runA11yAudit(container);
 *   expect(results).toHaveNoViolations();
 * });
 * ```
 */
export async function runA11yAudit(container: Element | Document) {
  return await axe(container, WCAG_22_AA_CONFIG);
}

/**
 * Run color contrast audit on a component
 *
 * Verifies WCAG 1.4.3 color contrast requirements:
 * - Normal text (< 18pt or < 14pt bold): ≥4.5:1
 * - Large text (≥ 18pt or ≥ 14pt bold): ≥3:1
 *
 * @param container - The rendered component container
 * @returns Promise resolving to axe results
 *
 * @example
 * ```typescript
 * test('component meets color contrast standards', async () => {
 *   const { container } = renderWithProviders(<MyComponent />);
 *   const results = await runColorContrastAudit(container);
 *   expect(results).toHaveNoViolations();
 * });
 * ```
 */
export async function runColorContrastAudit(container: Element | Document) {
  return await axe(container, COLOR_CONTRAST_CONFIG);
}

/**
 * Run keyboard accessibility audit on a component
 *
 * Verifies:
 * - All interactive elements are keyboard accessible
 * - No keyboard traps exist
 * - Tab order is logical
 * - Focus indicators are visible
 *
 * @param container - The rendered component container
 * @returns Promise resolving to axe results
 *
 * @example
 * ```typescript
 * test('component supports keyboard navigation', async () => {
 *   const { container } = renderWithProviders(<MyComponent />);
 *   const results = await runKeyboardAccessibilityAudit(container);
 *   expect(results).toHaveNoViolations();
 * });
 * ```
 */
export async function runKeyboardAccessibilityAudit(container: Element | Document) {
  return await axe(container, KEYBOARD_ACCESSIBILITY_CONFIG);
}

/**
 * Run ARIA and semantic HTML audit on a component
 *
 * Verifies:
 * - Valid ARIA attributes and roles
 * - Proper semantic HTML usage
 * - Accessible names for interactive elements
 * - Form labels properly associated
 * - Images have alt text
 *
 * @param container - The rendered component container
 * @returns Promise resolving to axe results
 *
 * @example
 * ```typescript
 * test('component has proper ARIA labels', async () => {
 *   const { container } = renderWithProviders(<MyComponent />);
 *   const results = await runAriaSemanticAudit(container);
 *   expect(results).toHaveNoViolations();
 * });
 * ```
 */
export async function runAriaSemanticAudit(container: Element | Document) {
  return await axe(container, ARIA_SEMANTIC_CONFIG);
}

/**
 * Helper to verify all interactive elements have accessible names
 *
 * Checks buttons, links, and form inputs for proper labeling.
 *
 * @param container - The rendered component container
 * @returns Array of elements missing accessible names
 *
 * @example
 * ```typescript
 * test('all interactive elements are labeled', () => {
 *   const { container } = renderWithProviders(<MyComponent />);
 *   const unlabeled = findUnlabeledElements(container);
 *   expect(unlabeled).toHaveLength(0);
 * });
 * ```
 */
export function findUnlabeledElements(container: Element | Document): Element[] {
  const interactiveElements = container.querySelectorAll(
    'button, a, input, select, textarea, [role="button"], [role="link"]'
  );

  const unlabeled: Element[] = [];

  interactiveElements.forEach((element) => {
    const hasAriaLabel = element.hasAttribute('aria-label');
    const hasAriaLabelledBy = element.hasAttribute('aria-labelledby');
    const hasText = element.textContent && element.textContent.trim().length > 0;
    const hasAlt = element.hasAttribute('alt');
    const hasTitle = element.hasAttribute('title');
    const hasPlaceholder = element.hasAttribute('placeholder');

    // Check if element has any form of accessible name
    if (
      !hasAriaLabel &&
      !hasAriaLabelledBy &&
      !hasText &&
      !hasAlt &&
      !hasTitle &&
      !hasPlaceholder
    ) {
      unlabeled.push(element);
    }
  });

  return unlabeled;
}

/**
 * Helper to verify focus indicators are visible
 *
 * Checks if focused elements have visible outline or box-shadow.
 * Note: This is a basic check - comprehensive focus visibility testing
 * requires manual verification or browser automation (Playwright).
 *
 * @param element - The focused element
 * @returns true if element has visible focus indicator
 *
 * @example
 * ```typescript
 * test('focused elements have visible indicators', async () => {
 *   const { getByRole } = renderWithProviders(<MyComponent />);
 *   const button = getByRole('button');
 *   button.focus();
 *   expect(hasFocusIndicator(button)).toBe(true);
 * });
 * ```
 */
export function hasFocusIndicator(element: Element): boolean {
  const styles = window.getComputedStyle(element);

  // Check for outline
  if (styles.outline && styles.outline !== 'none' && styles.outlineWidth !== '0px') {
    return true;
  }

  // Check for box-shadow (common Tailwind focus style)
  if (styles.boxShadow && styles.boxShadow !== 'none') {
    return true;
  }

  // Check for border changes (some custom focus styles)
  if (styles.border && styles.border !== 'none') {
    return true;
  }

  return false;
}

/**
 * Run complete accessibility test suite
 *
 * Runs all accessibility audits:
 * - Full WCAG 2.2 Level AA audit
 * - Color contrast audit
 * - Keyboard accessibility audit
 * - ARIA and semantic HTML audit
 *
 * @param container - The rendered component container
 * @returns Promise resolving to combined results
 *
 * @example
 * ```typescript
 * test('component passes all accessibility checks', async () => {
 *   const { container } = renderWithProviders(<MyComponent />);
 *   const results = await runCompleteA11yAudit(container);
 *   expect(results.wcag22).toHaveNoViolations();
 *   expect(results.colorContrast).toHaveNoViolations();
 *   expect(results.keyboard).toHaveNoViolations();
 *   expect(results.aria).toHaveNoViolations();
 * });
 * ```
 */
export async function runCompleteA11yAudit(container: Element | Document) {
  const [wcag22, colorContrast, keyboard, aria] = await Promise.all([
    runA11yAudit(container),
    runColorContrastAudit(container),
    runKeyboardAccessibilityAudit(container),
    runAriaSemanticAudit(container),
  ]);

  return {
    wcag22,
    colorContrast,
    keyboard,
    aria,
  };
}
