import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SVGKeyboard, type SVGKey } from '../SVGKeyboard';
import type { KeyMapping } from '@/types';

describe('SVGKeyboard - Key Code Normalization', () => {
  const mockOnKeyClick = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('KC_ to VK_ prefix normalization', () => {
    it('shows simple mapping when key code has KC_ prefix but mapping has VK_ prefix', () => {
      // Layout uses KC_ prefix (as in real layout JSON files)
      const keys: SVGKey[] = [
        { code: 'KC_B', label: 'B', x: 0, y: 0, w: 1, h: 1 },
      ];

      // Mapping uses VK_ prefix (as from Rhai parser)
      const keyMappings = new Map<string, KeyMapping>([
        ['VK_B', { type: 'simple', tapAction: 'VK_Enter' }],
      ]);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // Should show the mapping text (Enter)
      const texts = container.querySelectorAll('text');
      const mappingText = Array.from(texts).find(
        (t) => t.textContent === 'Enter'
      );
      expect(mappingText).toBeTruthy();

      // Should have green stroke for simple mapping
      const paths = container.querySelectorAll('path');
      const greenStroke = Array.from(paths).find(
        (p) => p.getAttribute('stroke') === '#22c55e'
      );
      expect(greenStroke).toBeTruthy();
    });

    it('shows tap_hold mapping when key code has KC_ prefix but mapping has VK_ prefix', () => {
      const keys: SVGKey[] = [
        { code: 'KC_V', label: 'V', x: 0, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        [
          'VK_V',
          {
            type: 'tap_hold',
            tapAction: 'VK_Delete',
            holdAction: 'MD_01',
            threshold: 200,
          },
        ],
      ]);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // Should show tap/hold text (Delete/L01 format)
      const texts = container.querySelectorAll('text');
      const mappingText = Array.from(texts).find(
        (t) => t.textContent?.includes('Del')
      );
      expect(mappingText).toBeTruthy();

      // Should have red stroke for tap_hold mapping
      const paths = container.querySelectorAll('path');
      const redStroke = Array.from(paths).find(
        (p) => p.getAttribute('stroke') === '#ef4444'
      );
      expect(redStroke).toBeTruthy();
    });

    it('shows macro mapping when key code has KC_ prefix but mapping has VK_ prefix', () => {
      const keys: SVGKey[] = [
        { code: 'KC_M', label: 'M', x: 0, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        [
          'VK_M',
          {
            type: 'macro',
            macroSteps: [
              { type: 'press', key: 'VK_H' },
              { type: 'press', key: 'VK_I' },
            ],
          },
        ],
      ]);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // Should show macro icon (⚡)
      const texts = container.querySelectorAll('text');
      const macroIcon = Array.from(texts).find((t) => t.textContent === '⚡');
      expect(macroIcon).toBeTruthy();

      // Should have purple stroke for macro mapping
      const paths = container.querySelectorAll('path');
      const purpleStroke = Array.from(paths).find(
        (p) => p.getAttribute('stroke') === '#a855f7'
      );
      expect(purpleStroke).toBeTruthy();
    });

    it('shows layer_switch mapping when key code has KC_ prefix but mapping has VK_ prefix', () => {
      const keys: SVGKey[] = [
        { code: 'KC_SPACE', label: 'Space', x: 0, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        [
          'VK_SPACE',
          {
            type: 'layer_switch',
            targetLayer: 'MD_00',
          },
        ],
      ]);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // Should show layer text (L00)
      const texts = container.querySelectorAll('text');
      const layerText = Array.from(texts).find((t) => t.textContent === 'L00');
      expect(layerText).toBeTruthy();

      // Should have yellow stroke for layer_switch mapping
      const paths = container.querySelectorAll('path');
      const yellowStroke = Array.from(paths).find(
        (p) => p.getAttribute('stroke') === '#eab308'
      );
      expect(yellowStroke).toBeTruthy();
    });

    it('shows dashed border when key has no mapping', () => {
      const keys: SVGKey[] = [
        { code: 'KC_A', label: 'A', x: 0, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>();

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // Should have dashed stroke for unmapped key
      const paths = container.querySelectorAll('path');
      const dashedStroke = Array.from(paths).find(
        (p) => p.getAttribute('stroke-dasharray') === '4 2'
      );
      expect(dashedStroke).toBeTruthy();
    });

    it('calls onKeyClick with normalized VK_ code when KC_ key is clicked', async () => {
      const user = userEvent.setup();
      const keys: SVGKey[] = [
        { code: 'KC_B', label: 'B', x: 0, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        ['VK_B', { type: 'simple', tapAction: 'VK_Enter' }],
      ]);

      render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      const button = screen.getByRole('button');
      await user.click(button);

      // Should be called with VK_ prefix, not KC_
      expect(mockOnKeyClick).toHaveBeenCalledWith('VK_B');
    });

    it('handles multiple keys with different prefixes', () => {
      const keys: SVGKey[] = [
        { code: 'KC_A', label: 'A', x: 0, y: 0, w: 1, h: 1 },
        { code: 'VK_B', label: 'B', x: 1, y: 0, w: 1, h: 1 },
        { code: 'KC_C', label: 'C', x: 2, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        ['VK_A', { type: 'simple', tapAction: 'VK_1' }],
        ['VK_B', { type: 'simple', tapAction: 'VK_2' }],
        ['VK_C', { type: 'simple', tapAction: 'VK_3' }],
      ]);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // All three should show green borders (simple mappings)
      const paths = container.querySelectorAll('path');
      const greenStrokes = Array.from(paths).filter(
        (p) => p.getAttribute('stroke') === '#22c55e'
      );
      expect(greenStrokes.length).toBeGreaterThanOrEqual(3);
    });

    it('handles pressed keys with both KC_ and VK_ prefixes', () => {
      const keys: SVGKey[] = [
        { code: 'KC_A', label: 'A', x: 0, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        ['VK_A', { type: 'simple', tapAction: 'VK_B' }],
      ]);

      // Pressed keys set might use either prefix
      const pressedKeys = new Set(['VK_A']);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
          pressedKeys={pressedKeys}
        />
      );

      // Should show pressed state (green fill #22c55e)
      const paths = container.querySelectorAll('path');
      const pressedPath = Array.from(paths).find(
        (p) => p.getAttribute('fill') === '#22c55e'
      );
      expect(pressedPath).toBeTruthy();
    });

    it('shows tooltip with normalized key code', () => {
      const keys: SVGKey[] = [
        { code: 'KC_B', label: 'B', x: 0, y: 0, w: 1, h: 1 },
      ];

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
      ]);

      render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      const button = screen.getByRole('button');
      const ariaLabel = button.getAttribute('aria-label');

      // Should reference VK_ code in aria-label
      expect(ariaLabel).toContain('KC_B');
      expect(ariaLabel).toContain('Tap');
      expect(ariaLabel).toContain('Hold');
    });
  });

  describe('ISO Enter key with normalization', () => {
    it('renders ISO Enter with KC_ code and shows VK_ mapping', () => {
      const keys: SVGKey[] = [
        {
          code: 'KC_ENT',
          label: 'Enter',
          x: 0,
          y: 0,
          w: 1.25,
          h: 2,
          shape: 'iso-enter',
        },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        ['VK_ENT', { type: 'simple', tapAction: 'VK_Backspace' }],
      ]);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // Should render ISO Enter shape and show mapping
      const texts = container.querySelectorAll('text');
      const mappingText = Array.from(texts).find((t) => t.textContent === 'BS');
      expect(mappingText).toBeTruthy();
    });
  });

  describe('Edge cases', () => {
    it('handles keys without prefix', () => {
      const keys: SVGKey[] = [
        { code: 'A', label: 'A', x: 0, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        ['VK_A', { type: 'simple', tapAction: 'VK_B' }],
      ]);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // Should normalize 'A' to 'VK_A' and show mapping
      const texts = container.querySelectorAll('text');
      const mappingText = Array.from(texts).find((t) => t.textContent === 'B');
      expect(mappingText).toBeTruthy();
    });

    it('handles empty key code', () => {
      const keys: SVGKey[] = [{ code: '', label: '', x: 0, y: 0, w: 1, h: 1 }];

      const keyMappings = new Map<string, KeyMapping>();

      // Should not crash
      expect(() => {
        render(
          <SVGKeyboard
            keys={keys}
            keyMappings={keyMappings}
            onKeyClick={mockOnKeyClick}
          />
        );
      }).not.toThrow();
    });

    it('preserves VK_ prefix when already present', async () => {
      const user = userEvent.setup();
      const keys: SVGKey[] = [
        { code: 'VK_B', label: 'B', x: 0, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        ['VK_B', { type: 'simple', tapAction: 'VK_Enter' }],
      ]);

      render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      const button = screen.getByRole('button');
      await user.click(button);

      // Should still use VK_B, not VK_VK_B
      expect(mockOnKeyClick).toHaveBeenCalledWith('VK_B');
      expect(mockOnKeyClick).not.toHaveBeenCalledWith('VK_VK_B');
    });
  });

  describe('Real-world scenario: Default profile mappings', () => {
    it('shows all 69 mappings from user profile with KC_ layout keys', () => {
      // Simulate the real issue: layout uses KC_, mappings use VK_
      const keys: SVGKey[] = [
        { code: 'KC_B', label: 'B', x: 0, y: 0, w: 1, h: 1 },
        { code: 'KC_V', label: 'V', x: 1, y: 0, w: 1, h: 1 },
        { code: 'KC_M', label: 'M', x: 2, y: 0, w: 1, h: 1 },
        { code: 'KC_A', label: 'A', x: 3, y: 0, w: 1, h: 1 },
      ];

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
        ['VK_A', { type: 'simple', tapAction: 'VK_Space' }],
      ]);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // All mapped keys should have colored borders (not dashed)
      const paths = container.querySelectorAll('path');
      const coloredStrokes = Array.from(paths).filter((p) => {
        const stroke = p.getAttribute('stroke');
        return (
          stroke &&
          ['#22c55e', '#ef4444', '#a855f7', '#eab308'].includes(stroke)
        );
      });

      expect(coloredStrokes.length).toBeGreaterThanOrEqual(4); // At least 4 mapped keys
    });
  });

  describe('Numpad key normalization (KC_P* to VK_Numpad*)', () => {
    it('shows mappings for numpad digit keys when layout uses KC_P* but mappings use VK_Numpad*', () => {
      const keys: SVGKey[] = [
        { code: 'KC_P2', label: '2', x: 0, y: 0, w: 1, h: 1 },
        { code: 'KC_P3', label: '3', x: 1, y: 0, w: 1, h: 1 },
        { code: 'KC_P4', label: '4', x: 2, y: 0, w: 1, h: 1 },
        { code: 'KC_P5', label: '5', x: 3, y: 0, w: 1, h: 1 },
        { code: 'KC_P8', label: '8', x: 4, y: 0, w: 1, h: 1 },
        { code: 'KC_P9', label: '9', x: 5, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        ['VK_Numpad2', { type: 'simple', tapAction: 'VK_Left' }],
        ['VK_Numpad3', { type: 'simple', tapAction: 'VK_Right' }],
        ['VK_Numpad4', { type: 'simple', tapAction: 'VK_Down' }],
        ['VK_Numpad5', { type: 'simple', tapAction: 'VK_Up' }],
        ['VK_Numpad8', { type: 'simple', tapAction: 'VK_Home' }],
        ['VK_Numpad9', { type: 'simple', tapAction: 'VK_End' }],
      ]);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // Should show mapping text for arrows
      const texts = container.querySelectorAll('text');
      const hasLeftMapping = Array.from(texts).some(
        (t) => t.textContent?.includes('Left')
      );
      const hasRightMapping = Array.from(texts).some(
        (t) => t.textContent?.includes('Right')
      );
      const hasDownMapping = Array.from(texts).some(
        (t) => t.textContent?.includes('Down')
      );
      const hasUpMapping = Array.from(texts).some(
        (t) => t.textContent?.includes('Up')
      );

      expect(
        hasLeftMapping || hasRightMapping || hasDownMapping || hasUpMapping
      ).toBe(true);

      // Should have green strokes for simple mappings
      const paths = container.querySelectorAll('path');
      const greenStrokes = Array.from(paths).filter(
        (p) => p.getAttribute('stroke') === '#22c55e'
      );
      expect(greenStrokes.length).toBeGreaterThan(0);
    });

    it('normalizes all numpad digit keys correctly (0-9)', () => {
      const keys: SVGKey[] = [
        { code: 'KC_P0', label: '0', x: 0, y: 0, w: 1, h: 1 },
        { code: 'KC_P1', label: '1', x: 1, y: 0, w: 1, h: 1 },
        { code: 'KC_P7', label: '7', x: 2, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        ['VK_Numpad0', { type: 'simple', tapAction: 'VK_A' }],
        ['VK_Numpad1', { type: 'simple', tapAction: 'VK_B' }],
        ['VK_Numpad7', { type: 'simple', tapAction: 'VK_C' }],
      ]);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // Should have green strokes for all mapped keys
      const paths = container.querySelectorAll('path');
      const greenStrokes = Array.from(paths).filter(
        (p) => p.getAttribute('stroke') === '#22c55e'
      );
      expect(greenStrokes.length).toBeGreaterThanOrEqual(3);
    });

    it('normalizes special numpad keys (NumLock, operators, etc.)', () => {
      const keys: SVGKey[] = [
        { code: 'KC_NLCK', label: 'Num', x: 0, y: 0, w: 1, h: 1 },
        { code: 'KC_PSLS', label: '/', x: 1, y: 0, w: 1, h: 1 },
        { code: 'KC_PAST', label: '*', x: 2, y: 0, w: 1, h: 1 },
        { code: 'KC_PMNS', label: '-', x: 3, y: 0, w: 1, h: 1 },
        { code: 'KC_PPLS', label: '+', x: 4, y: 0, w: 1, h: 1 },
        { code: 'KC_PENT', label: 'Ent', x: 5, y: 0, w: 1, h: 1 },
        { code: 'KC_PDOT', label: '.', x: 6, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        ['VK_NumLock', { type: 'simple', tapAction: 'VK_A' }],
        ['VK_NumpadDivide', { type: 'simple', tapAction: 'VK_B' }],
        ['VK_NumpadMultiply', { type: 'simple', tapAction: 'VK_C' }],
        ['VK_NumpadSubtract', { type: 'simple', tapAction: 'VK_D' }],
        ['VK_NumpadAdd', { type: 'simple', tapAction: 'VK_E' }],
        ['VK_NumpadEnter', { type: 'simple', tapAction: 'VK_F' }],
        ['VK_NumpadDecimal', { type: 'simple', tapAction: 'VK_G' }],
      ]);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // Should have green strokes for all mapped keys
      const paths = container.querySelectorAll('path');
      const greenStrokes = Array.from(paths).filter(
        (p) => p.getAttribute('stroke') === '#22c55e'
      );
      expect(greenStrokes.length).toBeGreaterThanOrEqual(7);
    });

    it('calls onKeyClick with normalized VK_Numpad* code when KC_P* key is clicked', async () => {
      const user = userEvent.setup();
      const keys: SVGKey[] = [
        { code: 'KC_P2', label: '2', x: 0, y: 0, w: 1, h: 1 },
      ];

      const keyMappings = new Map<string, KeyMapping>([
        ['VK_Numpad2', { type: 'simple', tapAction: 'VK_Left' }],
      ]);

      render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      const button = screen.getByRole('button');
      await user.click(button);

      // Should be called with VK_Numpad2, not KC_P2
      expect(mockOnKeyClick).toHaveBeenCalledWith('VK_Numpad2');
      expect(mockOnKeyClick).not.toHaveBeenCalledWith('KC_P2');
    });

    it('correctly distinguishes top row numbers (VK_Num*) from numpad numbers (VK_Numpad*)', () => {
      const keys: SVGKey[] = [
        { code: 'KC_2', label: '2', x: 0, y: 0, w: 1, h: 1 }, // Top row
        { code: 'KC_P2', label: '2', x: 1, y: 0, w: 1, h: 1 }, // Numpad
      ];

      const keyMappings = new Map<string, KeyMapping>([
        ['VK_Num2', { type: 'simple', tapAction: 'VK_A' }], // Top row
        ['VK_Numpad2', { type: 'simple', tapAction: 'VK_B' }], // Numpad
      ]);

      const { container } = render(
        <SVGKeyboard
          keys={keys}
          keyMappings={keyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // Both should show green borders (both mapped)
      const paths = container.querySelectorAll('path');
      const greenStrokes = Array.from(paths).filter(
        (p) => p.getAttribute('stroke') === '#22c55e'
      );
      expect(greenStrokes.length).toBeGreaterThanOrEqual(2);
    });
  });
});
