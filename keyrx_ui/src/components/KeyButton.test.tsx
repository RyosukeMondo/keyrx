import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { KeyButton, KeyMapping } from './KeyButton';

describe('KeyButton', () => {
  it('renders with label', () => {
    const onClick = vi.fn();
    renderWithProviders(<KeyButton keyCode="A" label="A" onClick={onClick} />);
    expect(screen.getByText('A')).toBeInTheDocument();
  });

  it('calls onClick when clicked', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    renderWithProviders(<KeyButton keyCode="A" label="A" onClick={onClick} />);

    await user.click(screen.getByRole('button'));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('renders without mapping (default state)', () => {
    const onClick = vi.fn();
    renderWithProviders(<KeyButton keyCode="A" label="A" onClick={onClick} />);

    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-slate-700/50');
    expect(button).toHaveClass('border-dashed');
    expect(button).toHaveAttribute(
      'aria-label',
      expect.stringContaining('Default')
    );
  });

  it('renders with simple mapping', () => {
    const onClick = vi.fn();
    const mapping: KeyMapping = {
      type: 'simple',
      tapAction: 'B',
    };

    renderWithProviders(
      <KeyButton keyCode="A" label="A" mapping={mapping} onClick={onClick} />
    );

    const button = screen.getByRole('button');
    // Simple mapping uses default background
    expect(button).toHaveClass('bg-slate-700');
  });

  it('renders with tap_hold mapping', () => {
    const onClick = vi.fn();
    const mapping: KeyMapping = {
      type: 'tap_hold',
      tapAction: 'A',
      holdAction: 'Ctrl',
      threshold: 200,
    };

    renderWithProviders(
      <KeyButton keyCode="A" label="A" mapping={mapping} onClick={onClick} />
    );

    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-red-900/15');
    expect(button).toHaveClass('border-red-500');
    expect(button).toHaveAttribute(
      'aria-label',
      expect.stringContaining('Tap: A, Hold: Ctrl')
    );
  });

  it('renders with macro mapping', () => {
    const onClick = vi.fn();
    const mapping: KeyMapping = {
      type: 'macro',
      macroSteps: [
        { type: 'press', key: 'A' },
        { type: 'delay', delay: 100 },
        { type: 'release', key: 'A' },
      ],
    };

    renderWithProviders(
      <KeyButton keyCode="A" label="A" mapping={mapping} onClick={onClick} />
    );

    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-purple-900/15');
    expect(button).toHaveClass('border-purple-500');
    expect(button).toHaveAttribute(
      'aria-label',
      expect.stringContaining('Macro (3 steps)')
    );
  });

  it('renders with layer_switch mapping', () => {
    const onClick = vi.fn();
    const mapping: KeyMapping = {
      type: 'layer_switch',
      targetLayer: 'gaming',
    };

    renderWithProviders(
      <KeyButton keyCode="A" label="A" mapping={mapping} onClick={onClick} />
    );

    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-yellow-900/15');
    expect(button).toHaveClass('border-yellow-500');
    expect(button).toHaveAttribute(
      'aria-label',
      expect.stringContaining('Layer: gaming')
    );
  });

  it('renders pressed state', () => {
    const onClick = vi.fn();
    renderWithProviders(
      <KeyButton keyCode="A" label="A" onClick={onClick} isPressed={true} />
    );

    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-green-500');
  });

  it('applies custom className', () => {
    const onClick = vi.fn();
    renderWithProviders(
      <KeyButton
        keyCode="A"
        label="A"
        onClick={onClick}
        className="custom-class"
      />
    );

    const button = screen.getByRole('button');
    expect(button).toHaveClass('custom-class');
  });

  it('has proper accessibility attributes', () => {
    const onClick = vi.fn();
    renderWithProviders(<KeyButton keyCode="A" label="A" onClick={onClick} />);

    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('aria-label');
    expect(button.getAttribute('aria-label')).toContain('Key A');
    expect(button.getAttribute('aria-label')).toContain('Click to configure');
  });

  it('has hover and focus styles', () => {
    const onClick = vi.fn();
    renderWithProviders(<KeyButton keyCode="A" label="A" onClick={onClick} />);

    const button = screen.getByRole('button');
    expect(button).toHaveClass(
      'hover:brightness-110',
      'hover:-translate-y-0.5',
      'hover:shadow-lg'
    );
    expect(button).toHaveClass(
      'focus:outline',
      'focus:outline-2',
      'focus:outline-primary-500'
    );
  });
});
