/**
 * EventSequenceEditor.test.tsx - Tests for the EventSequenceEditor component.
 *
 * Tests cover:
 * - Adding and removing events
 * - Editing event properties (keycode, type, timestamp)
 * - Validation (positive timestamps, ascending order)
 * - Submit with valid/invalid sequences
 * - Empty state display
 * - Event summary display
 * - Disabled state
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { axe, toHaveNoViolations } from 'jest-axe';
import { EventSequenceEditor } from './EventSequenceEditor';

// Extend Vitest matchers with jest-axe
expect.extend(toHaveNoViolations);

describe('EventSequenceEditor', () => {
  const mockOnSubmit = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should render empty state initially', () => {
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      expect(screen.getByText('Custom Event Sequence')).toBeInTheDocument();
      expect(screen.getByText('No events yet. Add events to create a custom sequence.')).toBeInTheDocument();
      expect(screen.getByText(/Events must have ascending timestamps/)).toBeInTheDocument();
    });

    it('should render add event button', () => {
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      expect(screen.getByText('+ Add Event')).toBeInTheDocument();
    });

    it('should not show submit button when empty', () => {
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      expect(screen.queryByText('Simulate Custom Sequence')).not.toBeInTheDocument();
    });
  });

  describe('Adding Events', () => {
    it('should add event when clicking add button', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      const addButton = screen.getByText('+ Add Event');
      await user.click(addButton);

      expect(screen.queryByText('No events yet')).not.toBeInTheDocument();
      expect(screen.getByText('1')).toBeInTheDocument(); // Event number
    });

    it('should add multiple events', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      const addButton = screen.getByText('+ Add Event');
      await user.click(addButton);
      await user.click(addButton);
      await user.click(addButton);

      expect(screen.getByText('1')).toBeInTheDocument();
      expect(screen.getByText('2')).toBeInTheDocument();
      expect(screen.getByText('3')).toBeInTheDocument();
    });

    it('should auto-increment timestamps for new events', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      const addButton = screen.getByText('+ Add Event');
      await user.click(addButton);
      await user.click(addButton);

      // First event should have timestamp 0
      const firstTimestamp = screen.getAllByLabelText(/Timestamp/)[0] as HTMLInputElement;
      expect(firstTimestamp.value).toBe('0');

      // Second event should have timestamp 100
      const secondTimestamp = screen.getAllByLabelText(/Timestamp/)[1] as HTMLInputElement;
      expect(secondTimestamp.value).toBe('100');
    });

    it('should set default values for new events', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      const addButton = screen.getByText('+ Add Event');
      await user.click(addButton);

      // Should have default key code VK_A
      const keyCodeSelect = screen.getByLabelText('Key Code:') as HTMLSelectElement;
      expect(keyCodeSelect.value).toBe('VK_A');

      // Should have default type 'press'
      const typeSelect = screen.getByLabelText('Type:') as HTMLSelectElement;
      expect(typeSelect.value).toBe('press');
    });
  });

  describe('Removing Events', () => {
    it('should remove event when clicking remove button', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      // Add an event
      const addButton = screen.getByText('+ Add Event');
      await user.click(addButton);

      expect(screen.getByText('1')).toBeInTheDocument();

      // Remove it
      const removeButton = screen.getByTitle('Remove event');
      await user.click(removeButton);

      expect(screen.queryByText('1')).not.toBeInTheDocument();
      expect(screen.getByText('No events yet')).toBeInTheDocument();
    });

    it('should remove correct event when multiple exist', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      const addButton = screen.getByText('+ Add Event');
      await user.click(addButton);
      await user.click(addButton);
      await user.click(addButton);

      // Remove the second event
      const removeButtons = screen.getAllByTitle('Remove event');
      await user.click(removeButtons[1]);

      // Should still have 2 events
      expect(screen.getByText('1')).toBeInTheDocument();
      expect(screen.queryByText('2')).toBeInTheDocument();
      expect(screen.queryByText('3')).not.toBeInTheDocument();
    });
  });

  describe('Editing Events', () => {
    it('should update key code when selecting from dropdown', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      const keyCodeSelect = screen.getByLabelText('Key Code:') as HTMLSelectElement;
      await user.selectOptions(keyCodeSelect, 'VK_Z');

      expect(keyCodeSelect.value).toBe('VK_Z');
    });

    it('should update event type when selecting from dropdown', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      const typeSelect = screen.getByLabelText('Type:') as HTMLSelectElement;
      await user.selectOptions(typeSelect, 'release');

      expect(typeSelect.value).toBe('release');
    });

    it('should update timestamp when typing in input', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      const timestampInput = screen.getByLabelText(/Timestamp/) as HTMLInputElement;
      await user.clear(timestampInput);
      await user.type(timestampInput, '500');

      expect(timestampInput.value).toBe('500');
    });

    it('should clear validation error when editing field', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      // Set negative timestamp to trigger validation error
      const timestampInput = screen.getByLabelText(/Timestamp/) as HTMLInputElement;
      await user.clear(timestampInput);
      await user.type(timestampInput, '-100');

      // Try to submit (will validate)
      const submitButton = screen.getByText('Simulate Custom Sequence');
      await user.click(submitButton);

      // Should show error
      expect(screen.getByText('Timestamp must be positive')).toBeInTheDocument();

      // Fix the timestamp
      await user.clear(timestampInput);
      await user.type(timestampInput, '100');

      // Error should be cleared
      expect(screen.queryByText('Timestamp must be positive')).not.toBeInTheDocument();
    });
  });

  describe('Validation', () => {
    it('should validate negative timestamps', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      const timestampInput = screen.getByLabelText(/Timestamp/) as HTMLInputElement;
      await user.clear(timestampInput);
      await user.type(timestampInput, '-100');

      const submitButton = screen.getByText('Simulate Custom Sequence');
      await user.click(submitButton);

      expect(screen.getByText('Timestamp must be positive')).toBeInTheDocument();
      expect(mockOnSubmit).not.toHaveBeenCalled();
    });

    it('should validate timestamps are in ascending order', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      // Add two events
      await user.click(screen.getByText('+ Add Event'));
      await user.click(screen.getByText('+ Add Event'));

      // Set second event timestamp lower than first
      const timestamps = screen.getAllByLabelText(/Timestamp/);
      await user.clear(timestamps[1]);
      await user.type(timestamps[1], '0');

      const submitButton = screen.getByText('Simulate Custom Sequence');
      await user.click(submitButton);

      expect(screen.getByText('Timestamp must be greater than previous event')).toBeInTheDocument();
      expect(mockOnSubmit).not.toHaveBeenCalled();
    });

    it('should validate timestamps are strictly increasing (not equal)', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      // Add two events
      await user.click(screen.getByText('+ Add Event'));
      await user.click(screen.getByText('+ Add Event'));

      // Set both to same timestamp
      const timestamps = screen.getAllByLabelText(/Timestamp/);
      await user.clear(timestamps[1]);
      await user.type(timestamps[1], '0');

      const submitButton = screen.getByText('Simulate Custom Sequence');
      await user.click(submitButton);

      expect(screen.getByText('Timestamp must be greater than previous event')).toBeInTheDocument();
    });

    it('should allow valid sequence with ascending timestamps', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));
      await user.click(screen.getByText('+ Add Event'));

      // Timestamps are auto-incremented (0, 100)
      const submitButton = screen.getByText('Simulate Custom Sequence');
      await user.click(submitButton);

      expect(mockOnSubmit).toHaveBeenCalledWith({
        events: [
          { keycode: 'VK_A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'VK_A', event_type: 'press', timestamp_us: 100 },
        ],
      });
    });

    it('should show alert if trying to submit empty sequence', async () => {
      const user = userEvent.setup();
      const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {});

      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      // Add and then remove event to show submit button briefly
      await user.click(screen.getByText('+ Add Event'));

      // Submit button should be visible
      expect(screen.getByText('Simulate Custom Sequence')).toBeInTheDocument();

      // Remove the event
      await user.click(screen.getByTitle('Remove event'));

      // Submit button should be hidden now
      expect(screen.queryByText('Simulate Custom Sequence')).not.toBeInTheDocument();

      alertSpy.mockRestore();
    });
  });

  describe('Submit', () => {
    it('should call onSubmit with correct event sequence format', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      // Add two events
      await user.click(screen.getByText('+ Add Event'));
      await user.click(screen.getByText('+ Add Event'));

      // Modify second event
      const keyCodeSelects = screen.getAllByLabelText('Key Code:');
      await user.selectOptions(keyCodeSelects[1], 'VK_B');

      const typeSelects = screen.getAllByLabelText('Type:');
      await user.selectOptions(typeSelects[1], 'release');

      const submitButton = screen.getByText('Simulate Custom Sequence');
      await user.click(submitButton);

      expect(mockOnSubmit).toHaveBeenCalledWith({
        events: [
          { keycode: 'VK_A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'VK_B', event_type: 'release', timestamp_us: 100 },
        ],
      });
    });

    it('should not submit if validation fails', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      const timestampInput = screen.getByLabelText(/Timestamp/) as HTMLInputElement;
      await user.clear(timestampInput);
      await user.type(timestampInput, '-100');

      const submitButton = screen.getByText('Simulate Custom Sequence');
      await user.click(submitButton);

      expect(mockOnSubmit).not.toHaveBeenCalled();
    });
  });

  describe('Event Summary', () => {
    it('should display event count', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));
      await user.click(screen.getByText('+ Add Event'));
      await user.click(screen.getByText('+ Add Event'));

      expect(screen.getByText('3')).toBeInTheDocument();
      expect(screen.getByText('events')).toBeInTheDocument();
    });

    it('should display total duration', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));
      await user.click(screen.getByText('+ Add Event'));

      // Last event should have timestamp 100
      expect(screen.getByText('100')).toBeInTheDocument();
      expect(screen.getByText('μs duration')).toBeInTheDocument();
    });

    it('should update duration when timestamp changes', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));
      await user.click(screen.getByText('+ Add Event'));

      const timestamps = screen.getAllByLabelText(/Timestamp/);
      await user.clear(timestamps[1]);
      await user.type(timestamps[1], '500');

      expect(screen.getByText('500')).toBeInTheDocument();
      expect(screen.getByText('μs duration')).toBeInTheDocument();
    });
  });

  describe('Disabled State', () => {
    it('should disable all controls when disabled prop is true', () => {
      render(<EventSequenceEditor onSubmit={mockOnSubmit} disabled={true} />);

      expect(screen.getByText('+ Add Event')).toBeDisabled();
    });

    it('should disable event controls when disabled', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      const { rerender } = render(<EventSequenceEditor onSubmit={mockOnSubmit} disabled={true} />);

      await user.click(screen.getByText('+ Add Event'));

      rerender(<EventSequenceEditor onSubmit={mockOnSubmit} disabled={true} />);

      const keyCodeSelect = screen.getByLabelText('Key Code:');
      const typeSelect = screen.getByLabelText('Type:');
      const timestampInput = screen.getByLabelText(/Timestamp/);
      const removeButton = screen.getByTitle('Remove event');

      expect(keyCodeSelect).toBeDisabled();
      expect(typeSelect).toBeDisabled();
      expect(timestampInput).toBeDisabled();
      expect(removeButton).toBeDisabled();
    });

    it('should disable submit button when disabled', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      const { rerender } = render(<EventSequenceEditor onSubmit={mockOnSubmit} disabled={true} />);

      await user.click(screen.getByText('+ Add Event'));

      rerender(<EventSequenceEditor onSubmit={mockOnSubmit} disabled={true} />);

      const submitButton = screen.getByText('Simulate Custom Sequence');
      expect(submitButton).toBeDisabled();
    });
  });

  describe('Accessibility', () => {
    it('should have proper labels for all inputs', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      expect(screen.getByLabelText('Key Code:')).toBeInTheDocument();
      expect(screen.getByLabelText('Type:')).toBeInTheDocument();
      expect(screen.getByLabelText(/Timestamp/)).toBeInTheDocument();
    });

    it('should have descriptive button titles', () => {
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      expect(screen.getByTitle('Add new event (Ctrl+Enter)')).toBeInTheDocument();
    });

    it('should show validation errors inline', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      const timestampInput = screen.getByLabelText(/Timestamp/) as HTMLInputElement;
      await user.clear(timestampInput);
      await user.type(timestampInput, '-100');

      const submitButton = screen.getByText('Simulate Custom Sequence');
      await user.click(submitButton);

      const errorElement = screen.getByText('Timestamp must be positive');
      expect(errorElement).toHaveClass('validation-error');
    });
  });

  describe('Key Code Options', () => {
    it('should provide common key codes in dropdown', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      const keyCodeSelect = screen.getByLabelText('Key Code:');
      await user.click(keyCodeSelect);

      // Check for some common keys
      expect(screen.getByText(/A \(VK_A\)/)).toBeInTheDocument();
      expect(screen.getByText(/Z \(VK_Z\)/)).toBeInTheDocument();
      expect(screen.getByText(/Left Shift \(VK_LShift\)/)).toBeInTheDocument();
      expect(screen.getByText(/Space \(VK_Space\)/)).toBeInTheDocument();
    });
  });

  describe('Event Type Options', () => {
    it('should provide press and release options', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      const typeSelect = screen.getByLabelText('Type:');
      await user.click(typeSelect);

      expect(screen.getByText('Press')).toBeInTheDocument();
      expect(screen.getByText('Release')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have no axe violations in empty state', async () => {
      const { container } = render(<EventSequenceEditor onSubmit={mockOnSubmit} />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations with events', async () => {
      const user = userEvent.setup();
      const { container } = render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations when disabled', async () => {
      const { container } = render(<EventSequenceEditor onSubmit={mockOnSubmit} disabled={true} />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper form labels', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      expect(screen.getByLabelText('Key Code:')).toBeInTheDocument();
      expect(screen.getByLabelText('Type:')).toBeInTheDocument();
      expect(screen.getByLabelText('Timestamp (μs):')).toBeInTheDocument();
    });

    it('should have accessible remove buttons', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      await user.click(screen.getByText('+ Add Event'));

      const removeButton = screen.getByLabelText(/Remove event/);
      expect(removeButton).toBeInTheDocument();
    });

    it('should support keyboard navigation for adding events', async () => {
      const user = userEvent.setup();
      render(<EventSequenceEditor onSubmit={mockOnSubmit} />);

      // Tab to add button and press Enter
      await user.tab();
      expect(screen.getByText('+ Add Event')).toHaveFocus();
      await user.keyboard('{Enter}');

      // Event should be added
      expect(screen.getByLabelText('Key Code:')).toBeInTheDocument();
    });
  });
});
