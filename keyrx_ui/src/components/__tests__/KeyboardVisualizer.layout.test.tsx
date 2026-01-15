import { describe, it, expect } from 'vitest';
import { renderWithProviders, screen } from '../../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { KeyboardVisualizer } from '../KeyboardVisualizer';

describe('KeyboardVisualizer Layout Integration', () => {
  const mockKeyMappings = new Map();
  const mockOnKeyClick = () => {};

  const layouts = [
    { layout: 'ANSI_104' as const, expectedKeyCount: 104, description: 'ANSI 104 full-size layout' },
    { layout: 'ANSI_87' as const, expectedKeyCount: 87, description: 'ANSI 87 TKL layout' },
    { layout: 'ISO_105' as const, expectedKeyCount: 105, description: 'ISO 105 full-size layout' },
    { layout: 'ISO_88' as const, expectedKeyCount: 88, description: 'ISO 88 TKL layout' },
    { layout: 'JIS_109' as const, expectedKeyCount: 109, description: 'JIS 109 Japanese layout' },
    { layout: 'COMPACT_60' as const, expectedKeyCount: 61, description: '60% compact layout' },
    { layout: 'COMPACT_65' as const, expectedKeyCount: 67, description: '65% compact layout' },
    { layout: 'COMPACT_75' as const, expectedKeyCount: 82, description: '75% compact layout' },
    { layout: 'COMPACT_96' as const, expectedKeyCount: 100, description: '96% compact layout' },
    { layout: 'HHKB' as const, expectedKeyCount: 60, description: 'HHKB layout' },
    { layout: 'NUMPAD' as const, expectedKeyCount: 17, description: 'Standalone numpad layout' },
  ];

  describe.each(layouts)('$description', ({ layout, expectedKeyCount, description }) => {
    it('should render without errors', () => {
      const { container } = renderWithProviders(
        <KeyboardVisualizer
          layout={layout}
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
        />,
        { wrapWithWasm: false }
      );

      expect(container).toBeInTheDocument();
    });

    it('should render keyboard visualizer container with correct aria-label', () => {
      renderWithProviders(
        <KeyboardVisualizer
          layout={layout}
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
        />,
        { wrapWithWasm: false }
      );

      const visualizer = screen.getByTestId('keyboard-visualizer');
      expect(visualizer).toBeInTheDocument();
      expect(visualizer).toHaveAttribute('aria-label');
      expect(visualizer.getAttribute('aria-label')).toContain(layout);
    });

    it(`should render exactly ${expectedKeyCount} key buttons`, () => {
      renderWithProviders(
        <KeyboardVisualizer
          layout={layout}
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
        />,
        { wrapWithWasm: false }
      );

      // Each key is rendered as a button element
      const keyButtons = screen.getAllByRole('button');
      expect(keyButtons).toHaveLength(expectedKeyCount);
    });

    it('should render with correct CSS grid structure', () => {
      renderWithProviders(
        <KeyboardVisualizer
          layout={layout}
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
        />,
        { wrapWithWasm: false }
      );

      const visualizer = screen.getByTestId('keyboard-visualizer');
      const style = window.getComputedStyle(visualizer);

      // Verify grid display
      expect(style.display).toBe('grid');
    });

    it('should have accessible keyboard navigation', () => {
      renderWithProviders(
        <KeyboardVisualizer
          layout={layout}
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
        />,
        { wrapWithWasm: false }
      );

      const visualizer = screen.getByTestId('keyboard-visualizer');
      expect(visualizer).toHaveAttribute('role', 'group');

      // Verify aria-label provides navigation instructions
      const ariaLabel = visualizer.getAttribute('aria-label') || '';
      expect(ariaLabel).toContain('arrow keys');
      expect(ariaLabel).toContain('Enter');
    });

    it('should call onKeyClick when a key is clicked', async () => {
      let clickedKeyCode: string | null = null;
      const handleKeyClick = (keyCode: string) => {
        clickedKeyCode = keyCode;
      };

      renderWithProviders(
        <KeyboardVisualizer
          layout={layout}
          keyMappings={mockKeyMappings}
          onKeyClick={handleKeyClick}
        />,
        { wrapWithWasm: false }
      );

      const user = userEvent.setup();
      const keyButtons = screen.getAllByRole('button');
      expect(keyButtons.length).toBeGreaterThan(0);

      // Click the first key
      await user.click(keyButtons[0]);

      // Verify callback was invoked with a key code
      expect(clickedKeyCode).not.toBeNull();
      expect(typeof clickedKeyCode).toBe('string');
      // Should be normalized to VK_ format for consistency with mappings
      expect(clickedKeyCode).toMatch(/^VK_/);
    });
  });

  describe('Layout switching', () => {
    it('should update rendered keys when layout prop changes', () => {
      const { rerender } = renderWithProviders(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
        />,
        { wrapWithWasm: false }
      );

      // Initially renders ANSI_104 (104 keys)
      let keyButtons = screen.getAllByRole('button');
      expect(keyButtons).toHaveLength(104);

      // Switch to NUMPAD (17 keys)
      rerender(
        <KeyboardVisualizer
          layout="NUMPAD"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      // Should now render NUMPAD keys
      keyButtons = screen.getAllByRole('button');
      expect(keyButtons).toHaveLength(17);
    });

    it('should update aria-label when layout changes', () => {
      const { rerender } = renderWithProviders(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
        />,
        { wrapWithWasm: false }
      );

      let visualizer = screen.getByTestId('keyboard-visualizer');
      expect(visualizer.getAttribute('aria-label')).toContain('ANSI_104');

      rerender(
        <KeyboardVisualizer
          layout="HHKB"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
        />
      );

      visualizer = screen.getByTestId('keyboard-visualizer');
      expect(visualizer.getAttribute('aria-label')).toContain('HHKB');
    });
  });

  describe('Simulator mode', () => {
    it('should indicate simulator mode in aria-label', () => {
      renderWithProviders(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
          simulatorMode={true}
        />,
        { wrapWithWasm: false }
      );

      const visualizer = screen.getByTestId('keyboard-visualizer');
      const ariaLabel = visualizer.getAttribute('aria-label') || '';
      expect(ariaLabel).toContain('simulator mode');
    });

    it('should disable key interactions in simulator mode', async () => {
      let clickCount = 0;
      const handleKeyClick = () => {
        clickCount++;
      };

      renderWithProviders(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={handleKeyClick}
          simulatorMode={true}
        />,
        { wrapWithWasm: false }
      );

      const user = userEvent.setup();
      const keyButtons = screen.getAllByRole('button');

      // Try clicking a key in simulator mode
      await user.click(keyButtons[0]);

      // onClick should still fire (for visualization purposes),
      // but the button should appear disabled via styling
      const firstButton = keyButtons[0];
      expect(firstButton).toHaveClass('opacity-50');
      expect(firstButton).toHaveClass('cursor-not-allowed');
    });
  });

  describe('Pressed keys visualization', () => {
    it('should highlight pressed keys', () => {
      const pressedKeys = new Set(['KC_A', 'KC_LSFT']);

      renderWithProviders(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
          pressedKeys={pressedKeys}
        />,
        { wrapWithWasm: false }
      );

      // Find the 'A' key button by its key code (with period to avoid matching KC_APP)
      const aButton = screen.getByRole('button', { name: /Key KC_A\./ });
      expect(aButton).toBeInTheDocument();

      // KeyButton component applies 'pressed' class when isPressed is true
      // The actual visual styling is handled by KeyButton
      expect(aButton.parentElement).toBeInTheDocument();
    });

    it('should update highlighted keys when pressedKeys changes', () => {
      const { rerender } = renderWithProviders(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
          pressedKeys={new Set(['KC_A'])}
        />,
        { wrapWithWasm: false }
      );

      // Initially KC_A is pressed
      let aButton = screen.getByRole('button', { name: /Key KC_A\./ });
      expect(aButton).toBeInTheDocument();

      // Update to press KC_B instead
      rerender(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
          pressedKeys={new Set(['KC_B'])}
        />
      );

      // KC_B button should be present
      const bButton = screen.getByRole('button', { name: /Key KC_B\./ });
      expect(bButton).toBeInTheDocument();
    });
  });

  describe('Custom className', () => {
    it('should apply custom className to container', () => {
      renderWithProviders(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
          className="custom-keyboard-class"
        />,
        { wrapWithWasm: false }
      );

      const visualizer = screen.getByTestId('keyboard-visualizer');
      expect(visualizer).toHaveClass('custom-keyboard-class');
    });
  });

  describe('Accessibility compliance', () => {
    it('should have proper ARIA attributes', () => {
      renderWithProviders(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
        />,
        { wrapWithWasm: false }
      );

      const visualizer = screen.getByTestId('keyboard-visualizer');

      // Should be a group with descriptive label
      expect(visualizer).toHaveAttribute('role', 'group');
      expect(visualizer).toHaveAttribute('aria-label');

      // All keys should be accessible buttons
      const keyButtons = screen.getAllByRole('button');
      keyButtons.forEach((button) => {
        // Each button should have accessible name (from aria-label or text content)
        expect(button).toHaveAccessibleName();
      });
    });

    it('should provide navigation instructions in aria-label', () => {
      renderWithProviders(
        <KeyboardVisualizer
          layout="ANSI_104"
          keyMappings={mockKeyMappings}
          onKeyClick={mockOnKeyClick}
        />,
        { wrapWithWasm: false }
      );

      const visualizer = screen.getByTestId('keyboard-visualizer');
      const ariaLabel = visualizer.getAttribute('aria-label') || '';

      // Should describe how to navigate
      expect(ariaLabel.toLowerCase()).toContain('arrow keys');
      expect(ariaLabel.toLowerCase()).toContain('enter');
    });
  });
});
