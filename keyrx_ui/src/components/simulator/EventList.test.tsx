/**
 * Unit tests for EventList component
 *
 * Tests rendering, virtualization, auto-scroll, callbacks, and accessibility.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { EventList } from './EventList';
import type { KeyEvent } from '@/types/rpc';

/**
 * Create a mock KeyEvent for testing
 */
function createMockEvent(
  overrides?: Partial<KeyEvent>
): KeyEvent {
  return {
    timestamp: Date.now() * 1000,
    keyCode: 'KEY_A',
    eventType: 'press',
    input: 'KEY_A',
    output: 'KEY_A',
    latency: 150,
    ...overrides,
  };
}

/**
 * Create multiple mock events
 */
function createMockEvents(count: number): KeyEvent[] {
  return Array.from({ length: count }, (_, i) =>
    createMockEvent({
      timestamp: (Date.now() - i * 1000) * 1000, // 1 second apart
      keyCode: `KEY_${String.fromCharCode(65 + (i % 26))}`,
      eventType: i % 2 === 0 ? 'press' : 'release',
      input: `KEY_${String.fromCharCode(65 + (i % 26))}`,
      output: `KEY_${String.fromCharCode(65 + (i % 26))}`,
      latency: 100 + i,
    })
  );
}

describe('EventList', () => {
  const mockOnClear = vi.fn();

  beforeEach(() => {
    mockOnClear.mockClear();
  });

  describe('Rendering', () => {
    it('renders empty state when no events', () => {
      render(
        <EventList events={[]} maxEvents={1000} onClear={mockOnClear} />
      );

      expect(
        screen.getByText('No events yet. Press a key to start.')
      ).toBeInTheDocument();
      expect(screen.getByText('(0 / 1000 events)')).toBeInTheDocument();
    });

    it('renders event count correctly', () => {
      const events = createMockEvents(5);
      render(
        <EventList
          events={events}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      expect(screen.getByText('(5 / 1000 events)')).toBeInTheDocument();
    });

    it('renders events with correct formatting', () => {
      const event = createMockEvent({
        eventType: 'press',
        input: 'KEY_A',
        output: 'KEY_B',
        latency: 250,
      });

      render(
        <EventList
          events={[event]}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      expect(screen.getByText('PRESS')).toBeInTheDocument();
      expect(
        screen.getByText('KEY_A → KEY_B (250μs)')
      ).toBeInTheDocument();
    });

    it('renders multiple events', () => {
      const events = createMockEvents(10);
      render(
        <EventList
          events={events}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      // Check that multiple event types are rendered
      expect(screen.getAllByText(/PRESS|RELEASE/)).toHaveLength(10);
    });

    it('applies correct color class for press events', () => {
      const event = createMockEvent({ eventType: 'press' });
      render(
        <EventList
          events={[event]}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      const eventType = screen.getByText('PRESS');
      expect(eventType).toHaveClass('text-green-400');
    });

    it('applies correct color class for release events', () => {
      const event = createMockEvent({ eventType: 'release' });
      render(
        <EventList
          events={[event]}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      const eventType = screen.getByText('RELEASE');
      expect(eventType).toHaveClass('text-red-400');
    });
  });

  describe('Virtualization', () => {
    it('uses non-virtualized list when below threshold', () => {
      const events = createMockEvents(50);
      const { container } = render(
        <EventList
          events={events}
          maxEvents={1000}
          onClear={mockOnClear}
          virtualizeThreshold={100}
        />
      );

      // Non-virtualized list should have overflow-y-auto
      const listContainer = container.querySelector('.overflow-y-auto');
      expect(listContainer).toBeInTheDocument();
    });

    it('uses virtualized list when above threshold', () => {
      const events = createMockEvents(150);
      render(
        <EventList
          events={events}
          maxEvents={1000}
          onClear={mockOnClear}
          virtualizeThreshold={100}
        />
      );

      // Virtualized list should be present (react-window creates a div with specific styles)
      // We can't easily test the internal structure, but we can verify events are rendered
      expect(screen.getAllByText(/PRESS|RELEASE/).length).toBeGreaterThan(0);
    });

    it('respects custom virtualization threshold', () => {
      const events = createMockEvents(30);
      const { container } = render(
        <EventList
          events={events}
          maxEvents={1000}
          onClear={mockOnClear}
          virtualizeThreshold={20}
        />
      );

      // With threshold of 20 and 30 events, should virtualize
      // We check for non-virtualized container NOT being present
      const nonVirtualizedList = container.querySelector('.overflow-y-auto');
      expect(nonVirtualizedList).not.toBeInTheDocument();
    });
  });

  describe('Clear button', () => {
    it('calls onClear when clear button clicked', async () => {
      const user = userEvent.setup();
      const events = createMockEvents(5);

      render(
        <EventList
          events={events}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      const clearButton = screen.getByRole('button', { name: /clear/i });
      await user.click(clearButton);

      expect(mockOnClear).toHaveBeenCalledTimes(1);
    });

    it('disables clear button when no events', () => {
      render(
        <EventList events={[]} maxEvents={1000} onClear={mockOnClear} />
      );

      const clearButton = screen.getByRole('button', { name: /clear/i });
      expect(clearButton).toBeDisabled();
    });

    it('enables clear button when events exist', () => {
      const events = createMockEvents(5);
      render(
        <EventList
          events={events}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      const clearButton = screen.getByRole('button', { name: /clear/i });
      expect(clearButton).not.toBeDisabled();
    });
  });

  describe('Accessibility', () => {
    it('has proper ARIA labels for table structure', () => {
      const events = createMockEvents(5);
      render(
        <EventList
          events={events}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      const table = screen.getByRole('table', { name: /key event log/i });
      expect(table).toBeInTheDocument();
      expect(table).toHaveAttribute('aria-rowcount', '5');
    });

    it('has proper ARIA label for clear button', () => {
      const events = createMockEvents(5);
      render(
        <EventList
          events={events}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      const clearButton = screen.getByRole('button', {
        name: 'Clear event log',
      });
      expect(clearButton).toBeInTheDocument();
    });

    it('has status role for empty state', () => {
      render(
        <EventList events={[]} maxEvents={1000} onClear={mockOnClear} />
      );

      const emptyState = screen.getByRole('status');
      expect(emptyState).toHaveTextContent(
        'No events yet. Press a key to start.'
      );
    });
  });

  describe('Timestamp formatting', () => {
    it('formats timestamp correctly', () => {
      const now = new Date('2024-01-15T14:30:45.000Z');
      const event = createMockEvent({
        timestamp: now.getTime() * 1000, // Convert to microseconds
      });

      render(
        <EventList
          events={[event]}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      // Timestamp should be formatted as HH:MM:SS in user's locale
      // We just check it exists and has the expected format pattern
      const timestamps = screen.getAllByText(/\d{1,2}:\d{2}:\d{2}/);
      expect(timestamps.length).toBeGreaterThan(0);
    });
  });

  describe('Edge cases', () => {
    it('handles max events limit display', () => {
      const events = createMockEvents(1500);
      render(
        <EventList
          events={events}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      // Should show min(events.length, maxEvents)
      expect(screen.getByText('(1000 / 1000 events)')).toBeInTheDocument();
    });

    it('handles events with high latency', () => {
      const event = createMockEvent({
        latency: 999999,
      });

      render(
        <EventList
          events={[event]}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      expect(screen.getByText(/999999μs/)).toBeInTheDocument();
    });

    it('handles events with identical timestamps', () => {
      const timestamp = Date.now() * 1000;
      const events = [
        createMockEvent({ timestamp }),
        createMockEvent({ timestamp }),
        createMockEvent({ timestamp }),
      ];

      render(
        <EventList
          events={events}
          maxEvents={1000}
          onClear={mockOnClear}
        />
      );

      // All events should render despite identical timestamps
      expect(screen.getAllByText(/PRESS|RELEASE/)).toHaveLength(3);
    });
  });
});
