/**
 * Unit tests for SimulationControls component
 *
 * Tests cover:
 * - Rendering with different states (running/stopped)
 * - Button callbacks (start, stop, clear)
 * - Disabled states
 * - Statistics display
 * - Accessibility (ARIA labels)
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  SimulationControls,
  type SimulationControlsProps,
} from './SimulationControls';

/**
 * Default props for testing
 */
const defaultProps: SimulationControlsProps = {
  isRunning: false,
  eventCount: 0,
  onStart: vi.fn(),
  onStop: vi.fn(),
  onClear: vi.fn(),
  statistics: {
    total: 0,
    pressCount: 0,
    releaseCount: 0,
    eventsPerSecond: 0,
  },
};

/**
 * Helper to render component with custom props
 */
function renderControls(props: Partial<SimulationControlsProps> = {}) {
  return render(<SimulationControls {...defaultProps} {...props} />);
}

describe('SimulationControls', () => {
  describe('Rendering', () => {
    it('should render Start button when not running', () => {
      renderControls({ isRunning: false });
      expect(
        screen.getByRole('button', { name: /start/i })
      ).toBeInTheDocument();
    });

    it('should render Stop button when running', () => {
      renderControls({ isRunning: true });
      expect(screen.getByRole('button', { name: /stop/i })).toBeInTheDocument();
    });

    it('should render Clear button', () => {
      renderControls();
      expect(
        screen.getByRole('button', { name: /clear/i })
      ).toBeInTheDocument();
    });

    it('should display statistics when events exist', () => {
      renderControls({
        eventCount: 10,
        statistics: {
          total: 10,
          pressCount: 5,
          releaseCount: 5,
          eventsPerSecond: 2,
        },
      });

      expect(screen.getByText('Statistics')).toBeInTheDocument();
      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('Press:')).toBeInTheDocument();
      expect(screen.getByText('Release:')).toBeInTheDocument();
      expect(screen.getByText('2')).toBeInTheDocument();
      // Both press and release are 5, so we should find exactly 2 elements
      const fiveElements = screen.getAllByText('5');
      expect(fiveElements).toHaveLength(2);
    });

    it('should not display statistics when no events', () => {
      renderControls({ eventCount: 0 });
      expect(screen.queryByText('Statistics')).not.toBeInTheDocument();
    });

    it('should show running indicator when simulation is active', () => {
      renderControls({ isRunning: true });
      expect(screen.getByText(/simulation running/i)).toBeInTheDocument();
    });

    it('should not show running indicator when stopped', () => {
      renderControls({ isRunning: false });
      expect(screen.queryByText(/simulation running/i)).not.toBeInTheDocument();
    });
  });

  describe('Button Interactions', () => {
    it('should call onStart when Start button is clicked', async () => {
      const onStart = vi.fn();
      const user = userEvent.setup();
      renderControls({ isRunning: false, onStart });

      await user.click(screen.getByRole('button', { name: /start/i }));

      expect(onStart).toHaveBeenCalledTimes(1);
    });

    it('should call onStop when Stop button is clicked', async () => {
      const onStop = vi.fn();
      const user = userEvent.setup();
      renderControls({ isRunning: true, onStop });

      await user.click(screen.getByRole('button', { name: /stop/i }));

      expect(onStop).toHaveBeenCalledTimes(1);
    });

    it('should call onClear when Clear button is clicked', async () => {
      const onClear = vi.fn();
      const user = userEvent.setup();
      renderControls({ eventCount: 10, onClear });

      await user.click(screen.getByRole('button', { name: /clear/i }));

      expect(onClear).toHaveBeenCalledTimes(1);
    });

    it('should not call onClear when button is disabled', async () => {
      const onClear = vi.fn();
      const user = userEvent.setup();
      renderControls({ eventCount: 0, onClear });

      const clearButton = screen.getByRole('button', { name: /clear/i });
      expect(clearButton).toBeDisabled();

      // Attempting to click a disabled button does nothing
      await user.click(clearButton);
      expect(onClear).not.toHaveBeenCalled();
    });
  });

  describe('Disabled States', () => {
    it('should disable Clear button when event count is 0', () => {
      renderControls({ eventCount: 0 });
      expect(screen.getByRole('button', { name: /clear/i })).toBeDisabled();
    });

    it('should enable Clear button when events exist', () => {
      renderControls({ eventCount: 5 });
      expect(screen.getByRole('button', { name: /clear/i })).toBeEnabled();
    });

    it('should not disable Start button', () => {
      renderControls({ isRunning: false });
      expect(screen.getByRole('button', { name: /start/i })).toBeEnabled();
    });

    it('should not disable Stop button', () => {
      renderControls({ isRunning: true });
      expect(screen.getByRole('button', { name: /stop/i })).toBeEnabled();
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA label for Start button', () => {
      renderControls({ isRunning: false });
      expect(
        screen.getByRole('button', { name: 'Start simulation' })
      ).toBeInTheDocument();
    });

    it('should have proper ARIA label for Stop button', () => {
      renderControls({ isRunning: true });
      expect(
        screen.getByRole('button', { name: 'Stop simulation' })
      ).toBeInTheDocument();
    });

    it('should have proper ARIA label for Clear button', () => {
      renderControls();
      expect(
        screen.getByRole('button', { name: 'Clear all events' })
      ).toBeInTheDocument();
    });

    it('should have region role for statistics', () => {
      renderControls({
        eventCount: 10,
        statistics: {
          total: 10,
          pressCount: 5,
          releaseCount: 5,
          eventsPerSecond: 2,
        },
      });

      expect(
        screen.getByRole('region', { name: /simulation statistics/i })
      ).toBeInTheDocument();
    });
  });

  describe('Statistics Display', () => {
    it('should display total event count', () => {
      renderControls({
        eventCount: 42,
        statistics: {
          total: 42,
          pressCount: 20,
          releaseCount: 22,
          eventsPerSecond: 5,
        },
      });

      expect(screen.getByText('Total Events:')).toBeInTheDocument();
      expect(screen.getByText('42')).toBeInTheDocument();
    });

    it('should display events per second', () => {
      renderControls({
        eventCount: 10,
        statistics: {
          total: 10,
          pressCount: 5,
          releaseCount: 5,
          eventsPerSecond: 3,
        },
      });

      expect(screen.getByText('Events/sec:')).toBeInTheDocument();
      expect(screen.getByText('3')).toBeInTheDocument();
    });

    it('should display press count', () => {
      renderControls({
        eventCount: 10,
        statistics: {
          total: 10,
          pressCount: 7,
          releaseCount: 3,
          eventsPerSecond: 2,
        },
      });

      expect(screen.getByText('Press:')).toBeInTheDocument();
      expect(screen.getByText('7')).toBeInTheDocument();
    });

    it('should display release count', () => {
      renderControls({
        eventCount: 10,
        statistics: {
          total: 10,
          pressCount: 4,
          releaseCount: 6,
          eventsPerSecond: 2,
        },
      });

      expect(screen.getByText('Release:')).toBeInTheDocument();
      expect(screen.getByText('6')).toBeInTheDocument();
    });

    it('should handle zero statistics', () => {
      renderControls({
        eventCount: 1,
        statistics: {
          total: 0,
          pressCount: 0,
          releaseCount: 0,
          eventsPerSecond: 0,
        },
      });

      expect(screen.getByText('Statistics')).toBeInTheDocument();
      const zeros = screen.getAllByText('0');
      expect(zeros.length).toBeGreaterThanOrEqual(4);
    });
  });

  describe('Custom className', () => {
    it('should apply custom className', () => {
      const { container } = renderControls({ className: 'custom-class' });
      expect(container.firstChild).toHaveClass('custom-class');
    });

    it('should work without custom className', () => {
      const { container } = renderControls();
      expect(container.firstChild).toHaveClass('flex');
    });
  });
});
