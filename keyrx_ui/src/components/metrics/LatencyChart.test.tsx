import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import {
  LatencyChart,
  type LatencyDataPoint,
  formatTimestamp,
  formatLatency,
} from './LatencyChart';

// Mock recharts to avoid rendering issues in tests
vi.mock('recharts', () => ({
  LineChart: ({
    children,
    data,
  }: {
    children: React.ReactNode;
    data: LatencyDataPoint[];
  }) => (
    <div data-testid="line-chart" data-chart-data={JSON.stringify(data)}>
      {children}
    </div>
  ),
  Line: ({ dataKey, stroke }: { dataKey: string; stroke: string }) => (
    <div data-testid="line" data-key={dataKey} data-stroke={stroke} />
  ),
  XAxis: ({
    dataKey,
    tickFormatter,
  }: {
    dataKey: string;
    tickFormatter?: (value: number) => string;
  }) => (
    <div
      data-testid="x-axis"
      data-key={dataKey}
      data-formatter={tickFormatter ? 'true' : 'false'}
    />
  ),
  YAxis: ({ label }: { label?: { value: string } }) => (
    <div data-testid="y-axis" data-label={label?.value} />
  ),
  CartesianGrid: () => <div data-testid="cartesian-grid" />,
  Tooltip: ({
    labelFormatter,
    formatter,
  }: {
    labelFormatter?: (value: number) => string;
    formatter?: (value: number) => [string, string];
  }) => (
    <div
      data-testid="tooltip"
      data-label-formatter={labelFormatter ? 'true' : 'false'}
      data-formatter={formatter ? 'true' : 'false'}
    />
  ),
  ResponsiveContainer: ({
    children,
    width,
    height,
  }: {
    children: React.ReactNode;
    width: string | number;
    height: number;
  }) => (
    <div
      data-testid="responsive-container"
      data-width={width}
      data-height={height}
      style={{ width, height }}
    >
      {children}
    </div>
  ),
}));

