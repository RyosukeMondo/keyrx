/**
 * SimulationResults.test.tsx - Tests for the SimulationResults component.
 *
 * Tests cover:
 * - Empty state rendering
 * - Timeline visualization with events
 * - Event type colors (modifiers, locks, layers)
 * - Input/output comparison highlighting
 * - Hover tooltips with full state details
 * - Legend display
 * - Timestamp formatting
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { axe, toHaveNoViolations } from 'jest-axe';
import SimulationResults from './SimulationResults';
import type { SimulationResult } from '../../wasm/core';

// Extend Vitest matchers with jest-axe
expect.extend(toHaveNoViolations);

describe('SimulationResults', () => {
  describe('Empty States', () => {
    it('should show empty state when result is null', () => {
      render(<SimulationResults result={null} />);

      expect(screen.getByText('No simulation results yet. Run a scenario or custom sequence to see results.')).toBeInTheDocument();
    });

    it('should show empty state when timeline is empty', () => {
      const emptyResult: SimulationResult = {
        timeline: [],
        latency_stats: {
          min_us: 0,
          avg_us: 0,
          max_us: 0,
          p95_us: 0,
          p99_us: 0,
        },
      };

      render(<SimulationResults result={emptyResult} />);

      expect(screen.getByText('Simulation completed with no events.')).toBeInTheDocument();
    });
  });

  describe('Timeline Rendering', () => {
    const mockResult: SimulationResult = {
      timeline: [
        {
          timestamp_us: 0,
          input: { keycode: 'VK_A', event_type: 'press', timestamp_us: 0 },
          outputs: [{ keycode: 'VK_A', event_type: 'press', timestamp_us: 0 }],
          state: {
            active_modifiers: [],
            active_locks: [],
            active_layer: null,
            raw_state: new Uint8Array(32),
          },
          latency_us: 10,
        },
        {
          timestamp_us: 100000,
          input: { keycode: 'VK_A', event_type: 'release', timestamp_us: 100000 },
          outputs: [{ keycode: 'VK_A', event_type: 'release', timestamp_us: 100000 }],
          state: {
            active_modifiers: [],
            active_locks: [],
            active_layer: null,
            raw_state: new Uint8Array(32),
          },
          latency_us: 12,
        },
      ],
      latency_stats: {
        min_us: 10,
        avg_us: 11,
        max_us: 12,
        p95_us: 11.9,
        p99_us: 11.98,
      },
    };

    it('should render timeline with events', () => {
      render(<SimulationResults result={mockResult} />);

      expect(screen.getByText('Simulation Results')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /Event at 0μs/ })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /Event at 100.0ms/ })).toBeInTheDocument();
    });

    it('should render time labels', () => {
      render(<SimulationResults result={mockResult} />);

      // Should show min, mid, and max time labels
      expect(screen.getByText('0μs')).toBeInTheDocument();
      expect(screen.getByText('100.0ms')).toBeInTheDocument();
    });

    it('should render legend', () => {
      render(<SimulationResults result={mockResult} />);

      expect(screen.getByText('Legend:')).toBeInTheDocument();
      expect(screen.getByText('Regular Event')).toBeInTheDocument();
      expect(screen.getByText('Modifier Change')).toBeInTheDocument();
      expect(screen.getByText('Lock Change')).toBeInTheDocument();
      expect(screen.getByText('Layer Change')).toBeInTheDocument();
      expect(screen.getByText('Input/Output Mismatch')).toBeInTheDocument();
    });
  });

  describe('Event Type Colors', () => {
    it('should highlight events with modifier changes', () => {
      const resultWithModifiers: SimulationResult = {
        timeline: [
          {
            timestamp_us: 0,
            input: { keycode: 'VK_LShift', event_type: 'press', timestamp_us: 0 },
            outputs: [{ keycode: 'VK_LShift', event_type: 'press', timestamp_us: 0 }],
            state: {
              active_modifiers: ['MD_00'],
              active_locks: [],
              active_layer: null,
              raw_state: new Uint8Array(32),
            },
            latency_us: 10,
          },
        ],
        latency_stats: {
          min_us: 10,
          avg_us: 10,
          max_us: 10,
          p95_us: 10,
          p99_us: 10,
        },
      };

      render(<SimulationResults result={resultWithModifiers} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      expect(event).toHaveClass('modifier-change');
    });

    it('should highlight events with lock changes', () => {
      const resultWithLocks: SimulationResult = {
        timeline: [
          {
            timestamp_us: 0,
            input: { keycode: 'VK_CapsLock', event_type: 'press', timestamp_us: 0 },
            outputs: [{ keycode: 'VK_CapsLock', event_type: 'press', timestamp_us: 0 }],
            state: {
              active_modifiers: [],
              active_locks: ['LK_00'],
              active_layer: null,
              raw_state: new Uint8Array(32),
            },
            latency_us: 10,
          },
        ],
        latency_stats: {
          min_us: 10,
          avg_us: 10,
          max_us: 10,
          p95_us: 10,
          p99_us: 10,
        },
      };

      render(<SimulationResults result={resultWithLocks} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      expect(event).toHaveClass('lock-change');
    });

    it('should highlight events with layer changes (highest priority)', () => {
      const resultWithLayer: SimulationResult = {
        timeline: [
          {
            timestamp_us: 0,
            input: { keycode: 'VK_Fn', event_type: 'press', timestamp_us: 0 },
            outputs: [{ keycode: 'VK_Fn', event_type: 'press', timestamp_us: 0 }],
            state: {
              active_modifiers: ['MD_00'],
              active_locks: ['LK_00'],
              active_layer: 'LAYER_FN',
              raw_state: new Uint8Array(32),
            },
            latency_us: 10,
          },
        ],
        latency_stats: {
          min_us: 10,
          avg_us: 10,
          max_us: 10,
          p95_us: 10,
          p99_us: 10,
        },
      };

      render(<SimulationResults result={resultWithLayer} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      expect(event).toHaveClass('layer-change');
    });
  });

  describe('Input/Output Comparison', () => {
    it('should highlight events with input/output mismatch', () => {
      const resultWithMismatch: SimulationResult = {
        timeline: [
          {
            timestamp_us: 0,
            input: { keycode: 'VK_A', event_type: 'press', timestamp_us: 0 },
            outputs: [{ keycode: 'VK_B', event_type: 'press', timestamp_us: 0 }],
            state: {
              active_modifiers: [],
              active_locks: [],
              active_layer: null,
              raw_state: new Uint8Array(32),
            },
            latency_us: 10,
          },
        ],
        latency_stats: {
          min_us: 10,
          avg_us: 10,
          max_us: 10,
          p95_us: 10,
          p99_us: 10,
        },
      };

      render(<SimulationResults result={resultWithMismatch} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      expect(event).toHaveClass('has-diff');
    });

    it('should not highlight when input matches single output', () => {
      const resultWithMatch: SimulationResult = {
        timeline: [
          {
            timestamp_us: 0,
            input: { keycode: 'VK_A', event_type: 'press', timestamp_us: 0 },
            outputs: [{ keycode: 'VK_A', event_type: 'press', timestamp_us: 0 }],
            state: {
              active_modifiers: [],
              active_locks: [],
              active_layer: null,
              raw_state: new Uint8Array(32),
            },
            latency_us: 10,
          },
        ],
        latency_stats: {
          min_us: 10,
          avg_us: 10,
          max_us: 10,
          p95_us: 10,
          p99_us: 10,
        },
      };

      render(<SimulationResults result={resultWithMatch} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      expect(event).not.toHaveClass('has-diff');
    });

    it('should highlight when multiple outputs', () => {
      const resultWithMultipleOutputs: SimulationResult = {
        timeline: [
          {
            timestamp_us: 0,
            input: { keycode: 'VK_A', event_type: 'press', timestamp_us: 0 },
            outputs: [
              { keycode: 'VK_A', event_type: 'press', timestamp_us: 0 },
              { keycode: 'VK_B', event_type: 'press', timestamp_us: 0 },
            ],
            state: {
              active_modifiers: [],
              active_locks: [],
              active_layer: null,
              raw_state: new Uint8Array(32),
            },
            latency_us: 10,
          },
        ],
        latency_stats: {
          min_us: 10,
          avg_us: 10,
          max_us: 10,
          p95_us: 10,
          p99_us: 10,
        },
      };

      render(<SimulationResults result={resultWithMultipleOutputs} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      expect(event).toHaveClass('has-diff');
    });
  });

  describe('Hover Tooltips', () => {
    const mockResult: SimulationResult = {
      timeline: [
        {
          timestamp_us: 0,
          input: { keycode: 'VK_A', event_type: 'press', timestamp_us: 0 },
          outputs: [{ keycode: 'VK_B', event_type: 'release', timestamp_us: 0 }],
          state: {
            active_modifiers: ['MD_00', 'MD_01'],
            active_locks: ['LK_00'],
            active_layer: 'LAYER_FN',
            raw_state: new Uint8Array(32),
          },
          latency_us: 15,
        },
      ],
      latency_stats: {
        min_us: 15,
        avg_us: 15,
        max_us: 15,
        p95_us: 15,
        p99_us: 15,
      },
    };

    it('should show tooltip on hover', async () => {
      const user = userEvent.setup();
      render(<SimulationResults result={mockResult} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      await user.hover(event);

      expect(screen.getByText('Timestamp:')).toBeInTheDocument();
      expect(screen.getByText('0μs')).toBeInTheDocument();
      expect(screen.getByText('Input:')).toBeInTheDocument();
      expect(screen.getByText('VK_A↓')).toBeInTheDocument();
      expect(screen.getByText('Outputs:')).toBeInTheDocument();
      expect(screen.getByText('VK_B↑')).toBeInTheDocument();
      expect(screen.getByText('Latency:')).toBeInTheDocument();
      expect(screen.getByText('15.00μs')).toBeInTheDocument();
    });

    it('should show modifier state in tooltip', async () => {
      const user = userEvent.setup();
      render(<SimulationResults result={mockResult} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      await user.hover(event);

      expect(screen.getByText('Modifiers:')).toBeInTheDocument();
      expect(screen.getByText('[MD_00, MD_01]')).toBeInTheDocument();
    });

    it('should show lock state in tooltip', async () => {
      const user = userEvent.setup();
      render(<SimulationResults result={mockResult} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      await user.hover(event);

      expect(screen.getByText('Locks:')).toBeInTheDocument();
      expect(screen.getByText('[LK_00]')).toBeInTheDocument();
    });

    it('should show layer state in tooltip', async () => {
      const user = userEvent.setup();
      render(<SimulationResults result={mockResult} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      await user.hover(event);

      expect(screen.getByText('Layer:')).toBeInTheDocument();
      expect(screen.getByText('LAYER_FN')).toBeInTheDocument();
    });

    it('should hide tooltip on mouse leave', async () => {
      const user = userEvent.setup();
      render(<SimulationResults result={mockResult} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      await user.hover(event);

      expect(screen.getByText('Timestamp:')).toBeInTheDocument();

      await user.unhover(event);

      expect(screen.queryByText('Timestamp:')).not.toBeInTheDocument();
    });

    it('should show "None" for events with no outputs', async () => {
      const user = userEvent.setup();
      const resultNoOutputs: SimulationResult = {
        timeline: [
          {
            timestamp_us: 0,
            input: { keycode: 'VK_A', event_type: 'press', timestamp_us: 0 },
            outputs: [],
            state: {
              active_modifiers: [],
              active_locks: [],
              active_layer: null,
              raw_state: new Uint8Array(32),
            },
            latency_us: 10,
          },
        ],
        latency_stats: {
          min_us: 10,
          avg_us: 10,
          max_us: 10,
          p95_us: 10,
          p99_us: 10,
        },
      };

      render(<SimulationResults result={resultNoOutputs} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      await user.hover(event);

      expect(screen.getByText('Outputs:')).toBeInTheDocument();
      expect(screen.getByText('None')).toBeInTheDocument();
    });
  });

  describe('Timestamp Formatting', () => {
    it('should format microseconds for values <1ms', () => {
      const result: SimulationResult = {
        timeline: [
          {
            timestamp_us: 500,
            input: { keycode: 'VK_A', event_type: 'press', timestamp_us: 500 },
            outputs: [],
            state: {
              active_modifiers: [],
              active_locks: [],
              active_layer: null,
              raw_state: new Uint8Array(32),
            },
            latency_us: 10,
          },
        ],
        latency_stats: {
          min_us: 10,
          avg_us: 10,
          max_us: 10,
          p95_us: 10,
          p99_us: 10,
        },
      };

      render(<SimulationResults result={result} />);

      expect(screen.getByRole('button', { name: /Event at 500μs/ })).toBeInTheDocument();
    });

    it('should format milliseconds for values <1s', () => {
      const result: SimulationResult = {
        timeline: [
          {
            timestamp_us: 1500,
            input: { keycode: 'VK_A', event_type: 'press', timestamp_us: 1500 },
            outputs: [],
            state: {
              active_modifiers: [],
              active_locks: [],
              active_layer: null,
              raw_state: new Uint8Array(32),
            },
            latency_us: 10,
          },
        ],
        latency_stats: {
          min_us: 10,
          avg_us: 10,
          max_us: 10,
          p95_us: 10,
          p99_us: 10,
        },
      };

      render(<SimulationResults result={result} />);

      expect(screen.getByRole('button', { name: /Event at 1.5ms/ })).toBeInTheDocument();
    });

    it('should format seconds for values >=1s', () => {
      const result: SimulationResult = {
        timeline: [
          {
            timestamp_us: 2500000,
            input: { keycode: 'VK_A', event_type: 'press', timestamp_us: 2500000 },
            outputs: [],
            state: {
              active_modifiers: [],
              active_locks: [],
              active_layer: null,
              raw_state: new Uint8Array(32),
            },
            latency_us: 10,
          },
        ],
        latency_stats: {
          min_us: 10,
          avg_us: 10,
          max_us: 10,
          p95_us: 10,
          p99_us: 10,
        },
      };

      render(<SimulationResults result={result} />);

      expect(screen.getByRole('button', { name: /Event at 2.50s/ })).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    const mockResult: SimulationResult = {
      timeline: [
        {
          timestamp_us: 0,
          input: { keycode: 'VK_A', event_type: 'press', timestamp_us: 0 },
          outputs: [{ keycode: 'VK_A', event_type: 'press', timestamp_us: 0 }],
          state: {
            active_modifiers: [],
            active_locks: [],
            active_layer: null,
            raw_state: new Uint8Array(32),
          },
          latency_us: 10,
        },
      ],
      latency_stats: {
        min_us: 10,
        avg_us: 10,
        max_us: 10,
        p95_us: 10,
        p99_us: 10,
      },
    };

    it('should have no axe violations in empty state', async () => {
      const { container } = render(<SimulationResults result={null} />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations with timeline data', async () => {
      const { container } = render(<SimulationResults result={mockResult} />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper ARIA label for timeline', () => {
      render(<SimulationResults result={mockResult} />);

      expect(screen.getByLabelText('Event Timeline')).toBeInTheDocument();
    });

    it('should have accessible event markers', () => {
      render(<SimulationResults result={mockResult} />);

      const event = screen.getByRole('button', { name: /Event at 0μs: VK_A↓/ });
      expect(event).toHaveAttribute('tabIndex', '0');
      expect(event).toHaveAttribute('aria-label');
    });

    it('should be keyboard navigable', () => {
      render(<SimulationResults result={mockResult} />);

      const event = screen.getByRole('button', { name: /Event at 0μs/ });
      expect(event).toHaveAttribute('tabIndex', '0');
    });

    it('should have accessible legend', () => {
      render(<SimulationResults result={mockResult} />);

      const legend = screen.getByText('Legend:').closest('div');
      expect(legend).toBeInTheDocument();
    });
  });
});
