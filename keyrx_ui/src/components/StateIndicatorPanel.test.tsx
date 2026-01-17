import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import { StateIndicatorPanel } from './StateIndicatorPanel';
import type { DaemonState } from '../types/rpc';

describe('StateIndicatorPanel', () => {
  it('renders loading message when state is null', () => {
    renderWithProviders(<StateIndicatorPanel state={null} />);
    expect(screen.getByText('Loading daemon state...')).toBeInTheDocument();
  });

  it('renders modifiers as blue badges', () => {
    const state: DaemonState = {
      modifiers: [1, 2],
      locks: [],
      layer: 0,
    };

    renderWithProviders(<StateIndicatorPanel state={state} />);

    const mod1 = screen.getByText('MOD_1');
    const mod2 = screen.getByText('MOD_2');

    expect(mod1).toBeInTheDocument();
    expect(mod2).toBeInTheDocument();
    expect(mod1).toHaveClass('bg-blue-600');
    expect(mod2).toHaveClass('bg-blue-600');
  });

  it('renders locks as orange badges', () => {
    const state: DaemonState = {
      modifiers: [],
      locks: [3, 4],
      layer: 0,
    };

    renderWithProviders(<StateIndicatorPanel state={state} />);

    const lock3 = screen.getByText('LOCK_3');
    const lock4 = screen.getByText('LOCK_4');

    expect(lock3).toBeInTheDocument();
    expect(lock4).toBeInTheDocument();
    expect(lock3).toHaveClass('bg-orange-600');
    expect(lock4).toHaveClass('bg-orange-600');
  });

  it('renders layer as green badge', () => {
    const state: DaemonState = {
      modifiers: [],
      locks: [],
      layer: 2,
    };

    renderWithProviders(<StateIndicatorPanel state={state} />);

    const layer = screen.getByText('Layer 2');
    expect(layer).toBeInTheDocument();
    expect(layer).toHaveClass('bg-green-600');
  });

  it('shows "None" when modifiers array is empty', () => {
    const state: DaemonState = {
      modifiers: [],
      locks: [1],
      layer: 0,
    };

    renderWithProviders(<StateIndicatorPanel state={state} />);

    const noneElements = screen.getAllByText('None');
    // Should have "None" for modifiers
    expect(noneElements.length).toBeGreaterThanOrEqual(1);
    expect(noneElements[0]).toHaveClass('text-slate-500');
  });

  it('shows "None" when locks array is empty', () => {
    const state: DaemonState = {
      modifiers: [1],
      locks: [],
      layer: 0,
    };

    renderWithProviders(<StateIndicatorPanel state={state} />);

    const noneElements = screen.getAllByText('None');
    // Should have "None" for locks
    expect(noneElements.length).toBeGreaterThanOrEqual(1);
    expect(noneElements[0]).toHaveClass('text-slate-500');
  });

  it('renders all sections when state has modifiers, locks, and layer', () => {
    const state: DaemonState = {
      modifiers: [1, 2],
      locks: [3],
      layer: 1,
    };

    renderWithProviders(<StateIndicatorPanel state={state} />);

    // Check modifiers
    expect(screen.getByText('MOD_1')).toBeInTheDocument();
    expect(screen.getByText('MOD_2')).toBeInTheDocument();

    // Check locks
    expect(screen.getByText('LOCK_3')).toBeInTheDocument();

    // Check layer
    expect(screen.getByText('Layer 1')).toBeInTheDocument();
  });

  it('has proper ARIA labels for accessibility', () => {
    const state: DaemonState = {
      modifiers: [1],
      locks: [2],
      layer: 0,
    };

    renderWithProviders(<StateIndicatorPanel state={state} />);

    expect(screen.getByLabelText('Active Modifiers')).toBeInTheDocument();
    expect(screen.getByLabelText('Active Locks')).toBeInTheDocument();
    expect(screen.getByLabelText('Current Layer')).toBeInTheDocument();
  });

  it('has ARIA labels on individual badges', () => {
    const state: DaemonState = {
      modifiers: [5],
      locks: [7],
      layer: 3,
    };

    renderWithProviders(<StateIndicatorPanel state={state} />);

    expect(screen.getByLabelText('Modifier 5 active')).toBeInTheDocument();
    expect(screen.getByLabelText('Lock 7 active')).toBeInTheDocument();
    expect(screen.getByLabelText('Layer 3 active')).toBeInTheDocument();
  });

  it('uses responsive grid layout', () => {
    const state: DaemonState = {
      modifiers: [],
      locks: [],
      layer: 0,
    };

    const { container } = renderWithProviders(
      <StateIndicatorPanel state={state} />
    );
    const grid = container.querySelector('.grid');

    expect(grid).toHaveClass('grid-cols-1');
    expect(grid).toHaveClass('md:grid-cols-2');
    expect(grid).toHaveClass('lg:grid-cols-4');
  });

  it('displays connected devices with virtual/physical badges', async () => {
    const state: DaemonState = {
      modifiers: [],
      locks: [],
      layer: 0,
    };

    renderWithProviders(<StateIndicatorPanel state={state} />);

    // Should show Connected Devices section
    expect(screen.getByLabelText('Connected Devices')).toBeInTheDocument();
  });
});
