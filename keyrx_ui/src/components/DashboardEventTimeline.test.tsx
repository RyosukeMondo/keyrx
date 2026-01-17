import { describe, it, expect, vi } from 'vitest';
import { screen, fireEvent } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import { DashboardEventTimeline } from './DashboardEventTimeline';
import type { KeyEvent } from '../types/rpc';

// Mock react-window
interface FixedSizeListProps {
  children: (props: {
    index: number;
    style: Record<string, unknown>;
  }) => React.ReactNode;
  itemCount: number;
  height: number;
  itemSize: number;
  className?: string;
}

vi.mock('react-window', () => ({
  FixedSizeList: ({
    children,
    itemCount,
    height,
    itemSize,
    className,
  }: FixedSizeListProps) => (
    <div
      data-testid="virtualized-list"
      data-item-count={itemCount}
      data-height={height}
      data-item-size={itemSize}
      className={className}
    >
      {/* Render first few items for testing */}
      {Array.from({ length: Math.min(itemCount, 3) }, (_, index) =>
        children({ index, style: {} })
      )}
    </div>
  ),
}));

// Mock utility functions
vi.mock('../utils/keyCodeMapping', () => ({
  formatKeyCode: (code: number) => {
    const keyMap: Record<number, string> = {
      65: 'A',
      13: 'Enter',
      32: 'Space',
      16: 'Shift',
    };
    return keyMap[code] || `Key${code}`;
  },
}));

vi.mock('../utils/timeFormatting', () => ({
  formatTimestampRelative: (timestamp: number) => {
    const now = Date.now();
    const diff = now - timestamp;
    if (diff < 1000) return 'just now';
    if (diff < 60000) return `${Math.floor(diff / 1000)}s ago`;
    return `${Math.floor(diff / 60000)}m ago`;
  },
}));

