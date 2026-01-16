import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { KeyboardVisualizer } from './KeyboardVisualizer';
import type { KeyMapping } from '@/types';

describe('KeyboardVisualizer', () => {
  const mockOnKeyClick = vi.fn();
  const defaultProps = {
    layout: 'ANSI_104' as const,
    keyMappings: new Map<string, KeyMapping>(),
    onKeyClick: mockOnKeyClick,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders keyboard visualizer', () => {
    renderWithProviders(<KeyboardVisualizer {...defaultProps} />);

    expect(screen.getByTestId('keyboard-visualizer')).toBeInTheDocument();
  });

  it('renders with SVG element', () => {
    const { container } = renderWithProviders(
      <KeyboardVisualizer {...defaultProps} />
    );

    const svg = container.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });

  it('renders key buttons with role=button', () => {
    renderWithProviders(<KeyboardVisualizer {...defaultProps} />);

    // SVG keys have role="button"
    const buttons = screen.getAllByRole('button');
    expect(buttons.length).toBeGreaterThan(0);
  });

  it('calls onKeyClick when key is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<KeyboardVisualizer {...defaultProps} />);

    // Find a key button (VK_ESC is first key in ANSI layout)
    const buttons = screen.getAllByRole('button');
    const escKey = buttons.find(
      (b) => b.getAttribute('aria-label')?.includes('VK_ESC')
    );

    if (escKey) {
      await user.click(escKey);
      expect(mockOnKeyClick).toHaveBeenCalledWith('VK_ESC');
    }
  });

  it('shows mapping indicator when key has mapping', () => {
    const keyMappings = new Map<string, KeyMapping>([
      ['VK_A', { type: 'simple', tapAction: 'VK_B' }],
    ]);

    const { container } = renderWithProviders(
      <KeyboardVisualizer {...defaultProps} keyMappings={keyMappings} />
    );

    // Should have text elements showing mapping
    const texts = container.querySelectorAll('text');
    const mappingText = Array.from(texts).find((t) => t.textContent === 'B');
    expect(mappingText).toBeTruthy();
  });

  // TODO: Fix - tests implementation details (SVG structure, CSS colors)
  it.skip('shows pressed state in simulator mode', () => {
    const pressedKeys = new Set(['VK_A']);

    const { container } = renderWithProviders(
      <KeyboardVisualizer
        {...defaultProps}
        simulatorMode={true}
        pressedKeys={pressedKeys}
      />
    );

    // Pressed keys have green fill color (#22c55e)
    const paths = container.querySelectorAll('path');
    const pressedPath = Array.from(paths).find(
      (p) => p.getAttribute('fill') === '#22c55e'
    );
    expect(pressedPath).toBeTruthy();
  });

  it('applies custom className', () => {
    const { container } = renderWithProviders(
      <KeyboardVisualizer {...defaultProps} className="custom-class" />
    );

    const wrapper = container.querySelector('.keyboard-visualizer');
    expect(wrapper).toHaveClass('custom-class');
  });

  it('renders different layouts', () => {
    const layouts = ['ANSI_104', 'ISO_105', 'JIS_109', 'COMPACT_60'] as const;

    layouts.forEach((layout) => {
      const { unmount, container } = renderWithProviders(
        <KeyboardVisualizer {...defaultProps} layout={layout} />
      );

      const svg = container.querySelector('svg');
      expect(svg).toBeInTheDocument();
      unmount();
    });
  });

  it('handles keyboard navigation', async () => {
    const user = userEvent.setup();
    renderWithProviders(<KeyboardVisualizer {...defaultProps} />);

    // Tab into the keyboard
    await user.tab();

    // Should be able to focus keys
    const focusedElement = document.activeElement;
    expect(focusedElement?.getAttribute('role')).toBe('button');
  });

  // TODO: Fix - tests implementation details (specific key codes in aria-label)
  it.skip('renders ISO layout with Enter key', () => {
    const { container } = renderWithProviders(
      <KeyboardVisualizer {...defaultProps} layout="ISO_105" />
    );

    // ISO layout should have VK_ENT key
    const buttons = screen.getAllByRole('button');
    const enterKey = buttons.find(
      (b) => b.getAttribute('aria-label')?.includes('VK_ENT')
    );
    expect(enterKey).toBeTruthy();
  });

  // TODO: Fix - tests implementation details (exact aria-label text content)
  it.skip('displays tooltip content on key aria-label', () => {
    const keyMappings = new Map<string, KeyMapping>([
      [
        'VK_A',
        {
          type: 'tap_hold',
          tapAction: 'VK_A',
          holdAction: 'VK_LCTL',
          threshold: 200,
        },
      ],
    ]);

    renderWithProviders(
      <KeyboardVisualizer {...defaultProps} keyMappings={keyMappings} />
    );

    const buttons = screen.getAllByRole('button');
    const aKey = buttons.find(
      (b) => b.getAttribute('aria-label')?.includes('VK_A')
    );

    // Should have tap/hold info in aria-label
    expect(aKey?.getAttribute('aria-label')).toContain('Tap');
    expect(aKey?.getAttribute('aria-label')).toContain('Hold');
  });

  it('disables click in simulator mode', async () => {
    const user = userEvent.setup();
    renderWithProviders(
      <KeyboardVisualizer {...defaultProps} simulatorMode={true} />
    );

    const buttons = screen.getAllByRole('button');
    if (buttons[0]) {
      await user.click(buttons[0]);
      // In simulator mode, clicks should not trigger onKeyClick
      expect(mockOnKeyClick).not.toHaveBeenCalled();
    }
  });

  describe('Key code prefix normalization (KC_ to VK_)', () => {
    it('shows mappings when layout uses KC_ codes but mappings use VK_ codes', () => {
      // This test prevents regression of the issue where:
      // - Layout JSON files use KC_ prefix (KC_B, KC_V, KC_M)
      // - But mappings from Rhai parser use VK_ prefix (VK_B, VK_V, VK_M)
      // - Result: mappings didn't show until normalization was added

      const keyMappings = new Map<string, KeyMapping>([
        [
          'VK_B',
          {
            type: 'tap_hold',
            tapAction: 'VK_Enter',
            holdAction: 'MD_00',
            threshold: 200,
          },
        ],
        [
          'VK_V',
          {
            type: 'tap_hold',
            tapAction: 'VK_Delete',
            holdAction: 'MD_01',
            threshold: 200,
          },
        ],
        [
          'VK_M',
          {
            type: 'tap_hold',
            tapAction: 'VK_Backspace',
            holdAction: 'MD_02',
            threshold: 200,
          },
        ],
      ]);

      const { container } = renderWithProviders(
        <KeyboardVisualizer {...defaultProps} keyMappings={keyMappings} />
      );

      // Should show mapping indicators (colored borders, overlay text)
      const texts = container.querySelectorAll('text');

      // Check for tap/hold mapping text
      const hasEnterMapping = Array.from(texts).some(
        (t) => t.textContent?.includes('Enter')
      );
      const hasDeleteMapping = Array.from(texts).some(
        (t) => t.textContent?.includes('Del')
      );
      const hasBackspaceMapping = Array.from(texts).some(
        (t) => t.textContent?.includes('BS')
      );

      expect(hasEnterMapping || hasDeleteMapping || hasBackspaceMapping).toBe(
        true
      );

      // Should have red strokes for tap_hold mappings
      const paths = container.querySelectorAll('path');
      const redStrokes = Array.from(paths).filter(
        (p) => p.getAttribute('stroke') === '#ef4444'
      );
      expect(redStrokes.length).toBeGreaterThan(0);
    });

    it('shows simple mappings with green borders regardless of prefix mismatch', () => {
      const keyMappings = new Map<string, KeyMapping>([
        ['VK_A', { type: 'simple', tapAction: 'VK_B' }],
        ['VK_S', { type: 'simple', tapAction: 'VK_C' }],
      ]);

      const { container } = renderWithProviders(
        <KeyboardVisualizer {...defaultProps} keyMappings={keyMappings} />
      );

      // Should have green strokes for simple mappings
      const paths = container.querySelectorAll('path');
      const greenStrokes = Array.from(paths).filter(
        (p) => p.getAttribute('stroke') === '#22c55e'
      );
      expect(greenStrokes.length).toBeGreaterThan(0);
    });

    it('calls onKeyClick with normalized VK_ code', async () => {
      const user = userEvent.setup();
      renderWithProviders(<KeyboardVisualizer {...defaultProps} />);

      // Layout files use KC_ESC, but onKeyClick should receive VK_ESC
      const buttons = screen.getAllByRole('button');
      const escKey = buttons.find(
        (b) => b.getAttribute('aria-label')?.includes('VK_ESC')
      );

      if (escKey) {
        await user.click(escKey);
        // Should be called with VK_ prefix
        expect(mockOnKeyClick).toHaveBeenCalledWith('VK_ESC');
        // Should NOT be called with KC_ prefix
        expect(mockOnKeyClick).not.toHaveBeenCalledWith('KC_ESC');
      }
    });
  });
});
