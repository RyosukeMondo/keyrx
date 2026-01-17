import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { StateSnapshot } from './StateSnapshot';
import type { StateSnapshotProps } from './StateSnapshot';

describe('StateSnapshot', () => {
  const defaultState: StateSnapshotProps['state'] = {
    activeLayer: 'Base',
    modifiers: [],
    locks: [],
    tapHoldTimers: 0,
    queuedEvents: 0,
  };

  describe('Rendering', () => {
    it('renders all state sections', () => {
      render(<StateSnapshot state={defaultState} />);

      expect(
        screen.getByRole('heading', { name: 'Active Layer' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('heading', { name: 'Tap/Hold Timers' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('heading', { name: 'Active Modifiers' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('heading', { name: 'Active Locks' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('heading', { name: 'Queued Events' })
      ).toBeInTheDocument();
    });

    it('renders active layer value', () => {
      render(<StateSnapshot state={{ ...defaultState, activeLayer: 'Nav' }} />);

      const layerStatus = screen.getByRole('status', {
        name: /Active layer: Nav/i,
      });
      expect(layerStatus).toBeInTheDocument();
      expect(layerStatus).toHaveTextContent('Nav');
    });

    it('renders tap/hold timer count', () => {
      render(<StateSnapshot state={{ ...defaultState, tapHoldTimers: 3 }} />);

      const timerStatus = screen.getByRole('status', {
        name: /3 active tap\/hold timers/i,
      });
      expect(timerStatus).toBeInTheDocument();
      expect(timerStatus).toHaveTextContent('3 active');
    });

    it('renders queued events count', () => {
      render(<StateSnapshot state={{ ...defaultState, queuedEvents: 5 }} />);

      const queueStatus = screen.getByRole('status', {
        name: /5 queued events/i,
      });
      expect(queueStatus).toBeInTheDocument();
      expect(queueStatus).toHaveTextContent('5');
    });
  });

  describe('Modifiers Display', () => {
    it('renders "None" when no modifiers active', () => {
      render(<StateSnapshot state={defaultState} />);

      const modifiersStatus = screen.getByRole('status', {
        name: /Active modifiers: None/i,
      });
      expect(modifiersStatus).toHaveTextContent('None');
    });

    it('renders single modifier', () => {
      render(
        <StateSnapshot state={{ ...defaultState, modifiers: ['Ctrl'] }} />
      );

      const modifiersStatus = screen.getByRole('status', {
        name: /Active modifiers: Ctrl/i,
      });
      expect(modifiersStatus).toHaveTextContent('Ctrl');
    });

    it('renders multiple modifiers as comma-separated list', () => {
      render(
        <StateSnapshot
          state={{ ...defaultState, modifiers: ['Ctrl', 'Shift', 'Alt'] }}
        />
      );

      const modifiersStatus = screen.getByRole('status', {
        name: /Active modifiers: Ctrl, Shift, Alt/i,
      });
      expect(modifiersStatus).toHaveTextContent('Ctrl, Shift, Alt');
    });
  });

  describe('Locks Display', () => {
    it('renders "None" when no locks active', () => {
      render(<StateSnapshot state={defaultState} />);

      const locksStatus = screen.getByRole('status', {
        name: /Active locks: None/i,
      });
      expect(locksStatus).toHaveTextContent('None');
    });

    it('renders single lock', () => {
      render(
        <StateSnapshot state={{ ...defaultState, locks: ['CapsLock'] }} />
      );

      const locksStatus = screen.getByRole('status', {
        name: /Active locks: CapsLock/i,
      });
      expect(locksStatus).toHaveTextContent('CapsLock');
    });

    it('renders multiple locks as comma-separated list', () => {
      render(
        <StateSnapshot
          state={{ ...defaultState, locks: ['CapsLock', 'NumLock'] }}
        />
      );

      const locksStatus = screen.getByRole('status', {
        name: /Active locks: CapsLock, NumLock/i,
      });
      expect(locksStatus).toHaveTextContent('CapsLock, NumLock');
    });
  });

  describe('Edge Cases', () => {
    it('handles zero values correctly', () => {
      render(<StateSnapshot state={defaultState} />);

      expect(screen.getByText('0 active')).toBeInTheDocument();
      expect(screen.getByText('0')).toBeInTheDocument();
    });

    it('handles empty arrays correctly', () => {
      render(<StateSnapshot state={defaultState} />);

      const noneTexts = screen.getAllByText('None');
      expect(noneTexts).toHaveLength(2); // Modifiers and Locks
    });

    it('handles complex layer names', () => {
      render(
        <StateSnapshot
          state={{ ...defaultState, activeLayer: 'Function Keys Layer 2' }}
        />
      );

      const layerStatus = screen.getByRole('status', {
        name: /Active layer: Function Keys Layer 2/i,
      });
      expect(layerStatus).toHaveTextContent('Function Keys Layer 2');
    });

    it('handles large numbers', () => {
      render(
        <StateSnapshot
          state={{ ...defaultState, tapHoldTimers: 999, queuedEvents: 1000 }}
        />
      );

      expect(screen.getByText('999 active')).toBeInTheDocument();
      expect(screen.getByText('1000')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('has proper ARIA labels for each section', () => {
      const state: StateSnapshotProps['state'] = {
        activeLayer: 'Nav',
        modifiers: ['Ctrl'],
        locks: ['CapsLock'],
        tapHoldTimers: 2,
        queuedEvents: 5,
      };
      render(<StateSnapshot state={state} />);

      expect(
        screen.getByRole('status', { name: /Active layer: Nav/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('status', { name: /2 active tap\/hold timers/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('status', { name: /Active modifiers: Ctrl/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('status', { name: /Active locks: CapsLock/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('status', { name: /5 queued events/i })
      ).toBeInTheDocument();
    });

    it('has a group role for the container', () => {
      render(<StateSnapshot state={defaultState} />);

      expect(
        screen.getByRole('group', { name: /Daemon state information/i })
      ).toBeInTheDocument();
    });
  });

  describe('Styling', () => {
    it('applies correct color classes to each section', () => {
      const state: StateSnapshotProps['state'] = {
        activeLayer: 'Base',
        modifiers: ['Ctrl'],
        locks: ['CapsLock'],
        tapHoldTimers: 1,
        queuedEvents: 2,
      };
      render(<StateSnapshot state={state} />);

      // Check color classes are present
      const layerValue = screen.getByText('Base');
      expect(layerValue).toHaveClass('text-blue-400');

      const timersValue = screen.getByText('1 active');
      expect(timersValue).toHaveClass('text-yellow-400');

      const modifiersValue = screen.getByText('Ctrl');
      expect(modifiersValue).toHaveClass('text-green-400');

      const locksValue = screen.getByText('CapsLock');
      expect(locksValue).toHaveClass('text-purple-400');

      const queueValue = screen.getByText('2');
      expect(queueValue).toHaveClass('text-red-400');
    });
  });

  describe('Snapshot', () => {
    it('matches snapshot with full state', () => {
      const state: StateSnapshotProps['state'] = {
        activeLayer: 'Navigation',
        modifiers: ['Ctrl', 'Shift'],
        locks: ['CapsLock', 'NumLock'],
        tapHoldTimers: 3,
        queuedEvents: 10,
      };
      const { container } = render(<StateSnapshot state={state} />);
      expect(container.firstChild).toMatchSnapshot();
    });

    it('matches snapshot with empty state', () => {
      const { container } = render(<StateSnapshot state={defaultState} />);
      expect(container.firstChild).toMatchSnapshot();
    });
  });
});