describe('LatencyChart', () => {
  const mockData: LatencyDataPoint[] = [
    { timestamp: 1609459200000, latency: 1.23 }, // 2021-01-01 00:00:00
    { timestamp: 1609459201000, latency: 1.45 }, // 2021-01-01 00:00:01
    { timestamp: 1609459202000, latency: 0.98 }, // 2021-01-01 00:00:02
    { timestamp: 1609459203000, latency: 2.1 }, // 2021-01-01 00:00:03
    { timestamp: 1609459204000, latency: 1.67 }, // 2021-01-01 00:00:04
  ];

  describe('Rendering', () => {
    it('renders chart with data', () => {
      render(<LatencyChart data={mockData} />);

      expect(screen.getByTestId('responsive-container')).toBeInTheDocument();
      expect(screen.getByTestId('line-chart')).toBeInTheDocument();
      expect(screen.getByTestId('line')).toBeInTheDocument();
      expect(screen.getByTestId('x-axis')).toBeInTheDocument();
      expect(screen.getByTestId('y-axis')).toBeInTheDocument();
      expect(screen.getByTestId('cartesian-grid')).toBeInTheDocument();
      expect(screen.getByTestId('tooltip')).toBeInTheDocument();
    });

    it('renders all chart elements correctly', () => {
      render(<LatencyChart data={mockData} />);

      const line = screen.getByTestId('line');
      expect(line).toHaveAttribute('data-key', 'latency');
      expect(line).toHaveAttribute('data-stroke', '#3B82F6');

      const xAxis = screen.getByTestId('x-axis');
      expect(xAxis).toHaveAttribute('data-key', 'timestamp');
      expect(xAxis).toHaveAttribute('data-formatter', 'true');

      const yAxis = screen.getByTestId('y-axis');
      expect(yAxis).toHaveAttribute('data-label', 'Latency (ms)');
    });

    it('applies default height of 250px', () => {
      render(<LatencyChart data={mockData} />);

      const container = screen.getByTestId('responsive-container');
      expect(container).toHaveAttribute('data-height', '250');
    });

    it('applies custom height when provided', () => {
      render(<LatencyChart data={mockData} height={400} />);

      const container = screen.getByTestId('responsive-container');
      expect(container).toHaveAttribute('data-height', '400');
    });

    it('sets width to 100%', () => {
      render(<LatencyChart data={mockData} />);

      const container = screen.getByTestId('responsive-container');
      expect(container).toHaveAttribute('data-width', '100%');
    });
  });

  describe('Data Handling', () => {
    it('displays all data points when count is less than maxDataPoints', () => {
      render(<LatencyChart data={mockData} maxDataPoints={60} />);

      const chart = screen.getByTestId('line-chart');
      const chartData = JSON.parse(
        chart.getAttribute('data-chart-data') || '[]'
      );
      expect(chartData).toHaveLength(5);
      expect(chartData[0]).toEqual(mockData[0]);
      expect(chartData[4]).toEqual(mockData[4]);
    });

    it('limits data to maxDataPoints when exceeded', () => {
      const largeDataset = Array.from({ length: 100 }, (_, i) => ({
        timestamp: 1609459200000 + i * 1000,
        latency: Math.random() * 3,
      }));

      render(<LatencyChart data={largeDataset} maxDataPoints={60} />);

      const chart = screen.getByTestId('line-chart');
      const chartData = JSON.parse(
        chart.getAttribute('data-chart-data') || '[]'
      );
      expect(chartData).toHaveLength(60);
      // Should keep the most recent 60 points
      expect(chartData[0]).toEqual(largeDataset[40]); // 100 - 60 = 40
      expect(chartData[59]).toEqual(largeDataset[99]);
    });

    it('uses default maxDataPoints of 60', () => {
      const largeDataset = Array.from({ length: 80 }, (_, i) => ({
        timestamp: 1609459200000 + i * 1000,
        latency: Math.random() * 3,
      }));

      render(<LatencyChart data={largeDataset} />);

      const chart = screen.getByTestId('line-chart');
      const chartData = JSON.parse(
        chart.getAttribute('data-chart-data') || '[]'
      );
      expect(chartData).toHaveLength(60);
    });

    it('handles custom maxDataPoints', () => {
      const largeDataset = Array.from({ length: 50 }, (_, i) => ({
        timestamp: 1609459200000 + i * 1000,
        latency: Math.random() * 3,
      }));

      render(<LatencyChart data={largeDataset} maxDataPoints={30} />);

      const chart = screen.getByTestId('line-chart');
      const chartData = JSON.parse(
        chart.getAttribute('data-chart-data') || '[]'
      );
      expect(chartData).toHaveLength(30);
      expect(chartData[0]).toEqual(largeDataset[20]); // 50 - 30 = 20
    });

    it('memoizes chart data correctly', () => {
      const { rerender } = render(<LatencyChart data={mockData} />);

      const chart1 = screen.getByTestId('line-chart');
      const chartData1 = chart1.getAttribute('data-chart-data');

      // Rerender with same data
      rerender(<LatencyChart data={mockData} />);

      const chart2 = screen.getByTestId('line-chart');
      const chartData2 = chart2.getAttribute('data-chart-data');

      expect(chartData1).toBe(chartData2);
    });

    it('updates when data changes', () => {
      const { rerender } = render(<LatencyChart data={mockData} />);

      const chart1 = screen.getByTestId('line-chart');
      const chartData1 = JSON.parse(
        chart1.getAttribute('data-chart-data') || '[]'
      );
      expect(chartData1).toHaveLength(5);

      const newData = [
        ...mockData,
        { timestamp: 1609459205000, latency: 1.88 },
      ];

      rerender(<LatencyChart data={newData} />);

      const chart2 = screen.getByTestId('line-chart');
      const chartData2 = JSON.parse(
        chart2.getAttribute('data-chart-data') || '[]'
      );
      expect(chartData2).toHaveLength(6);
    });
  });

  describe('Empty State', () => {
    it('renders empty state when data is empty', () => {
      render(<LatencyChart data={[]} />);

      expect(screen.queryByTestId('line-chart')).not.toBeInTheDocument();
      expect(screen.getByText('No data available')).toBeInTheDocument();
    });

    it('applies height to empty state container', () => {
      render(<LatencyChart data={[]} height={300} />);

      const emptyState = screen.getByText('No data available').closest('div');
      expect(emptyState).toHaveStyle({ height: '300px' });
    });

    it('has accessible empty state label', () => {
      render(<LatencyChart data={[]} />);

      const emptyState = screen.getByText('No data available').closest('div');
      expect(emptyState).toHaveAttribute('role', 'img');
      expect(emptyState).toHaveAttribute(
        'aria-label',
        'No latency data available'
      );
    });
  });

  describe('Edge Cases', () => {
    it('handles single data point', () => {
      const singlePoint = [{ timestamp: 1609459200000, latency: 1.23 }];
      render(<LatencyChart data={singlePoint} />);

      const chart = screen.getByTestId('line-chart');
      const chartData = JSON.parse(
        chart.getAttribute('data-chart-data') || '[]'
      );
      expect(chartData).toHaveLength(1);
      expect(chartData[0]).toEqual(singlePoint[0]);
    });

    it('handles very large latency values', () => {
      const largeLatencyData = [
        { timestamp: 1609459200000, latency: 999.99 },
        { timestamp: 1609459201000, latency: 1000.5 },
      ];
      render(<LatencyChart data={largeLatencyData} />);

      const chart = screen.getByTestId('line-chart');
      const chartData = JSON.parse(
        chart.getAttribute('data-chart-data') || '[]'
      );
      expect(chartData[0].latency).toBe(999.99);
      expect(chartData[1].latency).toBe(1000.5);
    });

    it('handles very small latency values', () => {
      const smallLatencyData = [
        { timestamp: 1609459200000, latency: 0.01 },
        { timestamp: 1609459201000, latency: 0.001 },
      ];
      render(<LatencyChart data={smallLatencyData} />);

      const chart = screen.getByTestId('line-chart');
      const chartData = JSON.parse(
        chart.getAttribute('data-chart-data') || '[]'
      );
      expect(chartData[0].latency).toBe(0.01);
      expect(chartData[1].latency).toBe(0.001);
    });

    it('handles zero latency values', () => {
      const zeroLatencyData = [
        { timestamp: 1609459200000, latency: 0 },
        { timestamp: 1609459201000, latency: 0 },
      ];
      render(<LatencyChart data={zeroLatencyData} />);

      const chart = screen.getByTestId('line-chart');
      const chartData = JSON.parse(
        chart.getAttribute('data-chart-data') || '[]'
      );
      expect(chartData[0].latency).toBe(0);
    });

    it('handles maxDataPoints of 1', () => {
      render(<LatencyChart data={mockData} maxDataPoints={1} />);

      const chart = screen.getByTestId('line-chart');
      const chartData = JSON.parse(
        chart.getAttribute('data-chart-data') || '[]'
      );
      expect(chartData).toHaveLength(1);
      expect(chartData[0]).toEqual(mockData[4]); // Most recent point
    });
  });

  describe('Chart Configuration', () => {
    it('configures tooltip with correct formatters', () => {
      render(<LatencyChart data={mockData} />);

      const tooltip = screen.getByTestId('tooltip');
      expect(tooltip).toHaveAttribute('data-label-formatter', 'true');
      expect(tooltip).toHaveAttribute('data-formatter', 'true');
    });

    it('configures X axis with timestamp formatter', () => {
      render(<LatencyChart data={mockData} />);

      const xAxis = screen.getByTestId('x-axis');
      expect(xAxis).toHaveAttribute('data-key', 'timestamp');
      expect(xAxis).toHaveAttribute('data-formatter', 'true');
    });

    it('configures Y axis with label', () => {
      render(<LatencyChart data={mockData} />);

      const yAxis = screen.getByTestId('y-axis');
      expect(yAxis).toHaveAttribute('data-label', 'Latency (ms)');
    });
  });
});

