import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QuickStatsCard } from './QuickStatsCard';

describe('QuickStatsCard', () => {
  const mockStats = {
    latencyAvg: 2.3,
    eventsToday: 1247,
    uptimeSeconds: 19380, // 5h 23m
  };

  it('renders loading state', () => {
    render(<QuickStatsCard loading={true} />);
    const loadingElements = screen.getAllByRole('generic');
    const hasAnimatePulse = loadingElements.some((el) =>
      el.classList.contains('animate-pulse')
    );
    expect(hasAnimatePulse).toBe(true);
  });

  it('renders empty state when no stats', () => {
    render(<QuickStatsCard />);
    expect(screen.getByText('Quick Stats')).toBeInTheDocument();
    expect(
      screen.getByText(/Statistics unavailable/)
    ).toBeInTheDocument();
  });

  it('renders stats heading', () => {
    render(<QuickStatsCard stats={mockStats} />);
    expect(screen.getByText('Quick Stats')).toBeInTheDocument();
  });

  it('renders latency with correct precision', () => {
    render(<QuickStatsCard stats={mockStats} />);
    expect(screen.getByText(/2.3ms avg/)).toBeInTheDocument();
  });

  it('formats event count with thousands separator', () => {
    render(<QuickStatsCard stats={mockStats} />);
    expect(screen.getByText(/1,247 today/)).toBeInTheDocument();
  });

  it('formats large event numbers correctly', () => {
    const largeStats = {
      ...mockStats,
      eventsToday: 123456,
    };
    render(<QuickStatsCard stats={largeStats} />);
    expect(screen.getByText(/123,456 today/)).toBeInTheDocument();
  });

  it('formats uptime correctly in hours and minutes', () => {
    render(<QuickStatsCard stats={mockStats} />);
    expect(screen.getByText(/5h 23m/)).toBeInTheDocument();
  });

  it('handles uptime less than 1 hour', () => {
    const shortUptime = {
      ...mockStats,
      uptimeSeconds: 1800, // 30 minutes
    };
    render(<QuickStatsCard stats={shortUptime} />);
    expect(screen.getByText(/0h 30m/)).toBeInTheDocument();
  });

  it('handles uptime with exact hours (no minutes)', () => {
    const exactHours = {
      ...mockStats,
      uptimeSeconds: 7200, // 2 hours exactly
    };
    render(<QuickStatsCard stats={exactHours} />);
    expect(screen.getByText(/2h 0m/)).toBeInTheDocument();
  });

  it('renders latency label with correct styling', () => {
    render(<QuickStatsCard stats={mockStats} />);
    const latencyLabel = screen.getByText('Latency:');
    expect(latencyLabel).toHaveClass('font-medium', 'text-slate-100');
  });

  it('renders events label with correct styling', () => {
    render(<QuickStatsCard stats={mockStats} />);
    const eventsLabel = screen.getByText('Events:');
    expect(eventsLabel).toHaveClass('font-medium', 'text-slate-100');
  });

  it('renders uptime label with correct styling', () => {
    render(<QuickStatsCard stats={mockStats} />);
    const uptimeLabel = screen.getByText('Uptime:');
    expect(uptimeLabel).toHaveClass('font-medium', 'text-slate-100');
  });

  it('uses bullet separators between stats', () => {
    render(<QuickStatsCard stats={mockStats} />);
    const bullets = screen.getAllByText('•');
    expect(bullets).toHaveLength(2);
  });

  it('applies custom className', () => {
    const { container } = render(
      <QuickStatsCard stats={mockStats} className="custom-class" />
    );
    expect(container.querySelector('.custom-class')).toBeInTheDocument();
  });

  it('handles zero latency', () => {
    const zeroLatency = {
      ...mockStats,
      latencyAvg: 0,
    };
    render(<QuickStatsCard stats={zeroLatency} />);
    expect(screen.getByText(/0.0ms avg/)).toBeInTheDocument();
  });

  it('handles high latency values', () => {
    const highLatency = {
      ...mockStats,
      latencyAvg: 15.7,
    };
    render(<QuickStatsCard stats={highLatency} />);
    expect(screen.getByText(/15.7ms avg/)).toBeInTheDocument();
  });
});
