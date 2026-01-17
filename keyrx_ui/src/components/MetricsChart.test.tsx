import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import { MetricsChart } from './MetricsChart';
import type { LatencyMetrics } from '../types/rpc';

// Mock Recharts components
interface ResponsiveContainerProps {
  children: React.ReactNode;
}

interface LineChartProps {
  children: React.ReactNode;
  data: unknown;
}

interface LineProps {
  dataKey: string;
  stroke: string;
  name: string;
}

interface ReferenceLineProps {
  y: number;
  label?: { value: string };
}

vi.mock('recharts', () => ({
  ResponsiveContainer: ({ children }: ResponsiveContainerProps) => (
    <div data-testid="responsive-container">{children}</div>
  ),
  LineChart: ({ children, data }: LineChartProps) => (
    <div data-testid="line-chart" data-chart-data={JSON.stringify(data)}>
      {children}
    </div>
  ),
  Line: ({ dataKey, stroke, name }: LineProps) => (
    <div
      data-testid={`line-${dataKey}`}
      data-stroke={stroke}
      data-name={name}
    />
  ),
  XAxis: () => <div data-testid="x-axis" />,
  YAxis: () => <div data-testid="y-axis" />,
  CartesianGrid: () => <div data-testid="cartesian-grid" />,
  Tooltip: () => <div data-testid="tooltip" />,
  Legend: () => <div data-testid="legend" />,
  ReferenceLine: ({ y, label }: ReferenceLineProps) => (
    <div data-testid="reference-line" data-y={y} data-label={label?.value} />
  ),
}));

