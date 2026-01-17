import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MetricsStatsCards } from './MetricsStatsCards';
import type { LatencyStats } from '../../types';

// Mock the Card component
vi.mock('../Card', () => ({
  Card: ({
    children,
    'aria-label': ariaLabel,
  }: {
    children: React.ReactNode;
    'aria-label'?: string;
  }) => (
    <div data-testid="card" aria-label={ariaLabel}>
      {children}
    </div>
  ),
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  Activity: () => <div data-testid="activity-icon" />,
  Clock: () => <div data-testid="clock-icon" />,
  Zap: () => <div data-testid="zap-icon" />,
  Cpu: () => <div data-testid="cpu-icon" />,
}));

describe('MetricsStatsCards', () => {
  const mockLatencyStats: LatencyStats = {
    min: 500, // 0.5ms in microseconds
    max: 5000, // 5ms in microseconds
    avg: 1500, // 1.5ms in microseconds
    p50: 1200,
    p95: 3000,
    p99: 4500,
    samples: 100,
    timestamp: '2024-01-01T00:00:00Z',
  };

  describe('rendering', () => {
    it('renders all 4 stat cards', () => {
      render(
        <MetricsStatsCards
          latencyStats={mockLatencyStats}
          eventCount={50}
          connected={true}
        />
      );

      const cards = screen.getAllByTestId('card');
      expect(cards).toHaveLength(4);
    });

    it('renders correct card labels', () => {
      render(
        <MetricsStatsCards
          latencyStats={mockLatencyStats}
          eventCount={50}
          connected={true}
        />
      );

      expect(screen.getByLabelText('Current latency')).toBeInTheDocument();
      expect(screen.getByLabelText('Average latency')).toBeInTheDocument();
      expect(screen.getByLabelText('Minimum latency')).toBeInTheDocument();
      expect(screen.getByLabelText('Maximum latency')).toBeInTheDocument();
    });

    it('renders correct icons for each card', () => {
      render(
        <MetricsStatsCards
          latencyStats={mockLatencyStats}
          eventCount={50}
          connected={true}
        />
      );

      expect(screen.getByTestId('activity-icon')).toBeInTheDocument();
      expect(screen.getByTestId('clock-icon')).toBeInTheDocument();
      expect(screen.getByTestId('zap-icon')).toBeInTheDocument();
      expect(screen.getByTestId('cpu-icon')).toBeInTheDocument();
    });

    it('renders section with correct aria-label', () => {
      render(
        <MetricsStatsCards
          latencyStats={mockLatencyStats}
          eventCount={50}
          connected={true}
        />
      );

      expect(screen.getByLabelText('Latency statistics')).toBeInTheDocument();
    });
  });

  describe('data display', () => {
    it('displays latency values with correct formatting', () => {
      render(
        <MetricsStatsCards
          latencyStats={mockLatencyStats}
          eventCount={50}
          connected={true}
        />
      );

      // Values are in microseconds, should be converted to milliseconds
      // Note: 1.50ms appears twice (current and average are both from avg field)
      const avgValues = screen.getAllByText('1.50ms');
      expect(avgValues.length).toBeGreaterThanOrEqual(2); // current and average
      expect(screen.getByText('0.50ms')).toBeInTheDocument(); // min
      expect(screen.getByText('5.00ms')).toBeInTheDocument(); // max
    });

    it('displays labels for each stat type', () => {
      render(
        <MetricsStatsCards
          latencyStats={mockLatencyStats}
          eventCount={50}
          connected={true}
        />
      );

      expect(screen.getByText('Current')).toBeInTheDocument();
      expect(screen.getByText('Average')).toBeInTheDocument();
      expect(screen.getByText('Min')).toBeInTheDocument();
      expect(screen.getByText('Max')).toBeInTheDocument();
    });

    it('converts microseconds to milliseconds correctly', () => {
      const stats: LatencyStats = {
        min: 1000, // 1ms
        max: 10000, // 10ms
        avg: 5000, // 5ms
        p50: 4000,
        p95: 8000,
        p99: 9500,
        samples: 200,
        timestamp: '2024-01-01T00:00:00Z',
      };

      render(
        <MetricsStatsCards
          latencyStats={stats}
          eventCount={100}
          connected={true}
        />
      );

      const avgValues = screen.getAllByText('5.00ms');
      expect(avgValues.length).toBeGreaterThanOrEqual(2); // current and average
      expect(screen.getByText('1.00ms')).toBeInTheDocument(); // min
      expect(screen.getByText('10.00ms')).toBeInTheDocument(); // max
    });
  });

  describe('null/empty states', () => {
    it('displays zero values when latencyStats is null', () => {
      render(
        <MetricsStatsCards
          latencyStats={null}
          eventCount={0}
          connected={false}
        />
      );

      const zeroValues = screen.getAllByText('0.00ms');
      expect(zeroValues).toHaveLength(4); // All 4 cards should show 0.00ms
    });

    it('handles zero event count', () => {
      render(
        <MetricsStatsCards
          latencyStats={mockLatencyStats}
          eventCount={0}
          connected={true}
        />
      );

      // Should still render cards with data
      const avgValues = screen.getAllByText('1.50ms');
      expect(avgValues.length).toBeGreaterThanOrEqual(2); // current and average
    });

    it('handles disconnected state', () => {
      render(
        <MetricsStatsCards
          latencyStats={mockLatencyStats}
          eventCount={50}
          connected={false}
        />
      );

      // Should still render data even when disconnected
      const avgValues = screen.getAllByText('1.50ms');
      expect(avgValues.length).toBeGreaterThanOrEqual(2); // current and average
    });
  });

  describe('edge cases', () => {
    it('handles very small latency values', () => {
      const stats: LatencyStats = {
        min: 1, // 0.001ms
        max: 10, // 0.01ms
        avg: 5, // 0.005ms
        p50: 4,
        p95: 8,
        p99: 9,
        samples: 10,
        timestamp: '2024-01-01T00:00:00Z',
      };

      render(
        <MetricsStatsCards
          latencyStats={stats}
          eventCount={10}
          connected={true}
        />
      );

      const avgValues = screen.getAllByText('0.01ms');
      expect(avgValues.length).toBeGreaterThanOrEqual(1); // current/avg or max
      expect(screen.getByText('0.00ms')).toBeInTheDocument(); // min (rounded down)
    });

    it('handles very large latency values', () => {
      const stats: LatencyStats = {
        min: 100000, // 100ms
        max: 500000, // 500ms
        avg: 250000, // 250ms
        p50: 200000,
        p95: 400000,
        p99: 480000,
        samples: 50,
        timestamp: '2024-01-01T00:00:00Z',
      };

      render(
        <MetricsStatsCards
          latencyStats={stats}
          eventCount={50}
          connected={true}
        />
      );

      const avgValues = screen.getAllByText('250.00ms');
      expect(avgValues.length).toBeGreaterThanOrEqual(2); // current and average
      expect(screen.getByText('100.00ms')).toBeInTheDocument(); // min
      expect(screen.getByText('500.00ms')).toBeInTheDocument(); // max
    });

    it('handles fractional values correctly', () => {
      const stats: LatencyStats = {
        min: 1234, // 1.234ms
        max: 5678, // 5.678ms
        avg: 3456, // 3.456ms
        p50: 3000,
        p95: 5000,
        p99: 5500,
        samples: 75,
        timestamp: '2024-01-01T00:00:00Z',
      };

      render(
        <MetricsStatsCards
          latencyStats={stats}
          eventCount={75}
          connected={true}
        />
      );

      const avgValues = screen.getAllByText('3.46ms');
      expect(avgValues.length).toBeGreaterThanOrEqual(2); // current and average
      expect(screen.getByText('1.23ms')).toBeInTheDocument(); // min
      expect(screen.getByText('5.68ms')).toBeInTheDocument(); // max
    });
  });

  describe('accessibility', () => {
    it('has proper ARIA labels on cards', () => {
      render(
        <MetricsStatsCards
          latencyStats={mockLatencyStats}
          eventCount={50}
          connected={true}
        />
      );

      expect(screen.getByLabelText('Current latency')).toBeInTheDocument();
      expect(screen.getByLabelText('Average latency')).toBeInTheDocument();
      expect(screen.getByLabelText('Minimum latency')).toBeInTheDocument();
      expect(screen.getByLabelText('Maximum latency')).toBeInTheDocument();
    });

    it('has aria-hidden on icon containers', () => {
      const { container } = render(
        <MetricsStatsCards
          latencyStats={mockLatencyStats}
          eventCount={50}
          connected={true}
        />
      );

      const iconContainers = container.querySelectorAll('[aria-hidden="true"]');
      expect(iconContainers.length).toBeGreaterThan(0);
    });

    it('has descriptive aria-label on values', () => {
      render(
        <MetricsStatsCards
          latencyStats={mockLatencyStats}
          eventCount={50}
          connected={true}
        />
      );

      expect(
        screen.getByLabelText('Current latency: 1.50ms')
      ).toBeInTheDocument();
      expect(
        screen.getByLabelText('Average latency: 1.50ms')
      ).toBeInTheDocument();
    });
  });

  describe('responsive layout', () => {
    it('applies correct grid classes for responsive layout', () => {
      const { container } = render(
        <MetricsStatsCards
          latencyStats={mockLatencyStats}
          eventCount={50}
          connected={true}
        />
      );

      const section = container.querySelector('section');
      expect(section).toHaveClass('grid');
      expect(section).toHaveClass('grid-cols-2');
      expect(section).toHaveClass('md:grid-cols-2');
      expect(section).toHaveClass('lg:grid-cols-4');
    });
  });
});
