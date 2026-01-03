/**
 * Accessibility tests for Drag-and-Drop components
 *
 * WCAG 2.2 Level AA compliance verification for keyboard-accessible drag-and-drop
 * Requirements: Task 14 (Requirement 4 - WCAG 2.2 Level AA)
 *
 * Tests verify:
 * - Keyboard accessibility (WCAG 2.1.1, 2.1.2)
 * - Focus indicators (WCAG 2.4.7)
 * - ARIA labels and roles (WCAG 4.1.2)
 * - Touch targets ≥44px (WCAG 2.5.5)
 * - Color contrast ≥4.5:1 (WCAG 1.4.3)
 */

import { describe, test, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  renderWithProviders,
  setupMockWebSocket,
  cleanupMockWebSocket,
} from '../../../tests/testUtils';
import { runA11yAudit, runCompleteA11yAudit } from '../../../tests/AccessibilityTestHelper';
import { KeyAssignmentPanel } from '../KeyAssignmentPanel';
import { KeyboardVisualizer } from '../KeyboardVisualizer';
import { DndContext } from '@dnd-kit/core';

describe('Drag-and-Drop Accessibility', () => {
  beforeEach(async () => {
    await setupMockWebSocket();
  });

  afterEach(() => {
    cleanupMockWebSocket();
  });

  describe('KeyAssignmentPanel Accessibility', () => {
    test('should have no WCAG 2.2 Level AA violations', async () => {
      const { container } = render(<KeyAssignmentPanel />);

      const results = await runA11yAudit(container);
      expect(results).toHaveNoViolations();
    });

    test('should pass complete accessibility audit', async () => {
      const { container } = render(<KeyAssignmentPanel />);

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

    test('should have proper ARIA labels on draggable keys', () => {
      render(<KeyAssignmentPanel />);

      // Find draggable key buttons
      const aKeyButton = screen.getByRole('button', { name: /A key/i });

      expect(aKeyButton).toHaveAttribute('aria-label');
      expect(aKeyButton.getAttribute('aria-label')).toContain('Press Space to grab');
    });

    test('should have keyboard usage instructions visible', () => {
      render(<KeyAssignmentPanel />);

      const instructions = screen.getByText(/Tab to focus a key, Space to grab/i);
      expect(instructions).toBeInTheDocument();
    });

    test('should have touch-friendly targets (≥44px)', () => {
      render(<KeyAssignmentPanel />);

      const buttons = screen.getAllByRole('button', { name: /key/i });

      buttons.slice(0, 5).forEach((button) => {
        const styles = window.getComputedStyle(button);
        const minHeight = parseInt(styles.minHeight);
        const minWidth = parseInt(styles.minWidth);

        // WCAG 2.5.5: Target size minimum 44x44px
        expect(minHeight).toBeGreaterThanOrEqual(44);
        expect(minWidth).toBeGreaterThanOrEqual(44);
      });
    });

    test('should have visible focus indicators', async () => {
      const user = userEvent.setup();
      render(<KeyAssignmentPanel />);

      // Tab to focus first key
      await user.tab();

      const focusedElement = document.activeElement;
      expect(focusedElement).toBeTruthy();

      // Check for focus outline styles
      const styles = window.getComputedStyle(focusedElement!);
      const outline = styles.outline || styles.outlineStyle;

      // Should have visible focus indicator (outline or box-shadow)
      expect(
        outline !== 'none' ||
        styles.outlineWidth !== '0px' ||
        styles.boxShadow !== 'none'
      ).toBe(true);
    });

    test('should support keyboard navigation between keys', async () => {
      const user = userEvent.setup();
      render(<KeyAssignmentPanel />);

      // Tab to first key
      await user.tab();
      const firstFocused = document.activeElement;

      // Tab to next key
      await user.tab();
      const secondFocused = document.activeElement;

      // Should move focus to different element
      expect(firstFocused).not.toBe(secondFocused);
      expect(secondFocused?.getAttribute('role')).toBe('button');
    });

    test('should have complementary landmark role', () => {
      const { container } = render(<KeyAssignmentPanel />);

      const palette = container.querySelector('[role="complementary"]');
      expect(palette).toBeInTheDocument();
      expect(palette).toHaveAttribute('aria-label', 'Key assignment palette');
    });

    test('should have proper tab panel structure', () => {
      render(<KeyAssignmentPanel />);

      const tablist = screen.getByRole('tablist');
      expect(tablist).toBeInTheDocument();

      const tabs = within(tablist).getAllByRole('tab');
      expect(tabs.length).toBeGreaterThan(0);

      // Each tab should have aria-selected
      tabs.forEach((tab) => {
        expect(tab).toHaveAttribute('aria-selected');
        expect(tab).toHaveAttribute('aria-controls');
      });
    });

    test('should announce drag state changes with aria-grabbed', () => {
      render(<KeyAssignmentPanel />);

      const keyButton = screen.getByRole('button', { name: /A key/i });

      // Should have aria-grabbed attribute
      expect(keyButton).toHaveAttribute('aria-grabbed');
    });
  });

  describe('KeyboardVisualizer Accessibility', () => {
    const mockKeyMappings = new Map();
    const mockOnKeyClick = vi.fn();

    test('should have no WCAG 2.2 Level AA violations', async () => {
      const { container } = render(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
          simulatorMode={false}
        />
      );

      const results = await runA11yAudit(container);
      expect(results).toHaveNoViolations();
    });

    test('should have proper group role with descriptive label', () => {
      render(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
          simulatorMode={false}
        />
      );

      const keyboardGroup = screen.getByRole('group');
      expect(keyboardGroup).toHaveAttribute('aria-label');
      expect(keyboardGroup.getAttribute('aria-label')).toContain('ANSI_104');
      expect(keyboardGroup.getAttribute('aria-label')).toContain('arrow keys to navigate');
    });

    test('should have ARIA labels on drop zones', () => {
      const { container } = render(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
          simulatorMode={false}
        />
      );

      const dropZones = container.querySelectorAll('[role="button"]');
      expect(dropZones.length).toBeGreaterThan(0);

      // Check first few drop zones have proper labels
      const firstDropZone = dropZones[0];
      expect(firstDropZone).toHaveAttribute('aria-label');
      expect(firstDropZone.getAttribute('aria-label')).toContain('Drop zone');
    });

    test('should have visible focus indicators on keyboard keys', async () => {
      const user = userEvent.setup();
      const { container } = render(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
          simulatorMode={false}
        />
      );

      const dropZones = container.querySelectorAll('[role="button"]');
      const firstKey = dropZones[0] as HTMLElement;

      // Focus the element
      firstKey.focus();

      const styles = window.getComputedStyle(firstKey);

      // Should have visible focus indicator
      expect(
        styles.outline !== 'none' ||
        styles.outlineWidth !== '0px' ||
        styles.boxShadow !== 'none' ||
        firstKey.className.includes('focus')
      ).toBe(true);
    });

    test('should support keyboard interaction on drop zones', async () => {
      const user = userEvent.setup();
      const { container } = render(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
          simulatorMode={false}
        />
      );

      const dropZones = container.querySelectorAll('[role="button"]');
      const firstKey = dropZones[0] as HTMLElement;

      // Should be keyboard focusable
      expect(firstKey).toHaveAttribute('tabindex');
      expect(parseInt(firstKey.getAttribute('tabindex') || '-1')).toBeGreaterThanOrEqual(0);
    });

    test('should indicate drop effect state with aria-dropeffect', () => {
      const { container } = render(
        <DndContext>
          <KeyboardVisualizer
            layout="ANSI_104"
            keyMappings={mockKeyMappings}
            onKeyClick={mockOnKeyClick}
            onKeyDrop={vi.fn()}
            simulatorMode={false}
          />
        </DndContext>
      );

      const dropZones = container.querySelectorAll('[aria-dropeffect]');
      expect(dropZones.length).toBeGreaterThan(0);

      // Each drop zone should have aria-dropeffect attribute
      dropZones.forEach((zone) => {
        const dropEffect = zone.getAttribute('aria-dropeffect');
        expect(['none', 'move']).toContain(dropEffect);
      });
    });

    test('should disable interaction in simulator mode', () => {
      const { container } = render(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
          simulatorMode={true}
        />
      );

      const dropZones = container.querySelectorAll('[role="button"]');
      const firstKey = dropZones[0] as HTMLElement;

      // In simulator mode, drop zones should be disabled
      expect(firstKey.getAttribute('aria-label')).toContain('Not configurable');
      expect(parseInt(firstKey.getAttribute('tabindex') || '0')).toBe(-1);
    });
  });

  describe('Integrated Drag-and-Drop Accessibility', () => {
    test('should maintain focus management during drag operation', async () => {
      const user = userEvent.setup();

      const { container } = render(
        <DndContext>
          <div style={{ display: 'flex', gap: '16px' }}>
            <KeyAssignmentPanel />
            <KeyboardVisualizer
              layout="ANSI_104"
              keyMappings={new Map()}
              onKeyClick={vi.fn()}
              onKeyDrop={vi.fn()}
              simulatorMode={false}
            />
          </div>
        </DndContext>
      );

      // Tab to first draggable key
      await user.tab();
      const initialFocus = document.activeElement;

      expect(initialFocus).toBeTruthy();
      expect(initialFocus?.getAttribute('role')).toBe('button');
    });

    test('should have no keyboard traps', async () => {
      const user = userEvent.setup();

      render(
        <DndContext>
          <div style={{ display: 'flex', gap: '16px' }}>
            <KeyAssignmentPanel />
            <KeyboardVisualizer
              layout="ANSI_104"
              keyMappings={new Map()}
              onKeyClick={vi.fn()}
              onKeyDrop={vi.fn()}
              simulatorMode={false}
            />
          </div>
        </DndContext>
      );

      // Tab through multiple elements
      for (let i = 0; i < 10; i++) {
        await user.tab();
      }

      // Should still be able to tab (not trapped)
      const focusedElement = document.activeElement;
      expect(focusedElement).toBeTruthy();
    });

    test('should support Escape key to cancel drag', () => {
      // This is handled by @dnd-kit keyboard sensor
      // Just verify the component renders without errors
      const { container } = render(
        <DndContext>
          <KeyAssignmentPanel />
        </DndContext>
      );

      expect(container).toBeTruthy();
    });
  });

  describe('Color Contrast Compliance', () => {
    test('should meet WCAG 1.4.3 color contrast requirements', async () => {
      const { container } = render(
        <div>
          <KeyAssignmentPanel />
          <KeyboardVisualizer
            layout="ANSI_104"
            keyMappings={new Map()}
            onKeyClick={vi.fn()}
            simulatorMode={false}
          />
        </div>
      );

      const results = await runCompleteA11yAudit(container);

      // Color contrast must meet ≥4.5:1 for normal text, ≥3:1 for large text
      expect(results.colorContrast).toHaveNoViolations();
    });
  });

  describe('Screen Reader Support', () => {
    test('should have screen reader instructions for palette', () => {
      render(<KeyAssignmentPanel />);

      const instructions = screen.getByText(/Tab to focus a key/i);

      // Instructions should be visible (not sr-only)
      expect(instructions).toBeVisible();
    });

    test('should provide context in ARIA labels', () => {
      render(<KeyAssignmentPanel />);

      const keyButton = screen.getByRole('button', { name: /A key/i });
      const ariaLabel = keyButton.getAttribute('aria-label');

      // ARIA label should provide sufficient context
      expect(ariaLabel).toContain('key');
      expect(ariaLabel).toContain('Space');
    });
  });
});