describe('MetricsChart', () => {
  it('renders empty state when data is empty', () => {
    renderWithProviders(<MetricsChart data={[]} />);
    expect(
      screen.getByText('No latency data available yet...')
    ).toBeInTheDocument();
  });

  it('renders chart title', () => {
    const data: LatencyMetrics[] = [
      {
        min: 1000,
        avg: 2000,
        max: 3000,
        p50: 1500,
        p95: 2500,
        p99: 2900,
        count: 10,
      },
    ];

    renderWithProviders(<MetricsChart data={data} />);
    expect(screen.getByText('Latency Metrics')).toBeInTheDocument();
  });

  it('renders chart components when data is provided', () => {
    const data: LatencyMetrics[] = [
      {
        min: 1000,
        avg: 2000,
        max: 3000,
        p50: 1500,
        p95: 2500,
        p99: 2900,
        count: 10,
      },
    ];

    renderWithProviders(<MetricsChart data={data} />);

    expect(screen.getByTestId('responsive-container')).toBeInTheDocument();
    expect(screen.getByTestId('line-chart')).toBeInTheDocument();
    expect(screen.getByTestId('x-axis')).toBeInTheDocument();
    expect(screen.getByTestId('y-axis')).toBeInTheDocument();
    expect(screen.getByTestId('cartesian-grid')).toBeInTheDocument();
    expect(screen.getByTestId('tooltip')).toBeInTheDocument();
    expect(screen.getByTestId('legend')).toBeInTheDocument();
  });

  it('renders three lines for avg, p95, and p99', () => {
    const data: LatencyMetrics[] = [
      {
        min: 1000,
        avg: 2000,
        max: 3000,
        p50: 1500,
        p95: 2500,
        p99: 2900,
        count: 10,
      },
    ];

    renderWithProviders(<MetricsChart data={data} />);

    expect(screen.getByTestId('line-avg')).toBeInTheDocument();
    expect(screen.getByTestId('line-p95')).toBeInTheDocument();
    expect(screen.getByTestId('line-p99')).toBeInTheDocument();
  });

  it('renders reference line at 5ms', () => {
    const data: LatencyMetrics[] = [
      {
        min: 1000,
        avg: 2000,
        max: 3000,
        p50: 1500,
        p95: 2500,
        p99: 2900,
        count: 10,
      },
    ];

    renderWithProviders(<MetricsChart data={data} />);

    const referenceLine = screen.getByTestId('reference-line');
    expect(referenceLine).toBeInTheDocument();
    expect(referenceLine).toHaveAttribute('data-y', '5');
    expect(referenceLine).toHaveAttribute('data-label', 'Target (5ms)');
  });

  it('converts microseconds to milliseconds', () => {
    const data: LatencyMetrics[] = [
      {
        min: 1000,
        avg: 2000,
        max: 3000,
        p50: 1500,
        p95: 2500,
        p99: 2900,
        count: 10,
      },
      {
        min: 2000,
        avg: 4000,
        max: 5000,
        p50: 3000,
        p95: 4500,
        p99: 4800,
        count: 20,
      },
    ];

    renderWithProviders(<MetricsChart data={data} />);

    const lineChart = screen.getByTestId('line-chart');
    const chartData = JSON.parse(
      lineChart.getAttribute('data-chart-data') || '[]'
    );

    // Verify conversion from microseconds to milliseconds
    expect(chartData[0].avg).toBe(2); // 2000 μs -> 2 ms
    expect(chartData[0].p95).toBe(2.5); // 2500 μs -> 2.5 ms
    expect(chartData[0].p99).toBe(2.9); // 2900 μs -> 2.9 ms

    expect(chartData[1].avg).toBe(4); // 4000 μs -> 4 ms
    expect(chartData[1].p95).toBe(4.5); // 4500 μs -> 4.5 ms
    expect(chartData[1].p99).toBe(4.8); // 4800 μs -> 4.8 ms
  });

  it('includes index in transformed data', () => {
    const data: LatencyMetrics[] = [
      {
        min: 1000,
        avg: 2000,
        max: 3000,
        p50: 1500,
        p95: 2500,
        p99: 2900,
        count: 10,
      },
      {
        min: 2000,
        avg: 4000,
        max: 5000,
        p50: 3000,
        p95: 4500,
        p99: 4800,
        count: 20,
      },
      {
        min: 3000,
        avg: 5000,
        max: 6000,
        p50: 4000,
        p95: 5500,
        p99: 5900,
        count: 30,
      },
    ];

    renderWithProviders(<MetricsChart data={data} />);

    const lineChart = screen.getByTestId('line-chart');
    const chartData = JSON.parse(
      lineChart.getAttribute('data-chart-data') || '[]'
    );

    expect(chartData[0].index).toBe(0);
    expect(chartData[1].index).toBe(1);
    expect(chartData[2].index).toBe(2);
  });

  it('uses correct colors for lines', () => {
    const data: LatencyMetrics[] = [
      {
        min: 1000,
        avg: 2000,
        max: 3000,
        p50: 1500,
        p95: 2500,
        p99: 2900,
        count: 10,
      },
    ];

    renderWithProviders(<MetricsChart data={data} />);

    const avgLine = screen.getByTestId('line-avg');
    const p95Line = screen.getByTestId('line-p95');
    const p99Line = screen.getByTestId('line-p99');

    expect(avgLine).toHaveAttribute('data-stroke', '#3b82f6'); // blue-500
    expect(p95Line).toHaveAttribute('data-stroke', '#f97316'); // orange-500
    expect(p99Line).toHaveAttribute('data-stroke', '#ef4444'); // red-500
  });

  it('uses correct names for lines', () => {
    const data: LatencyMetrics[] = [
      {
        min: 1000,
        avg: 2000,
        max: 3000,
        p50: 1500,
        p95: 2500,
        p99: 2900,
        count: 10,
      },
    ];

    renderWithProviders(<MetricsChart data={data} />);

    const avgLine = screen.getByTestId('line-avg');
    const p95Line = screen.getByTestId('line-p95');
    const p99Line = screen.getByTestId('line-p99');

    expect(avgLine).toHaveAttribute('data-name', 'Average');
    expect(p95Line).toHaveAttribute('data-name', 'P95');
    expect(p99Line).toHaveAttribute('data-name', 'P99');
  });

  it('handles single data point', () => {
    const data: LatencyMetrics[] = [
      {
        min: 1000,
        avg: 2000,
        max: 3000,
        p50: 1500,
        p95: 2500,
        p99: 2900,
        count: 10,
      },
    ];

    renderWithProviders(<MetricsChart data={data} />);

    const lineChart = screen.getByTestId('line-chart');
    const chartData = JSON.parse(
      lineChart.getAttribute('data-chart-data') || '[]'
    );

    expect(chartData.length).toBe(1);
    expect(chartData[0]).toEqual({
      index: 0,
      avg: 2,
      p95: 2.5,
      p99: 2.9,
    });
  });

  it('handles many data points', () => {
    const data: LatencyMetrics[] = Array.from({ length: 60 }, (_, i) => ({
      min: 1000 + i * 100,
      avg: 2000 + i * 100,
      max: 3000 + i * 100,
      p50: 1500 + i * 100,
      p95: 2500 + i * 100,
      p99: 2900 + i * 100,
      count: 10 + i,
    }));

    renderWithProviders(<MetricsChart data={data} />);

    const lineChart = screen.getByTestId('line-chart');
    const chartData = JSON.parse(
      lineChart.getAttribute('data-chart-data') || '[]'
    );

    expect(chartData.length).toBe(60);
    expect(chartData[0].index).toBe(0);
    expect(chartData[59].index).toBe(59);
  });

  it('uses responsive container', () => {
    const data: LatencyMetrics[] = [
      {
        min: 1000,
        avg: 2000,
        max: 3000,
        p50: 1500,
        p95: 2500,
        p99: 2900,
        count: 10,
      },
    ];

    renderWithProviders(<MetricsChart data={data} />);

    expect(screen.getByTestId('responsive-container')).toBeInTheDocument();
  });
});
