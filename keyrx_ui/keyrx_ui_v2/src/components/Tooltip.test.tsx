import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Tooltip } from './Tooltip';

describe('Tooltip', () => {

  it('renders children correctly', () => {
    render(
      <Tooltip content="Tooltip text">
        <button>Hover me</button>
      </Tooltip>
    );

    expect(screen.getByRole('button', { name: /hover me/i })).toBeInTheDocument();
  });

  it('shows tooltip after delay on hover', async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Tooltip text" delay={100}>
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');

    // Tooltip should not be visible initially
    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();

    await user.hover(trigger);

    // Wait for tooltip to appear after delay
    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
      expect(screen.getByRole('tooltip')).toHaveTextContent('Tooltip text');
    });
  });

  it('shows tooltip instantly on keyboard focus', async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Tooltip text" delay={500}>
        <button>Focus me</button>
      </Tooltip>
    );

    await user.tab();

    // Tooltip should appear immediately on focus (no delay)
    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });
  });

  it('hides tooltip on mouse leave', async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Tooltip text" delay={100}>
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    await user.hover(trigger);

    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });

    await user.unhover(trigger);

    await waitFor(() => {
      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
    });
  });

  it('hides tooltip on blur', async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Tooltip text">
        <button>Focus me</button>
      </Tooltip>
    );

    await user.tab();

    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });

    // Tab away to blur
    await user.tab();

    await waitFor(() => {
      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
    });
  });

  it('does not show tooltip when disabled', async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Tooltip text" disabled delay={100}>
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    await user.hover(trigger);

    // Wait a bit to ensure tooltip doesn't appear
    await new Promise(resolve => setTimeout(resolve, 200));

    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  });

  it('sets aria-describedby when tooltip is visible', async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Tooltip text" delay={100}>
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    await user.hover(trigger);

    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });

    // After tooltip appears, aria-describedby should be set on the wrapper
    const tooltip = screen.getByRole('tooltip');
    const wrapper = trigger.parentElement;
    expect(wrapper).toHaveAttribute('aria-describedby', tooltip.id);
  });

  it('cleans up timeout on unmount', async () => {
    const user = userEvent.setup();
    const { unmount } = render(
      <Tooltip content="Tooltip text" delay={100}>
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    await user.hover(trigger);

    // Unmount before timeout completes
    unmount();

    // Wait to ensure tooltip doesn't appear (no memory leak)
    await new Promise(resolve => setTimeout(resolve, 200));

    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  });

  it('cancels timeout if mouse leaves before delay', async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Tooltip text" delay={200}>
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    await user.hover(trigger);

    // Wait less than the delay
    await new Promise(resolve => setTimeout(resolve, 50));

    await user.unhover(trigger);

    // Wait past the original delay time
    await new Promise(resolve => setTimeout(resolve, 200));

    // Tooltip should not appear
    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  });

  it('supports custom className', async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Tooltip text" delay={100} className="custom-class">
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    await user.hover(trigger);

    await waitFor(() => {
      const tooltip = screen.getByRole('tooltip');
      expect(tooltip).toHaveClass('custom-class');
    });
  });

  it('supports ReactNode content', async () => {
    const user = userEvent.setup();
    render(
      <Tooltip
        content={
          <div>
            <strong>Bold text</strong>
            <span>and regular text</span>
          </div>
        }
        delay={100}
      >
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    await user.hover(trigger);

    await waitFor(() => {
      const tooltip = screen.getByRole('tooltip');
      expect(tooltip.querySelector('strong')).toHaveTextContent('Bold text');
      expect(tooltip.querySelector('span')).toHaveTextContent('and regular text');
    });
  });

  it('uses correct default delay of 500ms', async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Tooltip text">
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    await user.hover(trigger);

    // Should not appear immediately
    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();

    // Should appear after delay
    await waitFor(
      () => {
        expect(screen.getByRole('tooltip')).toBeInTheDocument();
      },
      { timeout: 1000 }
    );
  });
});
