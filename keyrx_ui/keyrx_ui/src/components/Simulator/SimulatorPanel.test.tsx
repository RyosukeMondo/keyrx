/**
 * SimulatorPanel.test.tsx - Tests for the main Simulator container component.
 *
 * Tests cover:
 * - Component rendering with all sections
 * - Configuration loading flow
 * - Simulation execution flow
 * - Auto-load from URL params/sessionStorage
 * - Error handling and display
 * - Loading states
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { axe, toHaveNoViolations } from 'jest-axe';
import { SimulatorPanel } from './SimulatorPanel';
import * as wasmCore from '../../wasm/core';
import * as simulatorNavigation from '../../utils/simulatorNavigation';

// Extend Vitest matchers with jest-axe
expect.extend(toHaveNoViolations);

// Mock the WASM core module
vi.mock('../../wasm/core', () => ({
  wasmCore: {
    init: vi.fn(),
    loadConfig: vi.fn(),
    simulate: vi.fn(),
  },
  WasmError: class WasmError extends Error {
    constructor(message: string) {
      super(message);
      this.name = 'WasmError';
    }
  },
}));

// Mock simulator navigation utilities
vi.mock('../../utils/simulatorNavigation', () => ({
  getPendingConfig: vi.fn(),
  clearPendingConfig: vi.fn(),
}));

// Mock child components
vi.mock('./ConfigLoader', () => ({
  ConfigLoader: ({ onLoad, isLoading, error }: {
    onLoad: (config: string) => void;
    isLoading: boolean;
    error: string | null;
  }) => (
    <div data-testid="config-loader">
      <button onClick={() => onLoad('test config')}>Load Config</button>
      {isLoading && <div>Loading...</div>}
      {error && <div role="alert">{error}</div>}
    </div>
  ),
}));

vi.mock('./ScenarioSelector', () => ({
  ScenarioSelector: ({ onRunScenario, disabled }: {
    onRunScenario: (seq: unknown) => void;
    disabled: boolean;
  }) => (
    <div data-testid="scenario-selector">
      <button onClick={() => onRunScenario({ events: [] })} disabled={disabled}>
        Run Scenario
      </button>
    </div>
  ),
}));

vi.mock('./EventSequenceEditor', () => ({
  EventSequenceEditor: ({ onSubmit, disabled }: {
    onSubmit: (seq: unknown) => void;
    disabled: boolean;
  }) => (
    <div data-testid="event-sequence-editor">
      <button onClick={() => onSubmit({ events: [] })} disabled={disabled}>
        Simulate Custom
      </button>
    </div>
  ),
}));

vi.mock('./SimulationResults', () => ({
  default: ({ result }: { result: unknown }) => (
    <div data-testid="simulation-results">
      Results: {JSON.stringify(result)}
    </div>
  ),
}));

vi.mock('./LatencyStats', () => ({
  LatencyStats: ({ stats }: { stats: unknown }) => (
    <div data-testid="latency-stats">
      Stats: {JSON.stringify(stats)}
    </div>
  ),
}));

describe('SimulatorPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (wasmCore.wasmCore.init as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);
    (simulatorNavigation.getPendingConfig as ReturnType<typeof vi.fn>).mockReturnValue(null);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should render all main sections', () => {
      render(<SimulatorPanel />);

      expect(screen.getByText('Configuration Simulator')).toBeInTheDocument();
      expect(screen.getByText('1. Load Configuration')).toBeInTheDocument();
      expect(screen.getByText('2. Select or Create Event Sequence')).toBeInTheDocument();
      expect(screen.getByTestId('config-loader')).toBeInTheDocument();
    });

    it('should show disabled message when no config is loaded', () => {
      render(<SimulatorPanel />);

      expect(screen.getByText('Load a configuration first to enable simulation')).toBeInTheDocument();
      expect(screen.queryByTestId('scenario-selector')).not.toBeInTheDocument();
    });

    it('should render status info', () => {
      render(<SimulatorPanel />);

      expect(screen.getByText(/Status:/)).toBeInTheDocument();
      expect(screen.getByText(/○ No configuration/)).toBeInTheDocument();
      expect(screen.getByText(/○ No simulation run/)).toBeInTheDocument();
    });
  });

  describe('Configuration Loading', () => {
    it('should load configuration successfully', async () => {
      const user = userEvent.setup();
      const mockConfigHandle = { id: 123 };
      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockResolvedValue(mockConfigHandle);

      render(<SimulatorPanel />);

      // Click load config button
      const loadButton = screen.getByText('Load Config');
      await user.click(loadButton);

      await waitFor(() => {
        expect(wasmCore.wasmCore.init).toHaveBeenCalled();
        expect(wasmCore.wasmCore.loadConfig).toHaveBeenCalledWith('test config');
      });

      // Should show scenario selector after config loads
      await waitFor(() => {
        expect(screen.getByTestId('scenario-selector')).toBeInTheDocument();
        expect(screen.getByTestId('event-sequence-editor')).toBeInTheDocument();
      });

      // Status should update
      expect(screen.getByText(/✓ Configuration loaded/)).toBeInTheDocument();
    });

    it('should handle config loading errors', async () => {
      const user = userEvent.setup();
      const errorMessage = 'Parse error at line 5';
      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockRejectedValue(
        new Error(errorMessage)
      );

      render(<SimulatorPanel />);

      const loadButton = screen.getByText('Load Config');
      await user.click(loadButton);

      await waitFor(() => {
        expect(screen.getByRole('alert')).toHaveTextContent(`Failed to load configuration: ${errorMessage}`);
      });

      // Should not show scenario selector
      expect(screen.queryByTestId('scenario-selector')).not.toBeInTheDocument();
    });

    it('should show loading state during config load', async () => {
      const user = userEvent.setup();
      let resolveLoad: (value: unknown) => void;
      const loadPromise = new Promise((resolve) => {
        resolveLoad = resolve;
      });
      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockReturnValue(loadPromise);

      render(<SimulatorPanel />);

      const loadButton = screen.getByText('Load Config');
      await user.click(loadButton);

      // Should show loading state
      await waitFor(() => {
        expect(screen.getByText('Loading...')).toBeInTheDocument();
      });

      // Resolve the promise
      resolveLoad!({ id: 123 });

      // Loading should disappear
      await waitFor(() => {
        expect(screen.queryByText('Loading...')).not.toBeInTheDocument();
      });
    });
  });

  describe('Simulation Execution', () => {
    it('should run simulation successfully', async () => {
      const user = userEvent.setup();
      const mockConfigHandle = { id: 123 };
      const mockResult = {
        timeline: [{ timestamp_us: 0, event_type: 'input', data: {} }],
        latency_stats: { min_us: 1, avg_us: 2, max_us: 3, p95_us: 2.5, p99_us: 2.9 },
      };

      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockResolvedValue(mockConfigHandle);
      (wasmCore.wasmCore.simulate as ReturnType<typeof vi.fn>).mockResolvedValue(mockResult);

      render(<SimulatorPanel />);

      // Load config first
      const loadButton = screen.getByText('Load Config');
      await user.click(loadButton);

      await waitFor(() => {
        expect(screen.getByTestId('scenario-selector')).toBeInTheDocument();
      });

      // Run simulation
      const runButton = screen.getByText('Run Scenario');
      await user.click(runButton);

      await waitFor(() => {
        expect(wasmCore.wasmCore.simulate).toHaveBeenCalledWith(mockConfigHandle, { events: [] });
      });

      // Should show results
      await waitFor(() => {
        expect(screen.getByTestId('simulation-results')).toBeInTheDocument();
        expect(screen.getByTestId('latency-stats')).toBeInTheDocument();
      });

      // Status should update
      expect(screen.getByText(/✓ Simulation complete \(1 events\)/)).toBeInTheDocument();
    });

    it('should handle simulation errors', async () => {
      const user = userEvent.setup();
      const mockConfigHandle = { id: 123 };
      const errorMessage = 'Invalid event sequence';

      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockResolvedValue(mockConfigHandle);
      (wasmCore.wasmCore.simulate as ReturnType<typeof vi.fn>).mockRejectedValue(
        new Error(errorMessage)
      );

      render(<SimulatorPanel />);

      // Load config first
      const loadButton = screen.getByText('Load Config');
      await user.click(loadButton);

      await waitFor(() => {
        expect(screen.getByTestId('scenario-selector')).toBeInTheDocument();
      });

      // Run simulation
      const runButton = screen.getByText('Run Scenario');
      await user.click(runButton);

      await waitFor(() => {
        expect(screen.getByRole('alert')).toHaveTextContent(`Simulation failed: ${errorMessage}`);
      });

      // Should not show results
      expect(screen.queryByTestId('simulation-results')).not.toBeInTheDocument();
    });

    it('should not allow simulation without loaded config', async () => {
      const user = userEvent.setup();

      render(<SimulatorPanel />);

      // Scenario selector should not be visible
      expect(screen.queryByTestId('scenario-selector')).not.toBeInTheDocument();
    });
  });

  describe('Auto-load Configuration', () => {
    it('should auto-load config from pending config on mount', async () => {
      const pendingConfig = 'auto-loaded config';
      const mockConfigHandle = { id: 456 };

      (simulatorNavigation.getPendingConfig as ReturnType<typeof vi.fn>).mockReturnValue(pendingConfig);
      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockResolvedValue(mockConfigHandle);

      render(<SimulatorPanel />);

      await waitFor(() => {
        expect(wasmCore.wasmCore.loadConfig).toHaveBeenCalledWith(pendingConfig);
        expect(simulatorNavigation.clearPendingConfig).toHaveBeenCalled();
      });

      // Should show auto-load notice
      await waitFor(() => {
        expect(screen.getByText('Configuration automatically loaded from editor')).toBeInTheDocument();
      });
    });

    it('should only attempt auto-load once', async () => {
      const pendingConfig = 'auto-loaded config';
      const mockConfigHandle = { id: 456 };

      (simulatorNavigation.getPendingConfig as ReturnType<typeof vi.fn>).mockReturnValue(pendingConfig);
      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockResolvedValue(mockConfigHandle);

      const { rerender } = render(<SimulatorPanel />);

      await waitFor(() => {
        expect(wasmCore.wasmCore.loadConfig).toHaveBeenCalledTimes(1);
      });

      // Re-render should not trigger another auto-load
      rerender(<SimulatorPanel />);

      // Still only called once
      expect(wasmCore.wasmCore.loadConfig).toHaveBeenCalledTimes(1);
    });

    it('should not auto-load if no pending config', () => {
      (simulatorNavigation.getPendingConfig as ReturnType<typeof vi.fn>).mockReturnValue(null);

      render(<SimulatorPanel />);

      expect(wasmCore.wasmCore.loadConfig).not.toHaveBeenCalled();
      expect(screen.queryByText('Configuration automatically loaded from editor')).not.toBeInTheDocument();
    });
  });

  describe('Error Handling', () => {
    it('should display WasmError messages correctly', async () => {
      const user = userEvent.setup();
      const wasmError = new wasmCore.WasmError('WASM-specific error');

      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockRejectedValue(wasmError);

      render(<SimulatorPanel />);

      const loadButton = screen.getByText('Load Config');
      await user.click(loadButton);

      await waitFor(() => {
        expect(screen.getByRole('alert')).toHaveTextContent('Failed to load configuration: WASM-specific error');
      });
    });

    it('should handle generic errors', async () => {
      const user = userEvent.setup();

      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockRejectedValue(
        new Error('Generic error')
      );

      render(<SimulatorPanel />);

      const loadButton = screen.getByText('Load Config');
      await user.click(loadButton);

      await waitFor(() => {
        expect(screen.getByRole('alert')).toHaveTextContent('Failed to load configuration: Generic error');
      });
    });

    it('should handle unknown errors', async () => {
      const user = userEvent.setup();

      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockRejectedValue('string error');

      render(<SimulatorPanel />);

      const loadButton = screen.getByText('Load Config');
      await user.click(loadButton);

      await waitFor(() => {
        expect(screen.getByRole('alert')).toHaveTextContent('Failed to load configuration: Unknown error occurred');
      });
    });
  });

  describe('User Flow Integration', () => {
    it('should support complete workflow: load config → run scenario → view results', async () => {
      const user = userEvent.setup();
      const mockConfigHandle = { id: 789 };
      const mockResult = {
        timeline: [
          { timestamp_us: 0, event_type: 'input', data: {} },
          { timestamp_us: 100, event_type: 'output', data: {} },
        ],
        latency_stats: { min_us: 1, avg_us: 2, max_us: 3, p95_us: 2.5, p99_us: 2.9 },
      };

      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockResolvedValue(mockConfigHandle);
      (wasmCore.wasmCore.simulate as ReturnType<typeof vi.fn>).mockResolvedValue(mockResult);

      render(<SimulatorPanel />);

      // Step 1: Load config
      expect(screen.getByText(/○ No configuration/)).toBeInTheDocument();
      const loadButton = screen.getByText('Load Config');
      await user.click(loadButton);

      await waitFor(() => {
        expect(screen.getByText(/✓ Configuration loaded/)).toBeInTheDocument();
      });

      // Step 2: Run scenario
      expect(screen.getByText(/○ No simulation run/)).toBeInTheDocument();
      const runButton = screen.getByText('Run Scenario');
      await user.click(runButton);

      await waitFor(() => {
        expect(screen.getByText(/✓ Simulation complete \(2 events\)/)).toBeInTheDocument();
      });

      // Step 3: Verify results displayed
      expect(screen.getByTestId('simulation-results')).toBeInTheDocument();
      expect(screen.getByTestId('latency-stats')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have no axe violations in initial state', async () => {
      const { container } = render(<SimulatorPanel />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations with loaded config', async () => {
      const user = userEvent.setup();
      const mockConfigHandle = { id: 123 };
      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockResolvedValue(mockConfigHandle);

      const { container } = render(<SimulatorPanel />);

      // Load config
      const loadButton = screen.getByText('Load Config');
      await user.click(loadButton);

      await waitFor(() => {
        expect(screen.getByTestId('scenario-selector')).toBeInTheDocument();
      });

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations with simulation results', async () => {
      const user = userEvent.setup();
      const mockConfigHandle = { id: 123 };
      const mockResult = {
        timeline: [{ timestamp_us: 0, event_type: 'input', data: {} }],
        latency_stats: { min_us: 1, avg_us: 2, max_us: 3, p95_us: 2.5, p99_us: 2.9 },
      };

      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockResolvedValue(mockConfigHandle);
      (wasmCore.wasmCore.simulate as ReturnType<typeof vi.fn>).mockResolvedValue(mockResult);

      const { container } = render(<SimulatorPanel />);

      // Load config and run simulation
      await user.click(screen.getByText('Load Config'));
      await waitFor(() => {
        expect(screen.getByTestId('scenario-selector')).toBeInTheDocument();
      });

      await user.click(screen.getByText('Run Scenario'));
      await waitFor(() => {
        expect(screen.getByTestId('simulation-results')).toBeInTheDocument();
      });

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper ARIA labels on main sections', () => {
      render(<SimulatorPanel />);

      // Main panel should have accessible label
      expect(screen.getByRole('heading', { name: 'Configuration Simulator' })).toBeInTheDocument();
      expect(screen.getByRole('heading', { name: '1. Load Configuration' })).toBeInTheDocument();
      expect(screen.getByRole('heading', { name: '2. Select or Create Event Sequence' })).toBeInTheDocument();
    });

    it('should have proper ARIA attributes for error alerts', async () => {
      const user = userEvent.setup();
      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockRejectedValue(
        new Error('Test error')
      );

      render(<SimulatorPanel />);

      const loadButton = screen.getByText('Load Config');
      await user.click(loadButton);

      await waitFor(() => {
        const alert = screen.getByRole('alert');
        expect(alert).toBeInTheDocument();
        expect(alert).toHaveTextContent('Failed to load configuration: Test error');
      });
    });

    it('should support keyboard navigation through main sections', async () => {
      const user = userEvent.setup();
      const mockConfigHandle = { id: 123 };
      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockResolvedValue(mockConfigHandle);

      render(<SimulatorPanel />);

      // Tab to load button and activate with Enter
      await user.tab();
      expect(screen.getByText('Load Config')).toHaveFocus();
      await user.keyboard('{Enter}');

      await waitFor(() => {
        expect(wasmCore.wasmCore.loadConfig).toHaveBeenCalled();
      });
    });

    it('should announce loading states to screen readers', async () => {
      const user = userEvent.setup();
      let resolveLoad: (value: unknown) => void;
      const loadPromise = new Promise((resolve) => {
        resolveLoad = resolve;
      });
      (wasmCore.wasmCore.loadConfig as ReturnType<typeof vi.fn>).mockReturnValue(loadPromise);

      render(<SimulatorPanel />);

      const loadButton = screen.getByText('Load Config');
      await user.click(loadButton);

      // Loading message should be in the document
      await waitFor(() => {
        expect(screen.getByText('Loading...')).toBeInTheDocument();
      });

      resolveLoad!({ id: 123 });

      await waitFor(() => {
        expect(screen.queryByText('Loading...')).not.toBeInTheDocument();
      });
    });
  });
});
