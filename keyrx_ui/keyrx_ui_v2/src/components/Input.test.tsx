import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Input } from './Input';

describe('Input', () => {
  describe('Rendering', () => {
    it('renders with required props', () => {
      const handleChange = vi.fn();
      render(
        <Input value="" onChange={handleChange} aria-label="Test input" />
      );

      const input = screen.getByLabelText('Test input');
      expect(input).toBeInTheDocument();
    });

    it('renders with text type by default', () => {
      const handleChange = vi.fn();
      render(
        <Input value="" onChange={handleChange} aria-label="Test input" />
      );

      const input = screen.getByLabelText('Test input');
      expect(input).toHaveAttribute('type', 'text');
    });

    it('renders with number type when specified', () => {
      const handleChange = vi.fn();
      render(
        <Input
          type="number"
          value=""
          onChange={handleChange}
          aria-label="Test input"
        />
      );

      const input = screen.getByLabelText('Test input');
      expect(input).toHaveAttribute('type', 'number');
    });

    it('renders with placeholder', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Test input"
          placeholder="Enter text"
        />
      );

      const input = screen.getByPlaceholderText('Enter text');
      expect(input).toBeInTheDocument();
    });
  });

  describe('Value and onChange', () => {
    it('displays the current value', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value="test value"
          onChange={handleChange}
          aria-label="Test input"
        />
      );

      const input = screen.getByLabelText('Test input') as HTMLInputElement;
      expect(input.value).toBe('test value');
    });

    it('calls onChange when value changes', () => {
      const handleChange = vi.fn();
      render(
        <Input value="" onChange={handleChange} aria-label="Test input" />
      );

      const input = screen.getByLabelText('Test input');
      fireEvent.change(input, { target: { value: 'new value' } });

      expect(handleChange).toHaveBeenCalledWith('new value');
    });
  });

  describe('Error state', () => {
    it('displays error message when error prop is provided', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Test input"
          error="This field is required"
          id="test-input"
        />
      );

      const errorMessage = screen.getByText('This field is required');
      expect(errorMessage).toBeInTheDocument();
      expect(errorMessage).toHaveClass('text-red-500');
    });

    it('sets aria-invalid when error is present', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Test input"
          error="Error message"
        />
      );

      const input = screen.getByLabelText('Test input');
      expect(input).toHaveAttribute('aria-invalid', 'true');
    });

    it('applies error border styling', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Test input"
          error="Error message"
        />
      );

      const input = screen.getByLabelText('Test input');
      expect(input).toHaveClass('border-red-500');
    });

    it('does not display error message when no error', () => {
      const handleChange = vi.fn();
      render(
        <Input value="" onChange={handleChange} aria-label="Test input" />
      );

      const errorMessage = screen.queryByRole('alert');
      expect(errorMessage).not.toBeInTheDocument();
    });
  });

  describe('Disabled state', () => {
    it('disables input when disabled prop is true', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Test input"
          disabled
        />
      );

      const input = screen.getByLabelText('Test input');
      expect(input).toBeDisabled();
    });

    it('applies disabled styling', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Test input"
          disabled
        />
      );

      const input = screen.getByLabelText('Test input');
      expect(input).toHaveClass('opacity-50');
      expect(input).toHaveClass('cursor-not-allowed');
    });

    it('does not call onChange when disabled', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Test input"
          disabled
        />
      );

      const input = screen.getByLabelText('Test input');
      fireEvent.change(input, { target: { value: 'new value' } });

      // onChange should not be called when disabled
      // (browser prevents change events on disabled inputs)
      expect(handleChange).not.toHaveBeenCalled();
    });
  });

  describe('maxLength and character counter', () => {
    it('enforces maxLength constraint', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Test input"
          maxLength={10}
        />
      );

      const input = screen.getByLabelText('Test input');
      expect(input).toHaveAttribute('maxLength', '10');
    });

    it('displays character counter when maxLength is set', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value="hello"
          onChange={handleChange}
          aria-label="Test input"
          maxLength={10}
        />
      );

      const counter = screen.getByText('5 / 10');
      expect(counter).toBeInTheDocument();
    });

    it('does not display character counter when maxLength is not set', () => {
      const handleChange = vi.fn();
      render(
        <Input value="hello" onChange={handleChange} aria-label="Test input" />
      );

      const counter = screen.queryByText(/\d+ \/ \d+/);
      expect(counter).not.toBeInTheDocument();
    });

    it('highlights counter in amber when near limit (90%)', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value="123456789"
          onChange={handleChange}
          aria-label="Test input"
          maxLength={10}
        />
      );

      const counter = screen.getByText('9 / 10');
      expect(counter).toHaveClass('text-amber-500');
    });

    it('displays counter in normal color when below 90%', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value="12345"
          onChange={handleChange}
          aria-label="Test input"
          maxLength={10}
        />
      );

      const counter = screen.getByText('5 / 10');
      expect(counter).toHaveClass('text-slate-400');
    });
  });

  describe('Focus state', () => {
    it('applies focus styling when focused', () => {
      const handleChange = vi.fn();
      render(
        <Input value="" onChange={handleChange} aria-label="Test input" />
      );

      const input = screen.getByLabelText('Test input');
      fireEvent.focus(input);

      expect(input).toHaveClass('border-primary-500');
    });

    it('removes focus styling when blurred', () => {
      const handleChange = vi.fn();
      render(
        <Input value="" onChange={handleChange} aria-label="Test input" />
      );

      const input = screen.getByLabelText('Test input');
      fireEvent.focus(input);
      fireEvent.blur(input);

      expect(input).not.toHaveClass('border-primary-500');
      expect(input).toHaveClass('border-slate-700');
    });
  });

  describe('Accessibility', () => {
    it('has required aria-label', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Username input"
        />
      );

      const input = screen.getByLabelText('Username input');
      expect(input).toBeInTheDocument();
    });

    it('sets aria-describedby when error is present', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Test input"
          error="Error message"
          id="test-input"
        />
      );

      const input = screen.getByLabelText('Test input');
      expect(input).toHaveAttribute('aria-describedby', 'test-input-error');
    });

    it('error message has role alert and aria-live assertive', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Test input"
          error="Error message"
          id="test-input"
        />
      );

      const errorMessage = screen.getByRole('alert');
      expect(errorMessage).toHaveAttribute('aria-live', 'assertive');
    });
  });

  describe('Custom className', () => {
    it('applies custom className', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Test input"
          className="custom-class"
        />
      );

      const input = screen.getByLabelText('Test input');
      expect(input).toHaveClass('custom-class');
    });

    it('preserves base classes when custom className is provided', () => {
      const handleChange = vi.fn();
      render(
        <Input
          value=""
          onChange={handleChange}
          aria-label="Test input"
          className="custom-class"
        />
      );

      const input = screen.getByLabelText('Test input');
      expect(input).toHaveClass('w-full');
      expect(input).toHaveClass('custom-class');
    });
  });
});