describe('DashboardEventTimeline', () => {
  const mockEvents: KeyEvent[] = [
    {
      timestamp: 1000000,
      keyCode: 65,
      eventType: 'press',
      input: 65,
      output: 66,
      latency: 500,
    },
    {
      timestamp: 2000000,
      keyCode: 13,
      eventType: 'release',
      input: 13,
      output: 13,
      latency: 300,
    },
    {
      timestamp: 3000000,
      keyCode: 32,
      eventType: 'press',
      input: 32,
      output: 32,
      latency: 400,
    },
  ];

  it('renders title', () => {
    renderWithProviders(
      <DashboardEventTimeline
        events={[]}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    expect(screen.getByText('Event Timeline')).toBeInTheDocument();
  });

  it('renders pause button with correct text when not paused', () => {
    renderWithProviders(
      <DashboardEventTimeline
        events={[]}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    expect(screen.getByText('Pause')).toBeInTheDocument();
  });

  it('renders resume button with correct text when paused', () => {
    renderWithProviders(
      <DashboardEventTimeline
        events={[]}
        isPaused={true}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    expect(screen.getByText('Resume')).toBeInTheDocument();
  });

  it('calls onTogglePause when pause button is clicked', () => {
    const handleTogglePause = vi.fn();

    renderWithProviders(
      <DashboardEventTimeline
        events={[]}
        isPaused={false}
        onTogglePause={handleTogglePause}
        onClear={vi.fn()}
      />
    );

    fireEvent.click(screen.getByText('Pause'));
    expect(handleTogglePause).toHaveBeenCalledTimes(1);
  });

  it('calls onClear when clear button is clicked', () => {
    const handleClear = vi.fn();

    renderWithProviders(
      <DashboardEventTimeline
        events={mockEvents}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={handleClear}
      />
    );

    fireEvent.click(screen.getByText('Clear'));
    expect(handleClear).toHaveBeenCalledTimes(1);
  });

  it('shows empty state when events array is empty', () => {
    renderWithProviders(
      <DashboardEventTimeline
        events={[]}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    expect(
      screen.getByText('No events yet. Start typing to see events appear.')
    ).toBeInTheDocument();
  });

  it('renders virtualized list when events are present', () => {
    renderWithProviders(
      <DashboardEventTimeline
        events={mockEvents}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    const list = screen.getByTestId('virtualized-list');
    expect(list).toBeInTheDocument();
    expect(list).toHaveAttribute('data-item-count', '3');
    expect(list).toHaveAttribute('data-height', '400');
    expect(list).toHaveAttribute('data-item-size', '50');
  });

  it('displays key codes using formatKeyCode', () => {
    renderWithProviders(
      <DashboardEventTimeline
        events={mockEvents}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    expect(screen.getByText('A')).toBeInTheDocument();
    expect(screen.getByText('Enter')).toBeInTheDocument();
    expect(screen.getByText('Space')).toBeInTheDocument();
  });

  it('displays event types with correct styling', () => {
    renderWithProviders(
      <DashboardEventTimeline
        events={mockEvents}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    const pressElements = screen.getAllByText('press');
    const releaseElements = screen.getAllByText('release');

    expect(pressElements.length).toBeGreaterThan(0);
    expect(releaseElements.length).toBeGreaterThan(0);

    // Verify green background for press
    expect(pressElements[0]).toHaveClass('bg-green-900');
    // Verify red background for release
    expect(releaseElements[0]).toHaveClass('bg-red-900');
  });

  it('shows tooltip on hover', () => {
    const { container } = renderWithProviders(
      <DashboardEventTimeline
        events={mockEvents}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    // Find first event row by looking for the relative positioning div
    const eventRow = container.querySelector(
      '.relative.flex.items-center'
    ) as HTMLElement;
    expect(eventRow).toBeTruthy();

    // Trigger hover
    fireEvent.mouseEnter(eventRow);

    // Tooltip should appear with timestamp
    expect(screen.getByText(/1000000μs/)).toBeInTheDocument();
  });

  it('hides tooltip on mouse leave', () => {
    const { container } = renderWithProviders(
      <DashboardEventTimeline
        events={mockEvents}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    const eventRow = container.querySelector(
      '.relative.flex.items-center'
    ) as HTMLElement;

    // Show tooltip
    fireEvent.mouseEnter(eventRow);
    expect(screen.getByText(/1000000μs/)).toBeInTheDocument();

    // Hide tooltip
    fireEvent.mouseLeave(eventRow);
    expect(screen.queryByText(/1000000μs/)).not.toBeInTheDocument();
  });

  it('tooltip displays full event details', () => {
    const { container } = renderWithProviders(
      <DashboardEventTimeline
        events={mockEvents}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    const eventRow = container.querySelector(
      '.relative.flex.items-center'
    ) as HTMLElement;
    fireEvent.mouseEnter(eventRow);

    // Check for all tooltip fields
    expect(screen.getByText('Timestamp:')).toBeInTheDocument();
    expect(screen.getByText('1000000μs')).toBeInTheDocument();
    expect(screen.getByText('Input:')).toBeInTheDocument();
    expect(screen.getByText('65')).toBeInTheDocument();
    expect(screen.getByText('Output:')).toBeInTheDocument();
    expect(screen.getByText('66')).toBeInTheDocument();
    expect(screen.getByText('Latency:')).toBeInTheDocument();
    expect(screen.getByText('500μs')).toBeInTheDocument();
  });

  it('pause button has minimum 44px tap target', () => {
    renderWithProviders(
      <DashboardEventTimeline
        events={[]}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    const pauseButton = screen.getByText('Pause');
    expect(pauseButton).toHaveClass('min-h-[44px]');
  });

  it('clear button has minimum 44px tap target', () => {
    renderWithProviders(
      <DashboardEventTimeline
        events={[]}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    const clearButton = screen.getByText('Clear');
    expect(clearButton).toHaveClass('min-h-[44px]');
  });

  it('has proper ARIA labels on buttons', () => {
    renderWithProviders(
      <DashboardEventTimeline
        events={[]}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    expect(screen.getByLabelText('Pause event updates')).toBeInTheDocument();
    expect(screen.getByLabelText('Clear all events')).toBeInTheDocument();
  });

  it('ARIA label changes based on paused state', () => {
    const { rerender } = renderWithProviders(
      <DashboardEventTimeline
        events={[]}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    expect(screen.getByLabelText('Pause event updates')).toBeInTheDocument();

    rerender(
      <DashboardEventTimeline
        events={[]}
        isPaused={true}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    expect(screen.getByLabelText('Resume event updates')).toBeInTheDocument();
  });

  it('handles large number of events efficiently with virtualization', () => {
    const manyEvents = Array.from({ length: 1000 }, (_, i) => ({
      timestamp: i * 1000,
      keyCode: 65 + (i % 26),
      eventType: i % 2 === 0 ? 'press' : 'release',
      input: 65 + (i % 26),
      output: 65 + (i % 26),
      latency: 100 + i,
    })) as KeyEvent[];

    renderWithProviders(
      <DashboardEventTimeline
        events={manyEvents}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    const list = screen.getByTestId('virtualized-list');
    expect(list).toHaveAttribute('data-item-count', '1000');

    // Only first 3 items should be rendered due to our mock
    const renderedEvents = screen.getAllByText(/press|release/);
    expect(renderedEvents.length).toBeLessThan(10); // Virtualization limits rendered items
  });

  it('displays input -> output mapping', () => {
    renderWithProviders(
      <DashboardEventTimeline
        events={mockEvents}
        isPaused={false}
        onTogglePause={vi.fn()}
        onClear={vi.fn()}
      />
    );

    // First event: 65 (A) -> 66 (B)
    expect(screen.getByText(/A → Key66/)).toBeInTheDocument();
  });
});
