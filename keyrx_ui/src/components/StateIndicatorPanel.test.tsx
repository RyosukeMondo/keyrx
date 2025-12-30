/**
 * Unit tests for StateIndicatorPanel component
 *
 * Tests state visualization, modifier/lock indicators, and accessibility
 */

import { render, screen } from '@testing-library/react';
import { describe, it, expect, beforeEach } from 'vitest';
import { act } from 'react';
import { StateIndicatorPanel } from './StateIndicatorPanel';
import { useDashboardStore } from '../store/dashboardStore';
import type { DaemonState } from '../store/dashboardStore';

describe('StateIndicatorPanel', () => {
  beforeEach(() => {
    // Reset store to initial state before each test
    useDashboardStore.getState().reset();
  });

  describe('Initial State Rendering', () => {
    it('should render with empty state (no modifiers, no locks, base layer)', () => {
      render(<StateIndicatorPanel />);

      // Verify section titles
      expect(screen.getByText('Active Modifiers')).toBeInTheDocument();
      expect(screen.getByText('Active Locks')).toBeInTheDocument();
      expect(screen.getByText('Current Layer')).toBeInTheDocument();

      // Verify empty states
      expect(screen.getByText('No active modifiers')).toBeInTheDocument();
      expect(screen.getByText('No active locks')).toBeInTheDocument();

      // Verify base layer is displayed
      expect(screen.getByText('base')).toBeInTheDocument();
    });

    it('should have proper accessibility structure', () => {
      render(<StateIndicatorPanel />);

      // Verify role="group" for badge containers
      const modifiersGroup = screen.getByRole('group', { name: 'Active modifiers' });
      expect(modifiersGroup).toBeInTheDocument();

      const locksGroup = screen.getByRole('group', { name: 'Active locks' });
      expect(locksGroup).toBeInTheDocument();

      const layerGroup = screen.getByRole('group', { name: 'Current layer' });
      expect(layerGroup).toBeInTheDocument();
    });

    it('should have aria-live regions for empty states', () => {
      render(<StateIndicatorPanel />);

      const emptyStates = screen.getAllByText(/No active/i);
      emptyStates.forEach((element) => {
        expect(element).toHaveAttribute('aria-live', 'polite');
      });
    });
  });

  describe('Active Modifiers Display', () => {
    it('should display single active modifier', () => {
      const state: DaemonState = {
        modifiers: ['MD_00'],
        locks: [],
        layer: 'base',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      expect(screen.getByText('MD_00')).toBeInTheDocument();
      expect(screen.queryByText('No active modifiers')).not.toBeInTheDocument();
    });

    it('should display multiple active modifiers', () => {
      const state: DaemonState = {
        modifiers: ['MD_00', 'MD_01', 'MD_02'],
        locks: [],
        layer: 'base',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      expect(screen.getByText('MD_00')).toBeInTheDocument();
      expect(screen.getByText('MD_01')).toBeInTheDocument();
      expect(screen.getByText('MD_02')).toBeInTheDocument();
    });

    it('should apply correct CSS classes to modifier badges', () => {
      const state: DaemonState = {
        modifiers: ['MD_00'],
        locks: [],
        layer: 'base',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      const badge = screen.getByText('MD_00');
      expect(badge).toHaveClass('state-badge');
      expect(badge).toHaveClass('state-badge-modifier');
    });

    it('should have proper accessibility labels for modifiers', () => {
      const state: DaemonState = {
        modifiers: ['MD_00'],
        locks: [],
        layer: 'base',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      const badge = screen.getByText('MD_00');
      expect(badge).toHaveAttribute('role', 'status');
      expect(badge).toHaveAttribute('aria-label', 'Modifier MD_00 active');
    });
  });

  describe('Active Locks Display', () => {
    it('should display single active lock', () => {
      const state: DaemonState = {
        modifiers: [],
        locks: ['LK_00'],
        layer: 'base',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      expect(screen.getByText('LK_00')).toBeInTheDocument();
      expect(screen.queryByText('No active locks')).not.toBeInTheDocument();
    });

    it('should display multiple active locks', () => {
      const state: DaemonState = {
        modifiers: [],
        locks: ['LK_00', 'LK_01', 'LK_02'],
        layer: 'base',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      expect(screen.getByText('LK_00')).toBeInTheDocument();
      expect(screen.getByText('LK_01')).toBeInTheDocument();
      expect(screen.getByText('LK_02')).toBeInTheDocument();
    });

    it('should apply correct CSS classes to lock badges', () => {
      const state: DaemonState = {
        modifiers: [],
        locks: ['LK_00'],
        layer: 'base',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      const badge = screen.getByText('LK_00');
      expect(badge).toHaveClass('state-badge');
      expect(badge).toHaveClass('state-badge-lock');
    });

    it('should have proper accessibility labels for locks', () => {
      const state: DaemonState = {
        modifiers: [],
        locks: ['LK_00'],
        layer: 'base',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      const badge = screen.getByText('LK_00');
      expect(badge).toHaveAttribute('role', 'status');
      expect(badge).toHaveAttribute('aria-label', 'Lock LK_00 active');
    });
  });

  describe('Current Layer Display', () => {
    it('should display base layer by default', () => {
      render(<StateIndicatorPanel />);

      const layerBadge = screen.getByText('base');
      expect(layerBadge).toBeInTheDocument();
      expect(layerBadge).toHaveClass('state-badge');
      expect(layerBadge).toHaveClass('state-badge-layer');
    });

    it('should display custom layer name', () => {
      const state: DaemonState = {
        modifiers: [],
        locks: [],
        layer: 'gaming',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      expect(screen.getByText('gaming')).toBeInTheDocument();
      expect(screen.queryByText('base')).not.toBeInTheDocument();
    });

    it('should have proper accessibility label for layer', () => {
      const state: DaemonState = {
        modifiers: [],
        locks: [],
        layer: 'gaming',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      const badge = screen.getByText('gaming');
      expect(badge).toHaveAttribute('role', 'status');
      expect(badge).toHaveAttribute('aria-label', 'Layer gaming active');
    });
  });

  describe('State Updates', () => {
    it('should update when modifiers change', () => {
      const { rerender } = render(<StateIndicatorPanel />);

      // Initial state: no modifiers
      expect(screen.getByText('No active modifiers')).toBeInTheDocument();

      // Update state: add modifiers
      const state: DaemonState = {
        modifiers: ['MD_00', 'MD_01'],
        locks: [],
        layer: 'base',
      };
      act(() => {
        useDashboardStore.getState().updateState(state);
      });
      rerender(<StateIndicatorPanel />);

      expect(screen.queryByText('No active modifiers')).not.toBeInTheDocument();
      expect(screen.getByText('MD_00')).toBeInTheDocument();
      expect(screen.getByText('MD_01')).toBeInTheDocument();
    });

    it('should update when locks change', () => {
      const { rerender } = render(<StateIndicatorPanel />);

      // Initial state: no locks
      expect(screen.getByText('No active locks')).toBeInTheDocument();

      // Update state: add locks
      const state: DaemonState = {
        modifiers: [],
        locks: ['LK_00'],
        layer: 'base',
      };
      act(() => {
        useDashboardStore.getState().updateState(state);
      });
      rerender(<StateIndicatorPanel />);

      expect(screen.queryByText('No active locks')).not.toBeInTheDocument();
      expect(screen.getByText('LK_00')).toBeInTheDocument();
    });

    it('should update when layer changes', () => {
      const { rerender } = render(<StateIndicatorPanel />);

      // Initial state: base layer
      expect(screen.getByText('base')).toBeInTheDocument();

      // Update state: change layer
      const state: DaemonState = {
        modifiers: [],
        locks: [],
        layer: 'navigation',
      };
      act(() => {
        useDashboardStore.getState().updateState(state);
      });
      rerender(<StateIndicatorPanel />);

      expect(screen.queryByText('base')).not.toBeInTheDocument();
      expect(screen.getByText('navigation')).toBeInTheDocument();
    });

    it('should handle rapid state updates', () => {
      const { rerender } = render(<StateIndicatorPanel />);

      // Simulate rapid state changes
      const states: DaemonState[] = [
        { modifiers: ['MD_00'], locks: [], layer: 'base' },
        { modifiers: ['MD_00', 'MD_01'], locks: [], layer: 'base' },
        { modifiers: ['MD_00', 'MD_01'], locks: ['LK_00'], layer: 'base' },
        { modifiers: [], locks: ['LK_00'], layer: 'gaming' },
        { modifiers: [], locks: [], layer: 'base' },
      ];

      states.forEach((state) => {
        act(() => {
          useDashboardStore.getState().updateState(state);
        });
        rerender(<StateIndicatorPanel />);
      });

      // Final state should be reflected
      expect(screen.getByText('No active modifiers')).toBeInTheDocument();
      expect(screen.getByText('No active locks')).toBeInTheDocument();
      expect(screen.getByText('base')).toBeInTheDocument();
    });
  });

  describe('Complex State Scenarios', () => {
    it('should display all three sections with active state', () => {
      const state: DaemonState = {
        modifiers: ['MD_00', 'MD_01'],
        locks: ['LK_00'],
        layer: 'gaming',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      // All modifiers should be displayed
      expect(screen.getByText('MD_00')).toBeInTheDocument();
      expect(screen.getByText('MD_01')).toBeInTheDocument();

      // All locks should be displayed
      expect(screen.getByText('LK_00')).toBeInTheDocument();

      // Layer should be displayed
      expect(screen.getByText('gaming')).toBeInTheDocument();

      // No empty states should be shown
      expect(screen.queryByText('No active modifiers')).not.toBeInTheDocument();
      expect(screen.queryByText('No active locks')).not.toBeInTheDocument();
    });

    it('should handle maximum modifiers scenario (simulate many modifiers)', () => {
      // Test with a large number of modifiers
      const modifiers = Array.from({ length: 50 }, (_, i) => `MD_${i.toString().padStart(2, '0')}`);
      const state: DaemonState = {
        modifiers,
        locks: [],
        layer: 'base',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      // Verify first and last modifiers are displayed
      expect(screen.getByText('MD_00')).toBeInTheDocument();
      expect(screen.getByText('MD_49')).toBeInTheDocument();
      expect(screen.queryByText('No active modifiers')).not.toBeInTheDocument();
    });

    it('should handle maximum locks scenario (simulate many locks)', () => {
      // Test with a large number of locks
      const locks = Array.from({ length: 50 }, (_, i) => `LK_${i.toString().padStart(2, '0')}`);
      const state: DaemonState = {
        modifiers: [],
        locks,
        layer: 'base',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      // Verify first and last locks are displayed
      expect(screen.getByText('LK_00')).toBeInTheDocument();
      expect(screen.getByText('LK_49')).toBeInTheDocument();
      expect(screen.queryByText('No active locks')).not.toBeInTheDocument();
    });

    it('should transition from populated to empty state', () => {
      const { rerender } = render(<StateIndicatorPanel />);

      // Start with populated state
      const populatedState: DaemonState = {
        modifiers: ['MD_00', 'MD_01'],
        locks: ['LK_00'],
        layer: 'gaming',
      };
      act(() => {
        useDashboardStore.getState().updateState(populatedState);
      });
      rerender(<StateIndicatorPanel />);

      expect(screen.getByText('MD_00')).toBeInTheDocument();
      expect(screen.getByText('LK_00')).toBeInTheDocument();

      // Transition to empty state
      const emptyState: DaemonState = {
        modifiers: [],
        locks: [],
        layer: 'base',
      };
      act(() => {
        useDashboardStore.getState().updateState(emptyState);
      });
      rerender(<StateIndicatorPanel />);

      expect(screen.getByText('No active modifiers')).toBeInTheDocument();
      expect(screen.getByText('No active locks')).toBeInTheDocument();
      expect(screen.getByText('base')).toBeInTheDocument();
    });

    it('should handle special characters in layer names', () => {
      const state: DaemonState = {
        modifiers: [],
        locks: [],
        layer: 'layer-with-dashes',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      expect(screen.getByText('layer-with-dashes')).toBeInTheDocument();
    });
  });

  describe('Accessibility Features', () => {
    it('should have status roles on all active badges', () => {
      const state: DaemonState = {
        modifiers: ['MD_00'],
        locks: ['LK_00'],
        layer: 'gaming',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      const statusElements = screen.getAllByRole('status');
      // Should have 3 status elements: 1 modifier, 1 lock, 1 layer
      expect(statusElements).toHaveLength(3);
    });

    it('should have descriptive aria-labels for all badges', () => {
      const state: DaemonState = {
        modifiers: ['MD_00'],
        locks: ['LK_00'],
        layer: 'gaming',
      };

      useDashboardStore.getState().updateState(state);
      render(<StateIndicatorPanel />);

      expect(screen.getByLabelText('Modifier MD_00 active')).toBeInTheDocument();
      expect(screen.getByLabelText('Lock LK_00 active')).toBeInTheDocument();
      expect(screen.getByLabelText('Layer gaming active')).toBeInTheDocument();
    });

    it('should maintain proper heading hierarchy', () => {
      render(<StateIndicatorPanel />);

      const headings = screen.getAllByRole('heading', { level: 3 });
      expect(headings).toHaveLength(3);
      expect(headings[0]).toHaveTextContent('Active Modifiers');
      expect(headings[1]).toHaveTextContent('Active Locks');
      expect(headings[2]).toHaveTextContent('Current Layer');
    });
  });
});
