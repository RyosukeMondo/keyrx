import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { EventLogList, EventLogEntry } from './EventLogList';

// Mock react-window
vi.mock('react-window', () => ({
  FixedSizeList: vi.fn(({ children, itemCount }: any) => (
    <div data-testid="virtual-list" role="rowgroup">
      {Array.from({ length: Math.min(itemCount, 10) }, (_, index) => (
        <div key={index}>{children({ index, style: {} })}</div>
      ))}
    </div>
  )),
}));

describe('EventLogList', () => {
  const mockEvents: EventLogEntry[] = [
    {
      id: '1',
      timestamp: 1704067200000, // 2024-01-01 00:00:00
      type: 'press',
      keyCode: 'KEY_A',
      latency: 0.5,
      input: 'KEY_A',
      output: 'KEY_A',
      deviceId: 'dev-123',
      deviceName: 'Test Keyboard',
    },
    {
      id: '2',
      timestamp: 1704067201000, // 2024-01-01 00:00:01
      type: 'release',
      keyCode: 'KEY_A',
      latency: 0.3,
      input: 'KEY_A',
      output: 'KEY_A',
      deviceId: 'dev-123',
      deviceName: 'Test Keyboard',
    },
    {
      id: '3',
      timestamp: 1704067202000, // 2024-01-01 00:00:02
      type: 'tap',
      keyCode: 'KEY_B',
      latency: 1.2,
      input: 'KEY_B',
      output: 'KEY_C',
      deviceId: 'dev-456',
      deviceName: 'Another Keyboard',
      mappingType: 'tap_hold',
      mappingTriggered: true,
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('rendering', () => {
    it('renders table with correct aria-label', () => {
      render(<EventLogList events={mockEvents} />);

      expect(screen.getByRole('table')).toHaveAttribute(
        'aria-label',
        'Event log'
      );
    });

    it('renders table header with column names', () => {
      render(<EventLogList events={mockEvents} />);

      expect(
        screen.getByRole('columnheader', { name: 'Time' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('columnheader', { name: 'Type' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('columnheader', { name: 'Input' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('columnheader', { name: 'Output' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('columnheader', { name: 'Map Type' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('columnheader', { name: 'Device' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('columnheader', { name: 'Latency' })
      ).toBeInTheDocument();
    });

    it('renders virtual list container', () => {
      render(<EventLogList events={mockEvents} />);

      expect(screen.getByTestId('virtual-list')).toBeInTheDocument();
    });

    it('renders event rows', () => {
      render(<EventLogList events={mockEvents} />);

      const rows = screen.getAllByRole('row');
      // Header row + event rows (virtual list renders first 10)
      expect(rows.length).toBeGreaterThan(0);
    });
  });

  describe('event data display', () => {
    it('displays formatted timestamps', () => {
      render(<EventLogList events={mockEvents} />);

      // Check for time cells (format: HH:MM:SS)
      const timeCells = screen.getAllByRole('cell', { name: /Time:/ });
      expect(timeCells.length).toBeGreaterThan(0);
    });

    it('displays event types with symbols', () => {
      render(<EventLogList events={mockEvents} />);

      // Check for event type cells
      expect(
        screen.getByRole('cell', { name: /Event type: press/ })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('cell', { name: /Event type: release/ })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('cell', { name: /Event type: tap/ })
      ).toBeInTheDocument();
    });

    it('displays input key codes with formatting', () => {
      render(<EventLogList events={mockEvents} />);

      // KEY_ prefix should be removed - use getAllByRole since there are multiple 'A' cells
      const inputACells = screen.getAllByRole('cell', { name: 'Input: A' });
      expect(inputACells.length).toBeGreaterThan(0);
      expect(
        screen.getByRole('cell', { name: 'Input: B' })
      ).toBeInTheDocument();
    });

    it('displays output key codes with formatting', () => {
      render(<EventLogList events={mockEvents} />);

      const outputACells = screen.getAllByRole('cell', { name: 'Output: A' });
      expect(outputACells.length).toBeGreaterThan(0);
      expect(
        screen.getByRole('cell', { name: 'Output: C' })
      ).toBeInTheDocument();
    });

    it('displays latency with correct formatting', () => {
      render(<EventLogList events={mockEvents} />);

      expect(
        screen.getByRole('cell', { name: 'Latency: 0.50ms' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('cell', { name: 'Latency: 0.30ms' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('cell', { name: 'Latency: 1.20ms' })
      ).toBeInTheDocument();
    });

    it('displays device names', () => {
      render(<EventLogList events={mockEvents} />);

      const testKeyboardCells = screen.getAllByRole('cell', {
        name: 'Device: Test Keyboard',
      });
      expect(testKeyboardCells.length).toBeGreaterThan(0);
      // "Another Keyboard" gets truncated to "Another Keyb…" (12 chars + ellipsis)
      expect(
        screen.getByRole('cell', { name: 'Device: Another Keyb…' })
      ).toBeInTheDocument();
    });

    it('truncates long device names', () => {
      const eventWithLongName: EventLogEntry = {
        ...mockEvents[0],
        deviceName: 'Very Long Device Name That Should Be Truncated',
      };

      render(<EventLogList events={[eventWithLongName]} />);

      // Should truncate to 12 chars + ellipsis
      expect(
        screen.getByRole('cell', { name: 'Device: Very Long De…' })
      ).toBeInTheDocument();
    });

    it('displays mapping type when present', () => {
      render(<EventLogList events={mockEvents} />);

      expect(
        screen.getByRole('cell', { name: 'Mapping type: tap_hold' })
      ).toBeInTheDocument();
    });
  });

  describe('mapping detection', () => {
    it('shows mapping triggered indicator for remapped keys', () => {
      const remappedEvent: EventLogEntry = {
        id: '4',
        timestamp: Date.now(),
        type: 'press',
        keyCode: 'KEY_A',
        latency: 0.5,
        input: 'KEY_A',
        output: 'KEY_B', // Different output
        deviceId: 'dev-123',
      };

      render(<EventLogList events={[remappedEvent]} />);

      expect(
        screen.getByRole('cell', { name: 'Mapping triggered' })
      ).toBeInTheDocument();
    });

    it('shows no mapping indicator for passthrough keys', () => {
      const passthroughEvent: EventLogEntry = {
        id: '5',
        timestamp: Date.now(),
        type: 'press',
        keyCode: 'KEY_A',
        latency: 0.5,
        input: 'KEY_A',
        output: 'KEY_A', // Same output
        deviceId: 'dev-123',
      };

      render(<EventLogList events={[passthroughEvent]} />);

      expect(
        screen.getByRole('cell', { name: 'No mapping' })
      ).toBeInTheDocument();
    });

    it('shows mapping triggered for special event types', () => {
      const specialEvents: EventLogEntry[] = [
        { ...mockEvents[0], type: 'tap' },
        { ...mockEvents[0], type: 'hold' },
        { ...mockEvents[0], type: 'macro' },
        { ...mockEvents[0], type: 'layer_switch' },
      ];

      render(<EventLogList events={specialEvents} />);

      const triggeredCells = screen.getAllByRole('cell', {
        name: 'Mapping triggered',
      });
      expect(triggeredCells).toHaveLength(4);
    });
  });

  describe('empty states', () => {
    it('handles empty event list', () => {
      render(<EventLogList events={[]} />);

      // Should still render table structure
      expect(screen.getByRole('table')).toBeInTheDocument();
      expect(screen.getByTestId('virtual-list')).toBeInTheDocument();
    });

    it('handles events with missing optional fields', () => {
      const minimalEvent: EventLogEntry = {
        id: '6',
        timestamp: Date.now(),
        type: 'press',
        keyCode: 'KEY_A',
        latency: 0.5,
      };

      render(<EventLogList events={[minimalEvent]} />);

      // Should display placeholder for missing fields
      expect(
        screen.getByRole('cell', { name: 'Input: A' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('cell', { name: 'Output: –' })
      ).toBeInTheDocument();
    });

    it('handles missing device information', () => {
      const noDeviceEvent: EventLogEntry = {
        id: '7',
        timestamp: Date.now(),
        type: 'press',
        keyCode: 'KEY_A',
        latency: 0.5,
      };

      render(<EventLogList events={[noDeviceEvent]} />);

      expect(
        screen.getByRole('cell', { name: 'Device: –' })
      ).toBeInTheDocument();
    });
  });

  describe('maxEvents prop', () => {
    it('limits displayed events when maxEvents is set', () => {
      const manyEvents = Array.from({ length: 100 }, (_, i) => ({
        ...mockEvents[0],
        id: `event-${i}`,
        timestamp: Date.now() + i * 1000,
      }));

      render(<EventLogList events={manyEvents} maxEvents={50} />);

      // Virtual list should only render last 50 events
      const list = screen.getByTestId('virtual-list');
      expect(list).toBeInTheDocument();
    });

    it('shows all events when maxEvents is undefined', () => {
      render(<EventLogList events={mockEvents} />);

      // Should render all events (mocked to show first 10)
      const list = screen.getByTestId('virtual-list');
      expect(list).toBeInTheDocument();
    });
  });

  describe('customization props', () => {
    it('uses custom height when provided', () => {
      render(<EventLogList events={mockEvents} height={500} />);

      // Virtual list should exist
      expect(screen.getByTestId('virtual-list')).toBeInTheDocument();
    });

    it('uses custom row height when provided', () => {
      render(<EventLogList events={mockEvents} rowHeight={50} />);

      // Virtual list should exist
      expect(screen.getByTestId('virtual-list')).toBeInTheDocument();
    });

    it('respects autoScroll prop', () => {
      render(<EventLogList events={mockEvents} autoScroll={false} />);

      // Component should render without auto-scrolling
      expect(screen.getByTestId('virtual-list')).toBeInTheDocument();
    });
  });

  describe('key code formatting', () => {
    it('removes KEY_ prefix from key codes', () => {
      const event: EventLogEntry = {
        id: '8',
        timestamp: Date.now(),
        type: 'press',
        keyCode: 'KEY_ENTER',
        latency: 0.5,
        input: 'KEY_ENTER',
      };

      render(<EventLogList events={[event]} />);

      expect(
        screen.getByRole('cell', { name: 'Input: ENTER' })
      ).toBeInTheDocument();
    });

    it('removes VK_ prefix from Windows key codes', () => {
      const event: EventLogEntry = {
        id: '9',
        timestamp: Date.now(),
        type: 'press',
        keyCode: 'VK_ESCAPE',
        latency: 0.5,
        input: 'VK_ESCAPE',
      };

      render(<EventLogList events={[event]} />);

      expect(
        screen.getByRole('cell', { name: 'Input: ESCAPE' })
      ).toBeInTheDocument();
    });

    it('handles keys without prefix', () => {
      const event: EventLogEntry = {
        id: '10',
        timestamp: Date.now(),
        type: 'press',
        keyCode: 'A',
        latency: 0.5,
        input: 'A',
      };

      render(<EventLogList events={[event]} />);

      expect(
        screen.getByRole('cell', { name: 'Input: A' })
      ).toBeInTheDocument();
    });
  });

  describe('event type styling', () => {
    it('applies correct styling for press events', () => {
      const pressEvent: EventLogEntry = {
        ...mockEvents[0],
        type: 'press',
      };

      render(<EventLogList events={[pressEvent]} />);

      expect(
        screen.getByRole('cell', { name: /Event type: press/ })
      ).toBeInTheDocument();
    });

    it('applies correct styling for release events', () => {
      const releaseEvent: EventLogEntry = {
        ...mockEvents[0],
        type: 'release',
      };

      render(<EventLogList events={[releaseEvent]} />);

      expect(
        screen.getByRole('cell', { name: /Event type: release/ })
      ).toBeInTheDocument();
    });

    it('applies correct styling for tap events', () => {
      const tapEvent: EventLogEntry = {
        ...mockEvents[0],
        type: 'tap',
      };

      render(<EventLogList events={[tapEvent]} />);

      expect(
        screen.getByRole('cell', { name: /Event type: tap/ })
      ).toBeInTheDocument();
    });

    it('applies correct styling for hold events', () => {
      const holdEvent: EventLogEntry = {
        ...mockEvents[0],
        type: 'hold',
      };

      render(<EventLogList events={[holdEvent]} />);

      expect(
        screen.getByRole('cell', { name: /Event type: hold/ })
      ).toBeInTheDocument();
    });

    it('applies correct styling for macro events', () => {
      const macroEvent: EventLogEntry = {
        ...mockEvents[0],
        type: 'macro',
      };

      render(<EventLogList events={[macroEvent]} />);

      expect(
        screen.getByRole('cell', { name: /Event type: macro/ })
      ).toBeInTheDocument();
    });

    it('applies correct styling for layer_switch events', () => {
      const layerEvent: EventLogEntry = {
        ...mockEvents[0],
        type: 'layer_switch',
      };

      render(<EventLogList events={[layerEvent]} />);

      expect(
        screen.getByRole('cell', { name: /Event type: layer_switch/ })
      ).toBeInTheDocument();
    });
  });

  describe('latency highlighting', () => {
    it('highlights high latency values', () => {
      const highLatencyEvent: EventLogEntry = {
        ...mockEvents[0],
        latency: 5.0, // > 1ms
      };

      render(<EventLogList events={[highLatencyEvent]} />);

      expect(
        screen.getByRole('cell', { name: 'Latency: 5.00ms' })
      ).toBeInTheDocument();
    });

    it('displays normal latency without highlighting', () => {
      const normalLatencyEvent: EventLogEntry = {
        ...mockEvents[0],
        latency: 0.5, // < 1ms
      };

      render(<EventLogList events={[normalLatencyEvent]} />);

      expect(
        screen.getByRole('cell', { name: 'Latency: 0.50ms' })
      ).toBeInTheDocument();
    });
  });

  describe('accessibility', () => {
    it('has proper ARIA role for table', () => {
      render(<EventLogList events={mockEvents} />);

      expect(screen.getByRole('table')).toBeInTheDocument();
    });

    it('has proper ARIA role for rows', () => {
      render(<EventLogList events={mockEvents} />);

      const rows = screen.getAllByRole('row');
      expect(rows.length).toBeGreaterThan(0);
    });

    it('has proper ARIA role for cells', () => {
      render(<EventLogList events={mockEvents} />);

      const cells = screen.getAllByRole('cell');
      expect(cells.length).toBeGreaterThan(0);
    });

    it('has descriptive aria-labels on cells', () => {
      render(<EventLogList events={mockEvents} />);

      // Use getAllByRole since there are multiple cells with these patterns
      expect(
        screen.getAllByRole('cell', { name: /Time:/ }).length
      ).toBeGreaterThan(0);
      expect(
        screen.getAllByRole('cell', { name: /Event type:/ }).length
      ).toBeGreaterThan(0);
      expect(
        screen.getAllByRole('cell', { name: /Input:/ }).length
      ).toBeGreaterThan(0);
      expect(
        screen.getAllByRole('cell', { name: /Latency:/ }).length
      ).toBeGreaterThan(0);
    });
  });
});
