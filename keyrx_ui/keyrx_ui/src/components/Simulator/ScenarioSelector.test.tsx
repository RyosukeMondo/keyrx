/**
 * ScenarioSelector.test.tsx - Tests for the ScenarioSelector component.
 *
 * Tests cover:
 * - Dropdown rendering with all scenarios
 * - Scenario selection and description display
 * - Run button functionality
 * - Disabled and loading states
 * - Keyboard shortcuts (Enter to run)
 * - Accessibility features
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { axe, toHaveNoViolations } from 'jest-axe';
import { ScenarioSelector } from './ScenarioSelector';

// Extend Vitest matchers with jest-axe
expect.extend(toHaveNoViolations);

// Mock the scenarios module
vi.mock('../../utils/scenarios', () => ({
  BUILT_IN_SCENARIOS: [
    {
      id: 'tap-hold-under',
      name: 'Tap-Hold Under Threshold',
      description: 'Simulates a key press and release within 200ms threshold to test tap behavior.',
      generator: vi.fn(() => ({
        events: [
          { key_code: 30, event_type: 'press', timestamp_us: 0 },
          { key_code: 30, event_type: 'release', timestamp_us: 150000 },
        ],
      })),
    },
    {
      id: 'tap-hold-over',
      name: 'Tap-Hold Over Threshold',
      description: 'Simulates a key press held beyond 200ms threshold to test hold behavior.',
      generator: vi.fn(() => ({
        events: [
          { key_code: 30, event_type: 'press', timestamp_us: 0 },
          { key_code: 30, event_type: 'release', timestamp_us: 250000 },
        ],
      })),
    },
    {
      id: 'layer-switch',
      name: 'Layer Switch',
      description: 'Activates a layer modifier and presses a key to test layer switching.',
      generator: vi.fn(() => ({
        events: [
          { key_code: 42, event_type: 'press', timestamp_us: 0 },
          { key_code: 30, event_type: 'press', timestamp_us: 50000 },
          { key_code: 30, event_type: 'release', timestamp_us: 100000 },
          { key_code: 42, event_type: 'release', timestamp_us: 150000 },
        ],
      })),
    },
    {
      id: 'modifier-combo',
      name: 'Modifier Combination (Shift+Ctrl+A)',
      description: 'Tests multiple modifier keys pressed simultaneously with a regular key.',
      generator: vi.fn(() => ({
        events: [
          { key_code: 225, event_type: 'press', timestamp_us: 0 },
          { key_code: 224, event_type: 'press', timestamp_us: 10000 },
          { key_code: 4, event_type: 'press', timestamp_us: 20000 },
          { key_code: 4, event_type: 'release', timestamp_us: 70000 },
          { key_code: 224, event_type: 'release', timestamp_us: 80000 },
          { key_code: 225, event_type: 'release', timestamp_us: 90000 },
        ],
      })),
    },
  ],
}));

describe('ScenarioSelector', () => {
  const mockOnRunScenario = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should render dropdown with all scenarios', () => {
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      expect(screen.getByLabelText('Select a test scenario')).toBeInTheDocument();
      expect(screen.getByText('Tap-Hold Under Threshold')).toBeInTheDocument();
      expect(screen.getByText('Tap-Hold Over Threshold')).toBeInTheDocument();
      expect(screen.getByText('Layer Switch')).toBeInTheDocument();
      expect(screen.getByText('Modifier Combination (Shift+Ctrl+A)')).toBeInTheDocument();
    });

    it('should render run button', () => {
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      expect(screen.getByText('Run Scenario')).toBeInTheDocument();
    });

    it('should show description for first scenario by default', () => {
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      expect(screen.getByText(/Simulates a key press and release within 200ms/)).toBeInTheDocument();
    });
  });

  describe('Scenario Selection', () => {
    it('should update description when selecting different scenario', async () => {
      const user = userEvent.setup();
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      const dropdown = screen.getByLabelText('Select a test scenario');
      await user.selectOptions(dropdown, 'layer-switch');

      expect(screen.getByText(/Activates a layer modifier and presses a key/)).toBeInTheDocument();
    });

    it('should maintain selected scenario across re-renders', async () => {
      const user = userEvent.setup();
      const { rerender } = render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      const dropdown = screen.getByLabelText('Select a test scenario');
      await user.selectOptions(dropdown, 'modifier-combo');

      expect(dropdown).toHaveValue('modifier-combo');

      // Re-render
      rerender(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      expect(dropdown).toHaveValue('modifier-combo');
    });

    it('should show all scenario options', async () => {
      const user = userEvent.setup();
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      const dropdown = screen.getByLabelText('Select a test scenario');

      // Check all options are available
      await user.click(dropdown);

      const options = screen.getAllByRole('option');
      expect(options).toHaveLength(4);
      expect(options[0]).toHaveTextContent('Tap-Hold Under Threshold');
      expect(options[1]).toHaveTextContent('Tap-Hold Over Threshold');
      expect(options[2]).toHaveTextContent('Layer Switch');
      expect(options[3]).toHaveTextContent('Modifier Combination (Shift+Ctrl+A)');
    });
  });

  describe('Run Scenario', () => {
    it('should call onRunScenario with generated event sequence', async () => {
      const user = userEvent.setup();
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      const runButton = screen.getByText('Run Scenario');
      await user.click(runButton);

      expect(mockOnRunScenario).toHaveBeenCalledWith({
        events: [
          { key_code: 30, event_type: 'press', timestamp_us: 0 },
          { key_code: 30, event_type: 'release', timestamp_us: 150000 },
        ],
      });
    });

    it('should run correct scenario after selection change', async () => {
      const user = userEvent.setup();
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      const dropdown = screen.getByLabelText('Select a test scenario');
      await user.selectOptions(dropdown, 'tap-hold-over');

      const runButton = screen.getByText('Run Scenario');
      await user.click(runButton);

      expect(mockOnRunScenario).toHaveBeenCalledWith({
        events: [
          { key_code: 30, event_type: 'press', timestamp_us: 0 },
          { key_code: 30, event_type: 'release', timestamp_us: 250000 },
        ],
      });
    });

    it('should not run when disabled', async () => {
      const user = userEvent.setup();
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} disabled={true} />);

      const runButton = screen.getByText('Run Scenario');
      expect(runButton).toBeDisabled();

      await user.click(runButton);

      expect(mockOnRunScenario).not.toHaveBeenCalled();
    });

    it('should not run when loading', async () => {
      const user = userEvent.setup();
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} isLoading={true} />);

      const runButton = screen.getByText('Running...');
      expect(runButton).toBeDisabled();

      await user.click(runButton);

      expect(mockOnRunScenario).not.toHaveBeenCalled();
    });
  });

  describe('Keyboard Shortcuts', () => {
    it('should run scenario on Enter key press', async () => {
      const user = userEvent.setup();
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      const dropdown = screen.getByLabelText('Select a test scenario');
      await user.click(dropdown);
      await user.keyboard('{Enter}');

      expect(mockOnRunScenario).toHaveBeenCalled();
    });

    it('should not run on Enter when disabled', async () => {
      const user = userEvent.setup();
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} disabled={true} />);

      const dropdown = screen.getByLabelText('Select a test scenario');
      await user.click(dropdown);
      await user.keyboard('{Enter}');

      expect(mockOnRunScenario).not.toHaveBeenCalled();
    });

    it('should not run on Enter when loading', async () => {
      const user = userEvent.setup();
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} isLoading={true} />);

      const dropdown = screen.getByLabelText('Select a test scenario');
      await user.click(dropdown);
      await user.keyboard('{Enter}');

      expect(mockOnRunScenario).not.toHaveBeenCalled();
    });
  });

  describe('Disabled State', () => {
    it('should disable dropdown when disabled', () => {
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} disabled={true} />);

      const dropdown = screen.getByLabelText('Select a test scenario');
      expect(dropdown).toBeDisabled();
    });

    it('should disable run button when disabled', () => {
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} disabled={true} />);

      const runButton = screen.getByText('Run Scenario');
      expect(runButton).toBeDisabled();
    });

    it('should show disabled tooltip', () => {
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} disabled={true} />);

      const runButton = screen.getByText('Run Scenario');
      expect(runButton).toHaveAttribute('title', 'Load a configuration first to enable scenarios');
    });
  });

  describe('Loading State', () => {
    it('should show loading text on button', () => {
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} isLoading={true} />);

      expect(screen.getByText('Running...')).toBeInTheDocument();
    });

    it('should disable all controls during loading', () => {
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} isLoading={true} />);

      expect(screen.getByLabelText('Select a test scenario')).toBeDisabled();
      expect(screen.getByText('Running...')).toBeDisabled();
    });

    it('should show loading tooltip', () => {
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} isLoading={true} />);

      const runButton = screen.getByText('Running...');
      expect(runButton).toHaveAttribute('title', 'Simulation in progress...');
    });
  });

  describe('Accessibility', () => {
    it('should have no axe violations in default state', async () => {
      const { container } = render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations when disabled', async () => {
      const { container } = render(<ScenarioSelector onRunScenario={mockOnRunScenario} disabled={true} />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations when loading', async () => {
      const { container } = render(<ScenarioSelector onRunScenario={mockOnRunScenario} isLoading={true} />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper aria labels', () => {
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      expect(screen.getByLabelText('Select a test scenario')).toBeInTheDocument();
      expect(screen.getByLabelText(/Run Tap-Hold Under Threshold/)).toBeInTheDocument();
    });

    it('should have aria-live region for description', () => {
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      const description = screen.getByText(/Simulates a key press and release within 200ms/).closest('div');
      expect(description).toHaveAttribute('aria-live', 'polite');
    });

    it('should update aria-label when selecting different scenario', async () => {
      const user = userEvent.setup();
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      const dropdown = screen.getByLabelText('Select a test scenario');
      await user.selectOptions(dropdown, 'layer-switch');

      const runButton = screen.getByLabelText(/Run Layer Switch/);
      expect(runButton).toBeInTheDocument();
    });

    it('should have descriptive tooltips for different states', () => {
      const { rerender } = render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      let runButton = screen.getByText('Run Scenario');
      expect(runButton).toHaveAttribute('title', 'Run the selected scenario (Enter)');

      rerender(<ScenarioSelector onRunScenario={mockOnRunScenario} disabled={true} />);
      runButton = screen.getByText('Run Scenario');
      expect(runButton).toHaveAttribute('title', 'Load a configuration first to enable scenarios');

      rerender(<ScenarioSelector onRunScenario={mockOnRunScenario} isLoading={true} />);
      runButton = screen.getByText('Running...');
      expect(runButton).toHaveAttribute('title', 'Simulation in progress...');
    });
  });

  describe('Error Handling', () => {
    it('should handle generator errors gracefully', async () => {
      const user = userEvent.setup();
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      // Mock a generator that throws
      const { BUILT_IN_SCENARIOS } = await import('../../utils/scenarios');
      const originalGenerator = BUILT_IN_SCENARIOS[0].generator;
      BUILT_IN_SCENARIOS[0].generator = vi.fn(() => {
        throw new Error('Generator failed');
      });

      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      const runButton = screen.getByText('Run Scenario');
      await user.click(runButton);

      expect(consoleErrorSpy).toHaveBeenCalledWith('Failed to generate scenario:', expect.any(Error));
      expect(mockOnRunScenario).not.toHaveBeenCalled();

      // Restore
      BUILT_IN_SCENARIOS[0].generator = originalGenerator;
      consoleErrorSpy.mockRestore();
    });
  });

  describe('Keyboard Navigation', () => {
    it('should support keyboard navigation through dropdown', async () => {
      const user = userEvent.setup();
      render(<ScenarioSelector onRunScenario={mockOnRunScenario} />);

      const dropdown = screen.getByLabelText('Select a test scenario');

      // Focus dropdown
      await user.click(dropdown);

      // Navigate with arrow keys
      await user.keyboard('{ArrowDown}');

      // Description should update
      expect(screen.getByText(/Simulates a key press held beyond 200ms/)).toBeInTheDocument();
    });
  });
});
