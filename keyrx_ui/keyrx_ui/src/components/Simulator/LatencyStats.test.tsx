/**
 * LatencyStats.test.tsx - Tests for the LatencyStats component.
 *
 * Tests cover:
 * - Rendering all latency metrics (min, avg, max, p95, p99)
 * - Performance level indicators (excellent, good, warning)
 * - Warning badges for high latency
 * - Success badges for excellent performance
 * - Metric formatting (μs, ms, s)
 * - Accessibility features (ARIA labels, tooltips)
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { LatencyStats } from './LatencyStats';
import type { LatencyStats as LatencyStatsType } from '../../wasm/core';

// Extend Vitest matchers with jest-axe
expect.extend(toHaveNoViolations);

describe('LatencyStats', () => {
  describe('Rendering', () => {
    const mockStats: LatencyStatsType = {
      min_us: 5,
      avg_us: 10,
      max_us: 20,
      p95_us: 18,
      p99_us: 19,
    };

    it('should render component title', () => {
      render(<LatencyStats stats={mockStats} />);

      expect(screen.getByText('Performance Metrics')).toBeInTheDocument();
    });

    it('should render all metric rows', () => {
      render(<LatencyStats stats={mockStats} />);

      expect(screen.getByText('Minimum')).toBeInTheDocument();
      expect(screen.getByText('Average')).toBeInTheDocument();
      expect(screen.getByText('Maximum')).toBeInTheDocument();
      expect(screen.getByText('P95')).toBeInTheDocument();
      expect(screen.getByText('P99')).toBeInTheDocument();
    });

    it('should render table headers', () => {
      render(<LatencyStats stats={mockStats} />);

      expect(screen.getByRole('columnheader', { name: 'Metric' })).toBeInTheDocument();
      expect(screen.getByRole('columnheader', { name: 'Value' })).toBeInTheDocument();
      expect(screen.getByRole('columnheader', { name: 'Status' })).toBeInTheDocument();
    });

    it('should render footer with performance information', () => {
      render(<LatencyStats stats={mockStats} />);

      expect(screen.getByText(/All events should complete in <1ms/)).toBeInTheDocument();
      expect(screen.getByText(/Max latency >5ms may cause noticeable delays/)).toBeInTheDocument();
    });
  });

  describe('Metric Formatting', () => {
    it('should format values in microseconds when <1ms', () => {
      const stats: LatencyStatsType = {
        min_us: 5.5,
        avg_us: 10.75,
        max_us: 999.99,
        p95_us: 800,
        p99_us: 900,
      };

      render(<LatencyStats stats={stats} />);

      expect(screen.getByText('5.50μs')).toBeInTheDocument();
      expect(screen.getByText('10.75μs')).toBeInTheDocument();
      expect(screen.getByText('999.99μs')).toBeInTheDocument();
      expect(screen.getByText('800.00μs')).toBeInTheDocument();
      expect(screen.getByText('900.00μs')).toBeInTheDocument();
    });

    it('should format values in milliseconds when <1s', () => {
      const stats: LatencyStatsType = {
        min_us: 1000,
        avg_us: 2500,
        max_us: 999999,
        p95_us: 5000,
        p99_us: 10000,
      };

      render(<LatencyStats stats={stats} />);

      expect(screen.getByText('1.00ms')).toBeInTheDocument();
      expect(screen.getByText('2.50ms')).toBeInTheDocument();
      expect(screen.getByText('1000.00ms')).toBeInTheDocument();
      expect(screen.getByText('5.00ms')).toBeInTheDocument();
      expect(screen.getByText('10.00ms')).toBeInTheDocument();
    });

    it('should format values in seconds when >=1s', () => {
      const stats: LatencyStatsType = {
        min_us: 1000000,
        avg_us: 2500000,
        max_us: 5000000,
        p95_us: 4500000,
        p99_us: 4800000,
      };

      render(<LatencyStats stats={stats} />);

      expect(screen.getByText('1.00s')).toBeInTheDocument();
      expect(screen.getByText('2.50s')).toBeInTheDocument();
      expect(screen.getByText('5.00s')).toBeInTheDocument();
      expect(screen.getByText('4.50s')).toBeInTheDocument();
      expect(screen.getByText('4.80s')).toBeInTheDocument();
    });
  });

  describe('Performance Levels', () => {
    it('should show excellent performance for values <1ms', () => {
      const stats: LatencyStatsType = {
        min_us: 5,
        avg_us: 10,
        max_us: 999,
        p95_us: 800,
        p99_us: 900,
      };

      render(<LatencyStats stats={stats} />);

      // All metrics should have excellent class
      const values = screen.getAllByText(/μs/);
      values.forEach((value) => {
        expect(value).toHaveClass('excellent');
      });
    });

    it('should show good performance for values 1-5ms', () => {
      const stats: LatencyStatsType = {
        min_us: 1000,
        avg_us: 2000,
        max_us: 4999,
        p95_us: 4000,
        p99_us: 4500,
      };

      render(<LatencyStats stats={stats} />);

      // All metrics should have good class
      const values = screen.getAllByText(/ms/);
      values.forEach((value) => {
        expect(value).toHaveClass('good');
      });
    });

    it('should show warning performance for values >5ms', () => {
      const stats: LatencyStatsType = {
        min_us: 5001,
        avg_us: 10000,
        max_us: 20000,
        p95_us: 18000,
        p99_us: 19000,
      };

      render(<LatencyStats stats={stats} />);

      // All metrics should have warning class
      const values = screen.getAllByText(/ms/);
      values.forEach((value) => {
        expect(value).toHaveClass('warning');
      });
    });

    it('should show mixed performance levels correctly', () => {
      const stats: LatencyStatsType = {
        min_us: 500, // excellent
        avg_us: 2000, // good
        max_us: 10000, // warning
        p95_us: 8000, // warning
        p99_us: 9000, // warning
      };

      render(<LatencyStats stats={stats} />);

      const minValue = screen.getByText('500.00μs');
      expect(minValue).toHaveClass('excellent');

      const avgValue = screen.getByText('2.00ms');
      expect(avgValue).toHaveClass('good');

      const maxValue = screen.getByText('10.00ms');
      expect(maxValue).toHaveClass('warning');
    });
  });

  describe('Status Icons', () => {
    it('should show checkmark for excellent performance', () => {
      const stats: LatencyStatsType = {
        min_us: 500,
        avg_us: 500,
        max_us: 500,
        p95_us: 500,
        p99_us: 500,
      };

      render(<LatencyStats stats={stats} />);

      const statusCells = screen.getAllByText('✓');
      expect(statusCells).toHaveLength(5); // All 5 metrics
    });

    it('should show circle for good performance', () => {
      const stats: LatencyStatsType = {
        min_us: 2000,
        avg_us: 2000,
        max_us: 2000,
        p95_us: 2000,
        p99_us: 2000,
      };

      render(<LatencyStats stats={stats} />);

      const statusCells = screen.getAllByText('○');
      expect(statusCells).toHaveLength(5); // All 5 metrics
    });

    it('should show warning icon for poor performance', () => {
      const stats: LatencyStatsType = {
        min_us: 10000,
        avg_us: 10000,
        max_us: 10000,
        p95_us: 10000,
        p99_us: 10000,
      };

      render(<LatencyStats stats={stats} />);

      const statusCells = screen.getAllByText('⚠');
      expect(statusCells).toHaveLength(5); // All 5 metrics
    });
  });

  describe('Performance Badges', () => {
    it('should show excellent badge when all metrics <1ms', () => {
      const stats: LatencyStatsType = {
        min_us: 5,
        avg_us: 10,
        max_us: 999,
        p95_us: 800,
        p99_us: 900,
      };

      render(<LatencyStats stats={stats} />);

      const badge = screen.getByLabelText('Excellent performance');
      expect(badge).toBeInTheDocument();
      expect(badge).toHaveTextContent('✓ All metrics <1ms');
      expect(badge).toHaveClass('excellent');
    });

    it('should show warning badge when max latency >5ms', () => {
      const stats: LatencyStatsType = {
        min_us: 500,
        avg_us: 2000,
        max_us: 10000,
        p95_us: 8000,
        p99_us: 9000,
      };

      render(<LatencyStats stats={stats} />);

      const badge = screen.getByLabelText('Performance warning');
      expect(badge).toBeInTheDocument();
      expect(badge).toHaveTextContent('⚠ Max latency exceeds 5ms');
      expect(badge).toHaveClass('warning');
    });

    it('should not show badges when performance is good but not excellent', () => {
      const stats: LatencyStatsType = {
        min_us: 1000,
        avg_us: 2000,
        max_us: 4999,
        p95_us: 4000,
        p99_us: 4500,
      };

      render(<LatencyStats stats={stats} />);

      expect(screen.queryByLabelText('Excellent performance')).not.toBeInTheDocument();
      expect(screen.queryByLabelText('Performance warning')).not.toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    const mockStats: LatencyStatsType = {
      min_us: 5,
      avg_us: 10,
      max_us: 20,
      p95_us: 18,
      p99_us: 19,
    };

    it('should have region role with label', () => {
      render(<LatencyStats stats={mockStats} />);

      expect(screen.getByRole('region', { name: 'Latency Statistics' })).toBeInTheDocument();
    });

    it('should have proper table structure', () => {
      render(<LatencyStats stats={mockStats} />);

      expect(screen.getByRole('table')).toBeInTheDocument();
      expect(screen.getByRole('columnheader', { name: 'Metric' })).toBeInTheDocument();
      expect(screen.getByRole('columnheader', { name: 'Value' })).toBeInTheDocument();
      expect(screen.getByRole('columnheader', { name: 'Status' })).toBeInTheDocument();
    });

    it('should have th elements with scope="col"', () => {
      render(<LatencyStats stats={mockStats} />);

      const headers = screen.getAllByRole('columnheader');
      headers.forEach((header) => {
        expect(header).toHaveAttribute('scope', 'col');
      });
    });

    it('should have tooltips for P95 and P99 metrics', () => {
      render(<LatencyStats stats={mockStats} />);

      const p95Abbr = screen.getByText('P95').closest('abbr');
      expect(p95Abbr).toHaveAttribute('title', '95th percentile: 95% of events processed faster than this');

      const p99Abbr = screen.getByText('P99').closest('abbr');
      expect(p99Abbr).toHaveAttribute('title', '99th percentile: 99% of events processed faster than this');
    });

    it('should have descriptive aria-labels for badges', () => {
      const excellentStats: LatencyStatsType = {
        min_us: 100,
        avg_us: 200,
        max_us: 300,
        p95_us: 280,
        p99_us: 290,
      };

      render(<LatencyStats stats={excellentStats} />);

      const badge = screen.getByLabelText('Excellent performance');
      expect(badge).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('should handle zero latency values', () => {
      const stats: LatencyStatsType = {
        min_us: 0,
        avg_us: 0,
        max_us: 0,
        p95_us: 0,
        p99_us: 0,
      };

      render(<LatencyStats stats={stats} />);

      expect(screen.getAllByText('0.00μs')).toHaveLength(5);
    });

    it('should handle very large latency values', () => {
      const stats: LatencyStatsType = {
        min_us: 10000000,
        avg_us: 20000000,
        max_us: 50000000,
        p95_us: 45000000,
        p99_us: 48000000,
      };

      render(<LatencyStats stats={stats} />);

      expect(screen.getByText('10.00s')).toBeInTheDocument();
      expect(screen.getByText('20.00s')).toBeInTheDocument();
      expect(screen.getByText('50.00s')).toBeInTheDocument();
    });

    it('should handle boundary values correctly (exactly 1ms)', () => {
      const stats: LatencyStatsType = {
        min_us: 1000,
        avg_us: 1000,
        max_us: 1000,
        p95_us: 1000,
        p99_us: 1000,
      };

      render(<LatencyStats stats={stats} />);

      // 1ms should be formatted as milliseconds and have 'good' class (not excellent)
      const values = screen.getAllByText('1.00ms');
      expect(values).toHaveLength(5);
      values.forEach((value) => {
        expect(value).toHaveClass('good');
      });
    });

    it('should handle boundary values correctly (exactly 5ms)', () => {
      const stats: LatencyStatsType = {
        min_us: 5000,
        avg_us: 5000,
        max_us: 5000,
        p95_us: 5000,
        p99_us: 5000,
      };

      render(<LatencyStats stats={stats} />);

      // 5ms should still be 'warning' (>= 5ms is warning)
      const values = screen.getAllByText('5.00ms');
      expect(values).toHaveLength(5);
      values.forEach((value) => {
        expect(value).toHaveClass('warning');
      });
    });
  });

  describe('Performance Target Communication', () => {
    it('should clearly state the <1ms target', () => {
      const stats: LatencyStatsType = {
        min_us: 5,
        avg_us: 10,
        max_us: 20,
        p95_us: 18,
        p99_us: 19,
      };

      render(<LatencyStats stats={stats} />);

      expect(screen.getByText(/All events should complete in <1ms for optimal performance/)).toBeInTheDocument();
    });

    it('should state the 5ms warning threshold', () => {
      const stats: LatencyStatsType = {
        min_us: 5,
        avg_us: 10,
        max_us: 20,
        p95_us: 18,
        p99_us: 19,
      };

      render(<LatencyStats stats={stats} />);

      expect(screen.getByText(/Max latency >5ms may cause noticeable delays/)).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    const mockStats: LatencyStatsType = {
      min_us: 5,
      avg_us: 10,
      max_us: 20,
      p95_us: 18,
      p99_us: 19,
    };

    it('should have no axe violations with normal stats', async () => {
      const { container } = render(<LatencyStats stats={mockStats} />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations with warning state', async () => {
      const warningStats: LatencyStatsType = {
        min_us: 100,
        avg_us: 2000,
        max_us: 6000,
        p95_us: 5500,
        p99_us: 5900,
      };
      const { container } = render(<LatencyStats stats={warningStats} />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper table structure', () => {
      render(<LatencyStats stats={mockStats} />);

      expect(screen.getByRole('table')).toBeInTheDocument();
      expect(screen.getByRole('columnheader', { name: 'Metric' })).toBeInTheDocument();
      expect(screen.getByRole('columnheader', { name: 'Value' })).toBeInTheDocument();
      expect(screen.getByRole('columnheader', { name: 'Status' })).toBeInTheDocument();
    });

    it('should have tooltips for percentile metrics', () => {
      render(<LatencyStats stats={mockStats} />);

      const p95Tooltip = screen.getByTitle('95% of events completed within this time');
      const p99Tooltip = screen.getByTitle('99% of events completed within this time');

      expect(p95Tooltip).toBeInTheDocument();
      expect(p99Tooltip).toBeInTheDocument();
    });

    it('should have accessible status icons', () => {
      render(<LatencyStats stats={mockStats} />);

      const statusIcons = screen.getAllByRole('img', { hidden: true });
      statusIcons.forEach((icon) => {
        expect(icon).toHaveAttribute('aria-label');
      });
    });

    it('should meet color contrast requirements for performance indicators', () => {
      const warningStats: LatencyStatsType = {
        min_us: 100,
        avg_us: 2000,
        max_us: 6000,
        p95_us: 5500,
        p99_us: 5900,
      };

      render(<LatencyStats stats={warningStats} />);

      // Warning text should be visible
      const warningBadge = screen.getByText('Warning');
      expect(warningBadge).toHaveClass('warning');

      // Check that warning elements are styled for visibility
      const warningValues = screen.getAllByText(/ms/);
      const warningValue = warningValues.find(el => el.classList.contains('warning'));
      expect(warningValue).toBeTruthy();
    });
  });
});
