/**
 * Unit tests for EventInjectionForm component
 *
 * Tests cover:
 * - Rendering form elements
 * - Input validation (non-empty key code)
 * - Form submission
 * - Event type selection
 * - Disabled state
 * - Accessibility (ARIA labels, error messages)
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  EventInjectionForm,
  type EventInjectionFormProps,
} from './EventInjectionForm';

/**
 * Default props for testing
 */
const defaultProps: EventInjectionFormProps = {
  onInjectEvent: vi.fn(),
  disabled: false,
};

/**
 * Helper to render component with custom props
 */
function renderForm(props: Partial<EventInjectionFormProps> = {}) {
  return render(<EventInjectionForm {...defaultProps} {...props} />);
}

describe('EventInjectionForm', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should render key code input field', () => {
      renderForm();
      expect(screen.getByLabelText(/key code/i)).toBeInTheDocument();
    });

    it('should render event type selector', () => {
      renderForm();
      expect(screen.getByLabelText(/event type/i)).toBeInTheDocument();
    });

    it('should render inject button', () => {
      renderForm();
      expect(
        screen.getByRole('button', { name: /inject/i })
      ).toBeInTheDocument();
    });

    it('should have press as default event type', () => {
      renderForm();
      const select = screen.getByLabelText(/event type/i) as HTMLSelectElement;
      expect(select.value).toBe('press');
    });

    it('should have placeholder text for key code input', () => {
      renderForm();
      const input = screen.getByLabelText(/key code/i);
      expect(input).toHaveAttribute('placeholder');
    });
  });

  describe('Input Validation', () => {
    it('should disable inject button when key code is empty', () => {
      renderForm();
      const button = screen.getByRole('button', { name: /inject/i });
      expect(button).toBeDisabled();
    });

    it('should enable inject button when key code is provided', async () => {
      const user = userEvent.setup();
      renderForm();
      const input = screen.getByLabelText(/key code/i);
      const button = screen.getByRole('button', { name: /inject/i });

      await user.type(input, 'A');

      expect(button).toBeEnabled();
    });

    it('should prevent submission with empty key code via disabled button', () => {
      renderForm();
      const button = screen.getByRole('button', { name: /inject/i });
      // Button should be disabled when input is empty
      expect(button).toBeDisabled();
    });

    it('should trim whitespace from key code', async () => {
      const user = userEvent.setup();
      const onInjectEvent = vi.fn();
      renderForm({ onInjectEvent });

      const input = screen.getByLabelText(/key code/i);
      await user.type(input, '  Space  ');

      const button = screen.getByRole('button', { name: /inject/i });
      await user.click(button);

      expect(onInjectEvent).toHaveBeenCalledWith({
        keyCode: 'Space',
        eventType: 'press',
      });
    });

    it('should reject whitespace-only key code', async () => {
      const user = userEvent.setup();
      const onInjectEvent = vi.fn();
      renderForm({ onInjectEvent });

      const input = screen.getByLabelText(/key code/i);
      await user.type(input, '   ');

      const button = screen.getByRole('button', { name: /inject/i });
      // Button should be disabled for whitespace-only input
      expect(button).toBeDisabled();
    });
  });

  describe('Form Submission', () => {
    it('should call onInjectEvent with correct data on submit', async () => {
      const user = userEvent.setup();
      const onInjectEvent = vi.fn();
      renderForm({ onInjectEvent });

      const input = screen.getByLabelText(/key code/i);
      await user.type(input, 'Enter');

      const button = screen.getByRole('button', { name: /inject/i });
      await user.click(button);

      expect(onInjectEvent).toHaveBeenCalledTimes(1);
      expect(onInjectEvent).toHaveBeenCalledWith({
        keyCode: 'Enter',
        eventType: 'press',
      });
    });

    it('should submit with release event type when selected', async () => {
      const user = userEvent.setup();
      const onInjectEvent = vi.fn();
      renderForm({ onInjectEvent });

      const input = screen.getByLabelText(/key code/i);
      const select = screen.getByLabelText(/event type/i);
      await user.type(input, 'Escape');
      await user.selectOptions(select, 'release');

      const button = screen.getByRole('button', { name: /inject/i });
      await user.click(button);

      expect(onInjectEvent).toHaveBeenCalledWith({
        keyCode: 'Escape',
        eventType: 'release',
      });
    });

    it('should clear form after successful submission', async () => {
      const user = userEvent.setup();
      const onInjectEvent = vi.fn();
      renderForm({ onInjectEvent });

      const input = screen.getByLabelText(/key code/i) as HTMLInputElement;
      await user.type(input, 'A');

      const button = screen.getByRole('button', { name: /inject/i });
      await user.click(button);

      expect(input.value).toBe('');
    });

    it('should not clear event type after submission', async () => {
      const user = userEvent.setup();
      const onInjectEvent = vi.fn();
      renderForm({ onInjectEvent });

      const input = screen.getByLabelText(/key code/i);
      const select = screen.getByLabelText(/event type/i) as HTMLSelectElement;
      await user.type(input, 'A');
      await user.selectOptions(select, 'release');

      const button = screen.getByRole('button', { name: /inject/i });
      await user.click(button);

      // Event type should remain as 'release'
      expect(select.value).toBe('release');
    });

    it('should handle form submission via Enter key', async () => {
      const user = userEvent.setup();
      const onInjectEvent = vi.fn();
      renderForm({ onInjectEvent });

      const input = screen.getByLabelText(/key code/i);
      await user.type(input, 'Space{Enter}');

      expect(onInjectEvent).toHaveBeenCalledWith({
        keyCode: 'Space',
        eventType: 'press',
      });
    });

    it('should handle multiple submissions', async () => {
      const user = userEvent.setup();
      const onInjectEvent = vi.fn();
      renderForm({ onInjectEvent });

      const input = screen.getByLabelText(/key code/i);
      const button = screen.getByRole('button', { name: /inject/i });

      // First submission
      await user.type(input, 'A');
      await user.click(button);

      // Second submission
      await user.type(input, 'B');
      await user.click(button);

      expect(onInjectEvent).toHaveBeenCalledTimes(2);
      expect(onInjectEvent).toHaveBeenNthCalledWith(1, {
        keyCode: 'A',
        eventType: 'press',
      });
      expect(onInjectEvent).toHaveBeenNthCalledWith(2, {
        keyCode: 'B',
        eventType: 'press',
      });
    });
  });

  describe('Event Type Selection', () => {
    it('should change event type from press to release', async () => {
      const user = userEvent.setup();
      renderForm();

      const select = screen.getByLabelText(/event type/i) as HTMLSelectElement;
      expect(select.value).toBe('press');

      await user.selectOptions(select, 'release');
      expect(select.value).toBe('release');
    });

    it('should have both press and release options', () => {
      renderForm();
      const select = screen.getByLabelText(/event type/i);
      const options = Array.from(select.querySelectorAll('option')).map(
        (opt) => (opt as HTMLOptionElement).value
      );

      expect(options).toEqual(['press', 'release']);
    });
  });

  describe('Disabled State', () => {
    it('should disable all inputs when disabled prop is true', () => {
      renderForm({ disabled: true });

      expect(screen.getByLabelText(/key code/i)).toBeDisabled();
      expect(screen.getByLabelText(/event type/i)).toBeDisabled();
      expect(screen.getByRole('button', { name: /inject/i })).toBeDisabled();
    });

    it('should enable all inputs when disabled prop is false', () => {
      renderForm({ disabled: false });

      expect(screen.getByLabelText(/key code/i)).toBeEnabled();
      expect(screen.getByLabelText(/event type/i)).toBeEnabled();
      // Button is disabled because key code is empty, not because of disabled prop
    });

    it('should not call onInjectEvent when disabled', async () => {
      const user = userEvent.setup();
      const onInjectEvent = vi.fn();
      renderForm({ disabled: true, onInjectEvent });

      const input = screen.getByLabelText(/key code/i);
      // Try to type (should be blocked)
      await user.type(input, 'A');

      const button = screen.getByRole('button', { name: /inject/i });
      await user.click(button);

      expect(onInjectEvent).not.toHaveBeenCalled();
    });
  });

  describe('Accessibility', () => {
    it('should have proper label for key code input', () => {
      renderForm();
      const input = screen.getByLabelText('Key Code');
      expect(input).toBeInTheDocument();
    });

    it('should have proper label for event type select', () => {
      renderForm();
      const select = screen.getByLabelText('Event Type');
      expect(select).toBeInTheDocument();
    });

    it('should have proper ARIA label for form', () => {
      renderForm();
      expect(
        screen.getByRole('form', { name: /manual event injection/i })
      ).toBeInTheDocument();
    });

    it('should have ARIA label for inject button', () => {
      renderForm();
      expect(
        screen.getByRole('button', { name: 'Inject keyboard event' })
      ).toBeInTheDocument();
    });

    it('should start with aria-invalid set to false', () => {
      renderForm();
      const input = screen.getByLabelText(/key code/i);
      expect(input).toHaveAttribute('aria-invalid', 'false');
    });

    it('should not show validation error initially', () => {
      renderForm();
      expect(
        screen.queryByText(/key code cannot be empty/i)
      ).not.toBeInTheDocument();
    });
  });

  describe('Custom className', () => {
    it('should apply custom className', () => {
      const { container } = renderForm({ className: 'custom-class' });
      expect(container.firstChild).toHaveClass('custom-class');
    });

    it('should work without custom className', () => {
      const { container } = renderForm();
      expect(container.firstChild).toHaveClass('flex');
    });
  });

  describe('Edge Cases', () => {
    it('should handle special key codes like Enter, Space, Tab', async () => {
      const user = userEvent.setup();
      const onInjectEvent = vi.fn();
      renderForm({ onInjectEvent });

      const specialKeys = ['Enter', 'Space', 'Tab', 'Escape'];

      for (const key of specialKeys) {
        const input = screen.getByLabelText(/key code/i);
        await user.clear(input);
        await user.type(input, key);

        const button = screen.getByRole('button', { name: /inject/i });
        await user.click(button);

        expect(onInjectEvent).toHaveBeenCalledWith({
          keyCode: key,
          eventType: 'press',
        });
      }

      expect(onInjectEvent).toHaveBeenCalledTimes(specialKeys.length);
    });

    it('should handle numeric key codes', async () => {
      const user = userEvent.setup();
      const onInjectEvent = vi.fn();
      renderForm({ onInjectEvent });

      const input = screen.getByLabelText(/key code/i);
      await user.type(input, '0');

      const button = screen.getByRole('button', { name: /inject/i });
      await user.click(button);

      expect(onInjectEvent).toHaveBeenCalledWith({
        keyCode: '0',
        eventType: 'press',
      });
    });

    it('should handle long key codes', async () => {
      const user = userEvent.setup();
      const onInjectEvent = vi.fn();
      renderForm({ onInjectEvent });

      const longKeyCode = 'VeryLongKeyCodeName';
      const input = screen.getByLabelText(/key code/i);
      await user.type(input, longKeyCode);

      const button = screen.getByRole('button', { name: /inject/i });
      await user.click(button);

      expect(onInjectEvent).toHaveBeenCalledWith({
        keyCode: longKeyCode,
        eventType: 'press',
      });
    });
  });
});
