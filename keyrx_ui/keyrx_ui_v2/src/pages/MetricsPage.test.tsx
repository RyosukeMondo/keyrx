import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MetricsPage } from './MetricsPage';

// Mock recharts to avoid rendering issues in tests
vi.mock('recharts', () => ({
  LineChart: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="line-chart">{children}</div>
  ),
  Line: () => <div data-testid="line" />,
  XAxis: () => <div data-testid="x-axis" />,
  YAxis: () => <div data-testid="y-axis" />,
  CartesianGrid: () => <div data-testid="cartesian-grid" />,
  Tooltip: () => <div data-testid="tooltip" />,
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="responsive-container">{children}</div>
  ),
}));

// Mock react-window
vi.mock('react-window', () => ({
  FixedSizeList: ({
    children,
    itemCount,
  }: {
    children: (props: { index: number; style: React.CSSProperties }) => React.ReactNode;
    itemCount: number;
  }) => (
    <div data-testid="virtual-list">
      {Array.from({ length: Math.min(itemCount, 10) }, (_, i) => (
        <div key={i}>{children({ index: i, style: {} })}</div>
      ))}
    </div>
  ),
}));

describe('MetricsPage', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  it('renders the page header', () => {
    render(<MetricsPage />);

    expect(screen.getByText('Performance Metrics')).toBeInTheDocument();
    expect(
      screen.getByText('Real-time monitoring and debugging tools')
    ).toBeInTheDocument();
  });

  it('renders latency statistics cards', () => {
    render(<MetricsPage />);

    expect(screen.getByText('Current')).toBeInTheDocument();
    expect(screen.getByText('Average')).toBeInTheDocument();
    expect(screen.getByText('Min')).toBeInTheDocument();
    expect(screen.getByText('Max')).toBeInTheDocument();
  });

  it('displays latency values in milliseconds', () => {
    render(<MetricsPage />);

    // Should display latency values with "ms" suffix
    const latencyValues = screen.getAllByText(/\d+\.\d+ms/);
    expect(latencyValues.length).toBeGreaterThan(0);
  });

  it('renders latency chart', () => {
    render(<MetricsPage />);

    expect(screen.getByText('Latency Over Time')).toBeInTheDocument();
    expect(screen.getByText('Last 60 seconds')).toBeInTheDocument();
    expect(screen.getByTestId('line-chart')).toBeInTheDocument();
  });

  it('initializes with latency data', () => {
    render(<MetricsPage />);

    // Should display latency values on mount
    const latencyValues = screen.getAllByText(/\d+\.\d+ms/);
    expect(latencyValues.length).toBeGreaterThan(0);
  });

  it('renders event log with headers', () => {
    render(<MetricsPage />);

    expect(screen.getByText('Event Log')).toBeInTheDocument();
    expect(screen.getByText('Timestamp')).toBeInTheDocument();
    expect(screen.getByText('Type')).toBeInTheDocument();
    expect(screen.getByText('Key Code')).toBeInTheDocument();
    expect(screen.getByText('Action')).toBeInTheDocument();
    expect(screen.getByText('Latency')).toBeInTheDocument();
  });

  it('renders virtual scrolling list for event log', () => {
    render(<MetricsPage />);

    expect(screen.getByTestId('virtual-list')).toBeInTheDocument();
  });

  it('displays event log entries', () => {
    render(<MetricsPage />);

    // Should render some event entries (at least the first 10 due to virtual scrolling mock)
    const eventTypes = ['press', 'release', 'tap', 'hold', 'macro', 'layer_switch'];
    const renderedTypes = eventTypes.filter((type) =>
      screen.queryAllByText(type).length > 0
    );

    expect(renderedTypes.length).toBeGreaterThan(0);
  });

  it('renders state inspector', () => {
    render(<MetricsPage />);

    expect(screen.getByText('State Inspector')).toBeInTheDocument();
    expect(screen.getByText('Current daemon internal state')).toBeInTheDocument();
  });

  it('displays state fields', () => {
    render(<MetricsPage />);

    expect(screen.getByText('Active Layer')).toBeInTheDocument();
    expect(screen.getByText('Tap/Hold Timers')).toBeInTheDocument();
    expect(screen.getByText('Active Modifiers')).toBeInTheDocument();
    expect(screen.getByText('Active Locks')).toBeInTheDocument();
    expect(screen.getByText('Queued Events')).toBeInTheDocument();
  });

  it('shows default state values', () => {
    render(<MetricsPage />);

    expect(screen.getByText('Base')).toBeInTheDocument(); // Active Layer
    expect(screen.getByText('0 active')).toBeInTheDocument(); // Tap/Hold Timers
    expect(screen.getAllByText('None')[0]).toBeInTheDocument(); // Active Modifiers
    expect(screen.getAllByText('None')[1]).toBeInTheDocument(); // Active Locks
    expect(screen.getByText('0')).toBeInTheDocument(); // Queued Events
  });

  it('displays multiple events in the log', () => {
    render(<MetricsPage />);

    // Get virtual list
    const virtualList = screen.getByTestId('virtual-list');

    // Should have multiple event entries
    expect(virtualList.children.length).toBeGreaterThan(0);
  });

  it('formats timestamps correctly', () => {
    render(<MetricsPage />);

    // Should display timestamps in HH:MM:SS.mmm format
    // Look for time patterns in the virtual list
    const virtualList = screen.getByTestId('virtual-list');
    expect(virtualList.textContent).toMatch(/\d{2}:\d{2}:\d{2}\.\d{3}/);
  });

  it('renders with responsive layout', () => {
    const { container } = render(<MetricsPage />);

    // Check for grid classes for responsive layout
    const grids = container.querySelectorAll('.grid');
    expect(grids.length).toBeGreaterThan(0);
  });

  it('displays event type colors', () => {
    const { container } = render(<MetricsPage />);

    // Event types should have color classes
    const colorClasses = [
      'text-green-400',
      'text-red-400',
      'text-blue-400',
      'text-yellow-400',
      'text-purple-400',
      'text-cyan-400',
    ];

    const hasColorClass = colorClasses.some((colorClass) =>
      container.querySelector(`.${colorClass}`)
    );

    expect(hasColorClass).toBe(true);
  });
});
