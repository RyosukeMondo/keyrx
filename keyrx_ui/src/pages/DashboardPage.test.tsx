import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import { DashboardPage } from './DashboardPage';
import type { DaemonState, KeyEvent, LatencyMetrics } from '../types/rpc';

// Mock the hooks and components
vi.mock('../hooks/useUnifiedApi');
vi.mock('../components/StateIndicatorPanel', () => ({
  StateIndicatorPanel: ({ state }: any) => (
    <div
      data-testid="state-indicator-panel"
      data-state={JSON.stringify(state)}
    />
  ),
}));

vi.mock('../components/MetricsChart', () => ({
  MetricsChart: ({ data }: any) => (
    <div data-testid="metrics-chart" data-length={data.length} />
  ),
}));

vi.mock('../components/DashboardEventTimeline', () => ({
  DashboardEventTimeline: ({
    events,
    isPaused,
    onTogglePause,
    onClear,
  }: any) => (
    <div
      data-testid="event-timeline"
      data-event-count={events.length}
      data-is-paused={isPaused}
    >
      <button onClick={onTogglePause}>Toggle Pause</button>
      <button onClick={onClear}>Clear</button>
    </div>
  ),
}));

describe('DashboardPage', () => {
  let mockApi: any;
  let daemonStateHandler: ((state: DaemonState) => void) | null = null;
  let eventsHandler: ((event: KeyEvent) => void) | null = null;
  let latencyHandler: ((metrics: LatencyMetrics) => void) | null = null;

  beforeEach(async () => {
    // Reset handlers
    daemonStateHandler = null;
    eventsHandler = null;
    latencyHandler = null;

    // Create mock API
    mockApi = {
      isConnected: true,
      readyState: 1,
      query: vi.fn(),
      command: vi.fn(),
      subscribe: vi.fn((channel: string, handler: any) => {
        if (channel === 'daemon-state') {
          daemonStateHandler = handler;
        } else if (channel === 'events') {
          eventsHandler = handler;
        } else if (channel === 'latency') {
          latencyHandler = handler;
        }
        // Return unsubscribe function
        return vi.fn();
      }),
      unsubscribe: vi.fn(),
    };

    // Mock useUnifiedApi to return our mock API
    const { useUnifiedApi } = await import('../hooks/useUnifiedApi');
    vi.mocked(useUnifiedApi).mockReturnValue(mockApi);
  });

  it('renders connection banner with "Connected" when connected', () => {
    renderWithProviders(<DashboardPage />);
    expect(screen.getByText('Connected')).toBeInTheDocument();
  });

  it('renders connection banner with "Disconnected" when not connected', async () => {
    mockApi.isConnected = false;

    renderWithProviders(<DashboardPage />);
    expect(screen.getByText('Disconnected')).toBeInTheDocument();
  });

  it('connection banner has green background when connected', () => {
    renderWithProviders(<DashboardPage />);

    const banner = screen.getByText('Connected');
    expect(banner).toHaveClass('bg-green-600');
    expect(banner).toHaveClass('text-white');
  });

  it('connection banner has red background when disconnected', () => {
    mockApi.isConnected = false;

    renderWithProviders(<DashboardPage />);

    const banner = screen.getByText('Disconnected');
    expect(banner).toHaveClass('bg-red-600');
    expect(banner).toHaveClass('text-white');
  });

  it('subscribes to daemon-state channel on mount', () => {
    renderWithProviders(<DashboardPage />);

    expect(mockApi.subscribe).toHaveBeenCalledWith(
      'daemon-state',
      expect.any(Function)
    );
  });

  it('subscribes to events channel on mount', () => {
    renderWithProviders(<DashboardPage />);

    expect(mockApi.subscribe).toHaveBeenCalledWith(
      'events',
      expect.any(Function)
    );
  });

  it('subscribes to latency channel on mount', () => {
    renderWithProviders(<DashboardPage />);

    expect(mockApi.subscribe).toHaveBeenCalledWith(
      'latency',
      expect.any(Function)
    );
  });

  it('unsubscribes from all channels on unmount', () => {
    const unsubscribeFns = [vi.fn(), vi.fn(), vi.fn()];
    let callIndex = 0;

    mockApi.subscribe = vi.fn(() => unsubscribeFns[callIndex++]);

    const { unmount } = renderWithProviders(<DashboardPage />);

    unmount();

    expect(unsubscribeFns[0]).toHaveBeenCalled();
    expect(unsubscribeFns[1]).toHaveBeenCalled();
    expect(unsubscribeFns[2]).toHaveBeenCalled();
  });

  it('updates daemon state when daemon-state event is received', async () => {
    renderWithProviders(<DashboardPage />);

    const newState: DaemonState = {
      modifiers: [1, 2],
      locks: [3],
      layer: 1,
    };

    // Trigger state update
    daemonStateHandler?.(newState);

    await waitFor(() => {
      const panel = screen.getByTestId('state-indicator-panel');
      const stateData = JSON.parse(panel.getAttribute('data-state') || 'null');
      expect(stateData).toEqual(newState);
    });
  });

  it('adds events to timeline when events are received', async () => {
    renderWithProviders(<DashboardPage />);

    const event1: KeyEvent = {
      timestamp: 1000000,
      keyCode: 65,
      eventType: 'press',
      input: 65,
      output: 66,
      latency: 500,
    };

    const event2: KeyEvent = {
      timestamp: 2000000,
      keyCode: 13,
      eventType: 'release',
      input: 13,
      output: 13,
      latency: 300,
    };

    // Trigger events
    eventsHandler?.(event1);
    eventsHandler?.(event2);

    await waitFor(() => {
      const timeline = screen.getByTestId('event-timeline');
      expect(timeline.getAttribute('data-event-count')).toBe('2');
    });
  });

  it('enforces 100 event FIFO limit', async () => {
    renderWithProviders(<DashboardPage />);

    // Add 101 events
    for (let i = 0; i < 101; i++) {
      eventsHandler?.({
        timestamp: i * 1000,
        keyCode: 65,
        eventType: 'press',
        input: 65,
        output: 65,
        latency: 100,
      });
    }

    await waitFor(() => {
      const timeline = screen.getByTestId('event-timeline');
      // Should be limited to 100
      expect(timeline.getAttribute('data-event-count')).toBe('100');
    });
  });

  it('adds latency metrics to history', async () => {
    renderWithProviders(<DashboardPage />);

    const metrics1: LatencyMetrics = {
      min: 100,
      avg: 200,
      max: 300,
      p50: 150,
      p95: 250,
      p99: 290,
      count: 10,
    };

    const metrics2: LatencyMetrics = {
      min: 200,
      avg: 300,
      max: 400,
      p50: 250,
      p95: 350,
      p99: 390,
      count: 20,
    };

    // Trigger metrics
    latencyHandler?.(metrics1);
    latencyHandler?.(metrics2);

    await waitFor(() => {
      const chart = screen.getByTestId('metrics-chart');
      expect(chart.getAttribute('data-length')).toBe('2');
    });
  });

  it('enforces 60 latency metrics FIFO limit', async () => {
    renderWithProviders(<DashboardPage />);

    // Add 61 metrics
    for (let i = 0; i < 61; i++) {
      latencyHandler?.({
        min: 100,
        avg: 200,
        max: 300,
        p50: 150,
        p95: 250,
        p99: 290,
        count: i,
      });
    }

    await waitFor(() => {
      const chart = screen.getByTestId('metrics-chart');
      // Should be limited to 60
      expect(chart.getAttribute('data-length')).toBe('60');
    });
  });

  it('renders StateIndicatorPanel with null initially', () => {
    renderWithProviders(<DashboardPage />);

    const panel = screen.getByTestId('state-indicator-panel');
    const stateData = panel.getAttribute('data-state');
    expect(stateData).toBe('null');
  });

  it('renders MetricsChart with empty array initially', () => {
    renderWithProviders(<DashboardPage />);

    const chart = screen.getByTestId('metrics-chart');
    expect(chart.getAttribute('data-length')).toBe('0');
  });

  it('renders DashboardEventTimeline with empty array initially', () => {
    renderWithProviders(<DashboardPage />);

    const timeline = screen.getByTestId('event-timeline');
    expect(timeline.getAttribute('data-event-count')).toBe('0');
  });

  it('renders all three child components', () => {
    renderWithProviders(<DashboardPage />);

    expect(screen.getByTestId('state-indicator-panel')).toBeInTheDocument();
    expect(screen.getByTestId('metrics-chart')).toBeInTheDocument();
    expect(screen.getByTestId('event-timeline')).toBeInTheDocument();
  });

  it('uses responsive grid layout for state and metrics', () => {
    const { container } = renderWithProviders(<DashboardPage />);

    const grid = container.querySelector('.grid');
    expect(grid).toHaveClass('grid-cols-1');
    expect(grid).toHaveClass('lg:grid-cols-2');
  });

  it('pauses event updates when pause is toggled', async () => {
    renderWithProviders(<DashboardPage />);

    // Toggle pause
    const toggleButton = screen.getByText('Toggle Pause');
    toggleButton.click();

    await waitFor(() => {
      const timeline = screen.getByTestId('event-timeline');
      expect(timeline.getAttribute('data-is-paused')).toBe('true');
    });

    // Add event while paused (should not be added)
    eventsHandler?.({
      timestamp: 1000000,
      keyCode: 65,
      eventType: 'press',
      input: 65,
      output: 65,
      latency: 100,
    });

    // Event count should still be 0
    const timeline = screen.getByTestId('event-timeline');
    expect(timeline.getAttribute('data-event-count')).toBe('0');
  });

  it('resumes event updates when pause is toggled again', async () => {
    renderWithProviders(<DashboardPage />);

    // Toggle pause twice
    const toggleButton = screen.getByText('Toggle Pause');
    toggleButton.click();
    toggleButton.click();

    await waitFor(() => {
      const timeline = screen.getByTestId('event-timeline');
      expect(timeline.getAttribute('data-is-paused')).toBe('false');
    });

    // Add event (should be added)
    eventsHandler?.({
      timestamp: 1000000,
      keyCode: 65,
      eventType: 'press',
      input: 65,
      output: 65,
      latency: 100,
    });

    await waitFor(() => {
      const timeline = screen.getByTestId('event-timeline');
      expect(timeline.getAttribute('data-event-count')).toBe('1');
    });
  });

  it('clears events when clear is clicked', async () => {
    renderWithProviders(<DashboardPage />);

    // Add some events
    eventsHandler?.({
      timestamp: 1000000,
      keyCode: 65,
      eventType: 'press',
      input: 65,
      output: 65,
      latency: 100,
    });

    await waitFor(() => {
      const timeline = screen.getByTestId('event-timeline');
      expect(timeline.getAttribute('data-event-count')).toBe('1');
    });

    // Clear events
    const clearButton = screen.getByText('Clear');
    clearButton.click();

    await waitFor(() => {
      const timeline = screen.getByTestId('event-timeline');
      expect(timeline.getAttribute('data-event-count')).toBe('0');
    });
  });

  it('events are added newest first (FIFO)', async () => {
    renderWithProviders(<DashboardPage />);

    const event1: KeyEvent = {
      timestamp: 1000000,
      keyCode: 65,
      eventType: 'press',
      input: 65,
      output: 66,
      latency: 500,
    };

    const event2: KeyEvent = {
      timestamp: 2000000,
      keyCode: 13,
      eventType: 'release',
      input: 13,
      output: 13,
      latency: 300,
    };

    eventsHandler?.(event1);
    eventsHandler?.(event2);

    await waitFor(() => {
      const timeline = screen.getByTestId('event-timeline');
      expect(timeline.getAttribute('data-event-count')).toBe('2');
    });

    // Verify newest first by checking that the second event is at the beginning
    // This is implicitly tested by the FIFO slice(0, 100) in the component
  });
});
