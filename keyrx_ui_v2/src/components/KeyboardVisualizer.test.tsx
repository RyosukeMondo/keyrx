import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { KeyboardVisualizer } from './KeyboardVisualizer';
import { KeyMapping } from './KeyButton';

describe('KeyboardVisualizer', () => {
  const mockOnKeyClick = vi.fn();
  const defaultProps = {
    layout: 'ANSI_104' as const,
    keyMappings: new Map<string, KeyMapping>(),
    onKeyClick: mockOnKeyClick,
  };

  it('renders keyboard with all keys', () => {
    render(<KeyboardVisualizer {...defaultProps} />);

    // Check for some key labels
    expect(
      screen.getByRole('button', { name: /Key KC_ESC\./ })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Key KC_A\./ })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Key KC_SPC\./ })
    ).toBeInTheDocument();
  });

  it('calls onKeyClick when key is clicked', async () => {
    const user = userEvent.setup();
    render(<KeyboardVisualizer {...defaultProps} />);

    const escKey = screen.getByRole('button', { name: /Key KC_ESC\./ });
    await user.click(escKey);

    expect(mockOnKeyClick).toHaveBeenCalledWith('KC_ESC');
  });

  it('shows custom mapping with blue tint', () => {
    const keyMappings = new Map<string, KeyMapping>([
      [
        'KC_A',
        {
          type: 'tap_hold',
          tapAction: 'KC_A',
          holdAction: 'KC_LCTL',
          threshold: 200,
        },
      ],
    ]);

    render(<KeyboardVisualizer {...defaultProps} keyMappings={keyMappings} />);

    const aKey = screen.getByRole('button', { name: /Key KC_A\./ });
    expect(aKey).toHaveClass('bg-blue-700');
  });

  it('shows pressed state in simulator mode', () => {
    const pressedKeys = new Set(['KC_A', 'KC_LSFT']);

    render(
      <KeyboardVisualizer
        {...defaultProps}
        simulatorMode={true}
        pressedKeys={pressedKeys}
      />
    );

    const aKey = screen.getByRole('button', { name: /Key KC_A\./ });
    expect(aKey).toHaveClass('bg-green-500');
  });

  it('renders with correct grid layout', () => {
    const { container } = render(<KeyboardVisualizer {...defaultProps} />);

    const grid = container.querySelector('.keyboard-grid');
    expect(grid).toHaveStyle({
      display: 'grid',
    });
  });

  it('applies custom className', () => {
    const { container } = render(
      <KeyboardVisualizer {...defaultProps} className="custom-class" />
    );

    const grid = container.querySelector('.keyboard-grid');
    expect(grid).toHaveClass('custom-class');
  });

  it('displays tooltip on hover', async () => {
    const user = userEvent.setup();
    const keyMappings = new Map<string, KeyMapping>([
      [
        'KC_A',
        {
          type: 'tap_hold',
          tapAction: 'KC_A',
          holdAction: 'KC_LCTL',
          threshold: 200,
        },
      ],
    ]);

    render(<KeyboardVisualizer {...defaultProps} keyMappings={keyMappings} />);

    const aKey = screen.getByRole('button', { name: /Key KC_A\./ });
    await user.hover(aKey);

    // Tooltip should appear after delay (tested in Tooltip.test.tsx)
    // Here we just verify the aria-label contains mapping info
    expect(aKey).toHaveAccessibleName(/Tap: KC_A, Hold: KC_LCTL/);
  });

  it('handles keyboard navigation', async () => {
    const user = userEvent.setup();
    render(<KeyboardVisualizer {...defaultProps} />);

    const escKey = screen.getByRole('button', { name: /Key KC_ESC\./ });

    // Tab to focus key
    await user.tab();
    expect(escKey).toHaveFocus();

    // Enter to click
    await user.keyboard('{Enter}');
    expect(mockOnKeyClick).toHaveBeenCalledWith('KC_ESC');
  });

  it('renders wide keys with correct span', () => {
    render(<KeyboardVisualizer {...defaultProps} />);

    // Space bar should span multiple columns
    const spaceKey = screen.getByRole('button', { name: /Key KC_SPC\./ });
    const spaceContainer = spaceKey.parentElement?.parentElement;

    // Space key spans 7 columns (w: 6.25 rounded up)
    expect(spaceContainer?.style.gridColumn).toContain('span');
  });
});
