import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { KeyButton, KeyMapping } from './KeyButton';

describe('KeyButton', () => {
  it('renders with label', () => {
    const onClick = vi.fn();
    render(<KeyButton keyCode="A" label="A" onClick={onClick} />);
    expect(screen.getByText('A')).toBeInTheDocument();
  });

  it('calls onClick when clicked', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(<KeyButton keyCode="A" label="A" onClick={onClick} />);

    await user.click(screen.getByRole('button'));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('renders without mapping (default state)', () => {
    const onClick = vi.fn();
    render(<KeyButton keyCode="A" label="A" onClick={onClick} />);

    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-slate-700');
    expect(button).toHaveAttribute('aria-label', expect.stringContaining('Default'));
  });

  it('renders with simple mapping', () => {
    const onClick = vi.fn();
    const mapping: KeyMapping = {
      type: 'simple',
      tapAction: 'B',
    };

    render(<KeyButton keyCode="A" label="A" mapping={mapping} onClick={onClick} />);

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

    render(<KeyButton keyCode="A" label="A" mapping={mapping} onClick={onClick} />);

    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-blue-700');
    expect(button).toHaveAttribute('aria-label', expect.stringContaining('Tap: A, Hold: Ctrl'));
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

    render(<KeyButton keyCode="A" label="A" mapping={mapping} onClick={onClick} />);

    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-blue-700');
    expect(button).toHaveAttribute('aria-label', expect.stringContaining('Macro (3 steps)'));
  });

  it('renders with layer_switch mapping', () => {
    const onClick = vi.fn();
    const mapping: KeyMapping = {
      type: 'layer_switch',
      targetLayer: 'gaming',
    };

    render(<KeyButton keyCode="A" label="A" mapping={mapping} onClick={onClick} />);

    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-blue-700');
    expect(button).toHaveAttribute('aria-label', expect.stringContaining('Layer: gaming'));
  });

  it('renders pressed state', () => {
    const onClick = vi.fn();
    render(<KeyButton keyCode="A" label="A" onClick={onClick} isPressed={true} />);

    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-green-500');
  });

  it('applies custom className', () => {
    const onClick = vi.fn();
    render(<KeyButton keyCode="A" label="A" onClick={onClick} className="custom-class" />);

    const button = screen.getByRole('button');
    expect(button).toHaveClass('custom-class');
  });

  it('has proper accessibility attributes', () => {
    const onClick = vi.fn();
    render(<KeyButton keyCode="A" label="A" onClick={onClick} />);

    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('aria-label');
    expect(button.getAttribute('aria-label')).toContain('Key A');
    expect(button.getAttribute('aria-label')).toContain('Click to configure');
  });

  it('has hover and focus styles', () => {
    const onClick = vi.fn();
    render(<KeyButton keyCode="A" label="A" onClick={onClick} />);

    const button = screen.getByRole('button');
    expect(button).toHaveClass('hover:bg-slate-600', 'hover:scale-105');
    expect(button).toHaveClass('focus:outline', 'focus:outline-2', 'focus:outline-primary-500');
  });
});