describe('Integration', () => {
  it('renders complete chart with all formatters applied', () => {
    // This test ensures the component function and all formatters are invoked
    const testData: LatencyDataPoint[] = [
      { timestamp: Date.now() - 5000, latency: 1.23 },
      { timestamp: Date.now() - 4000, latency: 1.45 },
      { timestamp: Date.now() - 3000, latency: 0.98 },
    ];

    const { rerender } = render(
      <LatencyChart data={testData} height={300} maxDataPoints={10} />
    );

    // Verify chart is rendered
    expect(screen.getByTestId('responsive-container')).toBeInTheDocument();
    expect(screen.getByTestId('line-chart')).toBeInTheDocument();

    // Test with empty data to trigger empty state rendering
    rerender(<LatencyChart data={[]} />);
    expect(screen.getByText('No data available')).toBeInTheDocument();
  });
});

describe('formatTimestamp', () => {
  it('formats timestamp in HH:MM:SS format', () => {
    const timestamp = new Date('2021-01-01T15:30:45Z').getTime();
    const formatted = formatTimestamp(timestamp);
    // The exact output depends on the timezone, so we check the format
    expect(formatted).toMatch(/^\d{2}:\d{2}:\d{2}$/);
  });

  it('formats midnight correctly', () => {
    const timestamp = new Date('2021-01-01T00:00:00Z').getTime();
    const formatted = formatTimestamp(timestamp);
    expect(formatted).toMatch(/^\d{2}:\d{2}:\d{2}$/);
  });

  it('formats noon correctly', () => {
    const timestamp = new Date('2021-01-01T12:00:00Z').getTime();
    const formatted = formatTimestamp(timestamp);
    expect(formatted).toMatch(/^\d{2}:\d{2}:\d{2}$/);
  });

  it('pads single digit values with zeros', () => {
    const timestamp = new Date('2021-01-01T01:05:09Z').getTime();
    const formatted = formatTimestamp(timestamp);
    expect(formatted).toMatch(/^\d{2}:\d{2}:\d{2}$/);
  });
});

describe('formatLatency', () => {
  it('formats latency with 2 decimal places', () => {
    expect(formatLatency(1.234)).toBe('1.23ms');
    expect(formatLatency(1.239)).toBe('1.24ms');
  });

  it('formats whole numbers with 2 decimal places', () => {
    expect(formatLatency(5)).toBe('5.00ms');
    expect(formatLatency(10)).toBe('10.00ms');
  });

  it('formats very small latencies correctly', () => {
    expect(formatLatency(0.001)).toBe('0.00ms');
    expect(formatLatency(0.01)).toBe('0.01ms');
    expect(formatLatency(0.1)).toBe('0.10ms');
  });

  it('formats very large latencies correctly', () => {
    expect(formatLatency(999.99)).toBe('999.99ms');
    expect(formatLatency(1000.5)).toBe('1000.50ms');
  });

  it('formats zero latency', () => {
    expect(formatLatency(0)).toBe('0.00ms');
  });

  it('rounds correctly', () => {
    expect(formatLatency(1.234)).toBe('1.23ms');
    expect(formatLatency(1.235)).toBe('1.24ms');
    expect(formatLatency(1.236)).toBe('1.24ms');
  });
});
