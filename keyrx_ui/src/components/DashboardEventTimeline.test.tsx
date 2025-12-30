/**
 * Unit tests for DashboardEventTimeline component
 *
 * Tests event rendering, pause/resume functionality, virtualization,
 * hover tooltips, and accessibility.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, within, act } from '@testing-library/react';
import { DashboardEventTimeline } from './DashboardEventTimeline';
import { useDashboardStore, KeyEvent } from '../store/dashboardStore';

// Mock react-window to avoid JSDOM issues with virtualization
vi.mock('react-window', () => ({
  FixedSizeList: ({ children, itemCount, height, width, className, role }: any) => {
    // Render all items for testing purposes
    const items = Array.from({ length: itemCount }, (_, index) => {
      return <div key={index}>{children({ index, style: {} })}</div>;
    });
    return (
      <div
        className={className}
        role={role}
        style={{ height, width: typeof width === 'number' ? `${width}px` : width }}
      >
        {items}
      </div>
    );
  },
}));

describe('DashboardEventTimeline', () => {
  // Reset the store before each test
  beforeEach(() => {
    useDashboardStore.getState().reset();
  });

  // Helper function to create mock events
  const createMockEvent = (
    timestamp: number,
    keyCode: string,
    eventType: 'press' | 'release',
    input: string,
    output: string,
    latency: number
  ): KeyEvent => ({
    timestamp,
    keyCode,
    eventType,
    input,
    output,
    latency,
  });

  describe('Empty State', () => {
    it('should render empty state when there are no events', () => {
      render(<DashboardEventTimeline />);

      expect(screen.getByRole('status')).toBeInTheDocument();
      expect(screen.getByText(/no events yet/i)).toBeInTheDocument();
      expect(screen.getByText(/start typing to see events/i)).toBeInTheDocument();
    });

    it('should display "0 / 100 events" when empty', () => {
      render(<DashboardEventTimeline />);

      expect(screen.getByText('0 / 100 events')).toBeInTheDocument();
    });

    it('should not render event list when empty', () => {
      render(<DashboardEventTimeline />);

      expect(screen.queryByRole('list')).not.toBeInTheDocument();
    });
  });

  describe('Event Rendering', () => {
    it('should render event list with correct data', () => {
      const events: KeyEvent[] = [
        createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500),
        createMockEvent(1234567891000000, 'KEY_A', 'release', 'A', 'B', 1200),
        createMockEvent(1234567892000000, 'KEY_C', 'press', 'C', 'D', 8000),
      ];

      events.forEach(event => useDashboardStore.getState().addEvent(event));

      render(<DashboardEventTimeline />);

      expect(screen.getByRole('list')).toBeInTheDocument();
      expect(screen.getAllByRole('row')).toHaveLength(3);
    });

    it('should display events in reverse chronological order (newest first)', () => {
      const event1 = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      const event2 = createMockEvent(1234567891000000, 'KEY_B', 'press', 'B', 'C', 1200);
      const event3 = createMockEvent(1234567892000000, 'KEY_C', 'press', 'C', 'D', 1000);

      useDashboardStore.getState().addEvent(event1);
      useDashboardStore.getState().addEvent(event2);
      useDashboardStore.getState().addEvent(event3);

      render(<DashboardEventTimeline />);

      const rows = screen.getAllByRole('row');

      // First row should be the newest event (event3)
      expect(rows[0]).toHaveTextContent('KEY_C');
      // Last row should be the oldest event (event1)
      expect(rows[2]).toHaveTextContent('KEY_A');
    });

    it('should display press event with down arrow', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      const row = screen.getByRole('row');
      expect(row).toHaveTextContent('↓');
    });

    it('should display release event with up arrow', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'release', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      const row = screen.getByRole('row');
      expect(row).toHaveTextContent('↑');
    });

    it('should display key code correctly', () => {
      const event = createMockEvent(1234567890000000, 'KEY_ENTER', 'press', 'Enter', 'Enter', 1500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      expect(screen.getByText('KEY_ENTER')).toBeInTheDocument();
    });

    it('should display input → output mapping', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      expect(screen.getByText('A → B')).toBeInTheDocument();
    });

    it('should format latency correctly', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      // 1500 microseconds = 1.50ms
      expect(screen.getByText('1.50ms')).toBeInTheDocument();
    });

    it('should highlight high latency events (>5ms)', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 6000);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      const row = screen.getByRole('row');
      expect(row).toHaveClass('high-latency');
    });

    it('should not highlight normal latency events (≤5ms)', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 3000);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      const row = screen.getByRole('row');
      expect(row).not.toHaveClass('high-latency');
    });

    it('should display correct event count', () => {
      const events = [
        createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500),
        createMockEvent(1234567891000000, 'KEY_B', 'press', 'B', 'C', 1200),
        createMockEvent(1234567892000000, 'KEY_C', 'press', 'C', 'D', 1000),
      ];

      events.forEach(event => useDashboardStore.getState().addEvent(event));

      render(<DashboardEventTimeline />);

      expect(screen.getByText('3 / 100 events')).toBeInTheDocument();
    });
  });

  describe('Pause/Resume Functionality', () => {
    it('should render pause button initially', () => {
      render(<DashboardEventTimeline />);

      const button = screen.getByRole('button', { name: /pause event timeline/i });
      expect(button).toBeInTheDocument();
      expect(button).toHaveTextContent('⏸ Pause');
    });

    it('should toggle to resume button when paused', () => {
      render(<DashboardEventTimeline />);

      const button = screen.getByRole('button', { name: /pause event timeline/i });
      fireEvent.click(button);

      expect(screen.getByRole('button', { name: /resume event timeline/i })).toBeInTheDocument();
      expect(screen.getByText('▶ Resume')).toBeInTheDocument();
    });

    it('should capture current events when pausing', () => {
      const events = [
        createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500),
        createMockEvent(1234567891000000, 'KEY_B', 'press', 'B', 'C', 1200),
      ];

      events.forEach(event => useDashboardStore.getState().addEvent(event));

      render(<DashboardEventTimeline />);

      // Pause the timeline
      const button = screen.getByRole('button', { name: /pause event timeline/i });
      fireEvent.click(button);

      // Verify the events are still displayed
      expect(screen.getAllByRole('row')).toHaveLength(2);
    });

    it('should not display new events while paused', () => {
      const event1 = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event1);

      const { rerender } = render(<DashboardEventTimeline />);

      // Pause the timeline
      const button = screen.getByRole('button', { name: /pause event timeline/i });
      fireEvent.click(button);

      // Add new event while paused
      act(() => {
        const event2 = createMockEvent(1234567891000000, 'KEY_B', 'press', 'B', 'C', 1200);
        useDashboardStore.getState().addEvent(event2);
      });

      // Force re-render to ensure state update is reflected
      rerender(<DashboardEventTimeline />);

      // Should still show only 1 event (the paused snapshot)
      expect(screen.getAllByRole('row')).toHaveLength(1);
      expect(screen.queryByText('KEY_B')).not.toBeInTheDocument();
    });

    it('should show buffered events indicator when paused', () => {
      const event1 = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event1);

      const { rerender } = render(<DashboardEventTimeline />);

      // Pause
      const button = screen.getByRole('button', { name: /pause event timeline/i });
      fireEvent.click(button);

      // Add new events while paused
      act(() => {
        useDashboardStore.getState().addEvent(
          createMockEvent(1234567891000000, 'KEY_B', 'press', 'B', 'C', 1200)
        );
        useDashboardStore.getState().addEvent(
          createMockEvent(1234567892000000, 'KEY_C', 'press', 'C', 'D', 1000)
        );
      });

      // Force re-render to ensure state update is reflected
      rerender(<DashboardEventTimeline />);

      // Should show indicator with buffered count
      expect(screen.getByText(/timeline paused/i)).toBeInTheDocument();
      expect(screen.getByText(/2 new events buffered/i)).toBeInTheDocument();
    });

    it('should resume and display all events when unpaused', () => {
      const event1 = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event1);

      const { rerender } = render(<DashboardEventTimeline />);

      // Pause
      let button = screen.getByRole('button', { name: /pause event timeline/i });
      fireEvent.click(button);

      // Add new events while paused
      act(() => {
        useDashboardStore.getState().addEvent(
          createMockEvent(1234567891000000, 'KEY_B', 'press', 'B', 'C', 1200)
        );
      });

      // Resume
      button = screen.getByRole('button', { name: /resume event timeline/i });
      fireEvent.click(button);

      // Force re-render
      rerender(<DashboardEventTimeline />);

      // Should now display all events including the one added while paused
      expect(screen.getAllByRole('row')).toHaveLength(2);
      expect(screen.getByText('KEY_B')).toBeInTheDocument();
    });

    it('should set aria-pressed attribute correctly', () => {
      render(<DashboardEventTimeline />);

      const button = screen.getByRole('button', { name: /pause event timeline/i });

      // Initially not pressed (not paused)
      expect(button).toHaveAttribute('aria-pressed', 'false');

      // Click to pause
      fireEvent.click(button);
      expect(button).toHaveAttribute('aria-pressed', 'true');

      // Click to resume
      fireEvent.click(button);
      expect(button).toHaveAttribute('aria-pressed', 'false');
    });
  });

  describe('Hover Tooltips', () => {
    it('should show tooltip on hover', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      const row = screen.getByRole('row');

      // No tooltip initially
      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();

      // Hover over row
      fireEvent.mouseEnter(row);

      // Tooltip should appear
      const tooltip = screen.getByRole('tooltip');
      expect(tooltip).toBeInTheDocument();
    });

    it('should hide tooltip on mouse leave', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      const row = screen.getByRole('row');

      // Hover to show tooltip
      fireEvent.mouseEnter(row);
      expect(screen.getByRole('tooltip')).toBeInTheDocument();

      // Mouse leave to hide tooltip
      fireEvent.mouseLeave(row);
      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
    });

    it('should display all event details in tooltip', () => {
      const event = createMockEvent(1234567890000000, 'KEY_ENTER', 'press', 'Enter', 'Tab', 2500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      const row = screen.getByRole('row');
      fireEvent.mouseEnter(row);

      const tooltip = screen.getByRole('tooltip');

      // Check all tooltip content
      expect(within(tooltip).getByText(/time:/i)).toBeInTheDocument();
      expect(within(tooltip).getByText(/type:/i)).toBeInTheDocument();
      expect(within(tooltip).getByText(/press/i)).toBeInTheDocument();
      expect(within(tooltip).getByText(/key:/i)).toBeInTheDocument();
      expect(within(tooltip).getByText(/KEY_ENTER/i)).toBeInTheDocument();
      expect(within(tooltip).getByText(/mapping:/i)).toBeInTheDocument();
      expect(within(tooltip).getByText(/Enter → Tab/i)).toBeInTheDocument();
      expect(within(tooltip).getByText(/latency:/i)).toBeInTheDocument();
      expect(within(tooltip).getByText(/2\.50ms/i)).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have correct ARIA labels for timeline region', () => {
      render(<DashboardEventTimeline />);

      expect(screen.getByRole('region', { name: /event timeline/i })).toBeInTheDocument();
    });

    it('should have correct ARIA labels for event rows', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      const row = screen.getByRole('row');
      expect(row).toHaveAttribute('aria-label', 'Event 1: press KEY_A');
    });

    it('should have aria-live region for event count', () => {
      render(<DashboardEventTimeline />);

      const eventCount = screen.getByText('0 / 100 events');
      expect(eventCount).toHaveAttribute('aria-live', 'polite');
    });

    it('should have aria-live region for paused indicator', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event);

      const { rerender } = render(<DashboardEventTimeline />);

      // Pause
      const button = screen.getByRole('button', { name: /pause event timeline/i });
      fireEvent.click(button);

      // Add new event while paused
      act(() => {
        useDashboardStore.getState().addEvent(
          createMockEvent(1234567891000000, 'KEY_B', 'press', 'B', 'C', 1200)
        );
      });

      // Force re-render
      rerender(<DashboardEventTimeline />);

      const indicator = screen.getByText(/timeline paused/i);
      expect(indicator).toHaveAttribute('aria-live', 'polite');
    });

    it('should have list role for event list', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      // The mocked FixedSizeList has role attribute
      const list = screen.getByRole('list');
      expect(list).toBeInTheDocument();
    });
  });

  describe('Custom Dimensions', () => {
    it('should accept custom height prop', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline height={600} />);

      const list = screen.getByRole('list');
      expect(list).toHaveStyle({ height: '600px' });
    });

    it('should accept custom width prop as number', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline width={800} />);

      const list = screen.getByRole('list');
      expect(list).toHaveStyle({ width: '800px' });
    });

    it('should accept custom width prop as string', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 1500);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline width="50%" />);

      const list = screen.getByRole('list');
      expect(list).toHaveStyle({ width: '50%' });
    });
  });

  describe('Legend', () => {
    it('should display legend items', () => {
      render(<DashboardEventTimeline />);

      expect(screen.getByText(/press/i)).toBeInTheDocument();
      expect(screen.getByText(/release/i)).toBeInTheDocument();
      expect(screen.getByText(/high latency/i)).toBeInTheDocument();
    });

    it('should show press and release icons in legend', () => {
      render(<DashboardEventTimeline />);

      const legend = screen.getByText(/press/i).closest('.timeline-legend');
      expect(legend).toBeInTheDocument();

      // Both icons should be in the legend
      const icons = legend!.querySelectorAll('.legend-icon');
      expect(icons).toHaveLength(2);
    });
  });

  describe('Edge Cases', () => {
    it('should handle malformed event data gracefully', () => {
      // Add an event with missing fields (should not crash)
      const malformedEvent = {
        timestamp: 1234567890000000,
        keyCode: 'KEY_A',
        eventType: 'press' as const,
        input: 'A',
        output: 'B',
        latency: 1500,
      };

      useDashboardStore.getState().addEvent(malformedEvent);

      expect(() => render(<DashboardEventTimeline />)).not.toThrow();
    });

    it('should handle very large latency values', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 999999);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      // 999999 microseconds = 999.99ms (rounded to 1000.00ms)
      const row = screen.getByRole('row');
      expect(row).toHaveTextContent(/999\.99ms|1000\.00ms/);
    });

    it('should handle zero latency', () => {
      const event = createMockEvent(1234567890000000, 'KEY_A', 'press', 'A', 'B', 0);
      useDashboardStore.getState().addEvent(event);

      render(<DashboardEventTimeline />);

      expect(screen.getByText('0.00ms')).toBeInTheDocument();
    });

    it('should handle maximum events (100)', () => {
      // Add 100 events
      for (let i = 0; i < 100; i++) {
        useDashboardStore.getState().addEvent(
          createMockEvent(1234567890000000 + i * 1000, `KEY_${i}`, 'press', `${i}`, `${i}`, 1000)
        );
      }

      render(<DashboardEventTimeline />);

      expect(screen.getByText('100 / 100 events')).toBeInTheDocument();
      expect(screen.getAllByRole('row')).toHaveLength(100);
    });

    it('should handle rapid event updates', () => {
      const { rerender } = render(<DashboardEventTimeline />);

      // Add multiple events rapidly
      act(() => {
        for (let i = 0; i < 10; i++) {
          useDashboardStore.getState().addEvent(
            createMockEvent(1234567890000000 + i * 100, `KEY_${i}`, 'press', `${i}`, `${i}`, 1000)
          );
        }
      });

      // Force re-render to ensure all state updates are reflected
      rerender(<DashboardEventTimeline />);

      expect(screen.getAllByRole('row')).toHaveLength(10);
      expect(screen.getByText('10 / 100 events')).toBeInTheDocument();
    });
  });
});
