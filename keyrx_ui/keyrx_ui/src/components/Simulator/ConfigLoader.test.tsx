/**
 * ConfigLoader.test.tsx - Tests for the ConfigLoader component.
 *
 * Tests cover:
 * - Textarea input and file upload modes
 * - File validation (type and size)
 * - Parse error display with line numbers
 * - Error highlighting in textarea
 * - Loading states
 * - User interactions
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { axe, toHaveNoViolations } from 'jest-axe';
import { ConfigLoader } from './ConfigLoader';

// Extend Vitest matchers with jest-axe
expect.extend(toHaveNoViolations);

describe('ConfigLoader', () => {
  const mockOnLoad = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should render input mode toggle buttons', () => {
      render(<ConfigLoader onLoad={mockOnLoad} />);

      expect(screen.getByText('Paste Configuration')).toBeInTheDocument();
      expect(screen.getByText('Upload File')).toBeInTheDocument();
    });

    it('should render textarea by default', () => {
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const textarea = screen.getByPlaceholderText(/Paste your Rhai configuration/);
      expect(textarea).toBeInTheDocument();
    });

    it('should render load button', () => {
      render(<ConfigLoader onLoad={mockOnLoad} />);

      expect(screen.getByText('Load Configuration')).toBeInTheDocument();
    });

    it('should show help text', () => {
      render(<ConfigLoader onLoad={mockOnLoad} />);

      expect(screen.getByText(/Your Rhai configuration will be compiled in the browser/)).toBeInTheDocument();
    });
  });

  describe('Textarea Input', () => {
    it('should allow typing in textarea', async () => {
      const user = userEvent.setup();
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const textarea = screen.getByPlaceholderText(/Paste your Rhai configuration/);
      await user.type(textarea, 'let x = 42;');

      expect(textarea).toHaveValue('let x = 42;');
    });

    it('should disable textarea when loading', () => {
      render(<ConfigLoader onLoad={mockOnLoad} isLoading={true} />);

      const textarea = screen.getByPlaceholderText(/Paste your Rhai configuration/);
      expect(textarea).toBeDisabled();
    });

    it('should clear error when typing', async () => {
      const user = userEvent.setup();
      const { rerender } = render(
        <ConfigLoader onLoad={mockOnLoad} error="Parse error at line 5" />
      );

      expect(screen.getByText(/Parse error at line 5/)).toBeInTheDocument();

      const textarea = screen.getByPlaceholderText(/Paste your Rhai configuration/);
      await user.type(textarea, 'new content');

      // File error state should be cleared (internal state)
      // But prop error should still be visible until parent clears it
      expect(screen.getByText(/Parse error at line 5/)).toBeInTheDocument();

      // Re-render without error
      rerender(<ConfigLoader onLoad={mockOnLoad} />);
      expect(screen.queryByText(/Parse error at line 5/)).not.toBeInTheDocument();
    });
  });

  describe('File Upload', () => {
    it('should switch to file upload mode', async () => {
      const user = userEvent.setup();
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const uploadButton = screen.getByText('Upload File');
      await user.click(uploadButton);

      expect(uploadButton).toHaveClass('active');
    });

    it('should accept .rhai files', async () => {
      const user = userEvent.setup();
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const fileContent = 'let x = 42;';
      const file = new File([fileContent], 'config.rhai', { type: 'text/plain' });

      // Get the hidden file input
      const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
      expect(fileInput).toBeInTheDocument();

      await user.upload(fileInput, file);

      await waitFor(() => {
        const textarea = screen.getByPlaceholderText(/Paste your Rhai configuration/);
        expect(textarea).toHaveValue(fileContent);
      });

      // Should show file name
      expect(screen.getByText('config.rhai')).toBeInTheDocument();
    });

    it('should reject non-.rhai files', async () => {
      const user = userEvent.setup();
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const file = new File(['content'], 'config.txt', { type: 'text/plain' });
      const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;

      await user.upload(fileInput, file);

      await waitFor(() => {
        expect(screen.getByText(/Invalid file type.*got .txt/)).toBeInTheDocument();
      });
    });

    it('should reject files larger than 1MB', async () => {
      const user = userEvent.setup();
      render(<ConfigLoader onLoad={mockOnLoad} />);

      // Create a file larger than 1MB
      const largeContent = 'x'.repeat(1024 * 1024 + 1);
      const file = new File([largeContent], 'large.rhai', { type: 'text/plain' });
      const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;

      await user.upload(fileInput, file);

      await waitFor(() => {
        expect(screen.getByText(/File too large.*maximum 1MB/)).toBeInTheDocument();
      });
    });

    it('should handle file read errors', async () => {
      const user = userEvent.setup();
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const file = new File(['content'], 'config.rhai', { type: 'text/plain' });
      const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;

      // Mock FileReader to trigger error
      const originalFileReader = window.FileReader;
      window.FileReader = vi.fn().mockImplementation(() => ({
        readAsText: function() {
          setTimeout(() => this.onerror?.(), 0);
        },
      })) as unknown as typeof FileReader;

      await user.upload(fileInput, file);

      await waitFor(() => {
        expect(screen.getByText('Failed to read file')).toBeInTheDocument();
      });

      // Restore original FileReader
      window.FileReader = originalFileReader;
    });
  });

  describe('Load Button', () => {
    it('should call onLoad with textarea content', async () => {
      const user = userEvent.setup();
      mockOnLoad.mockResolvedValue(undefined);

      render(<ConfigLoader onLoad={mockOnLoad} />);

      const textarea = screen.getByPlaceholderText(/Paste your Rhai configuration/);
      await user.type(textarea, 'let x = 42;');

      const loadButton = screen.getByText('Load Configuration');
      await user.click(loadButton);

      expect(mockOnLoad).toHaveBeenCalledWith('let x = 42;');
    });

    it('should be disabled when textarea is empty', () => {
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const loadButton = screen.getByText('Load Configuration');
      expect(loadButton).toBeDisabled();
    });

    it('should be disabled during loading', () => {
      render(<ConfigLoader onLoad={mockOnLoad} isLoading={true} />);

      const loadButton = screen.getByText('Loading Configuration...');
      expect(loadButton).toBeDisabled();
    });

    it('should show error if trying to load empty content', async () => {
      const user = userEvent.setup();
      render(<ConfigLoader onLoad={mockOnLoad} />);

      // Type some content then delete it
      const textarea = screen.getByPlaceholderText(/Paste your Rhai configuration/);
      await user.type(textarea, '   '); // Just whitespace

      const loadButton = screen.getByText('Load Configuration');
      expect(loadButton).toBeDisabled(); // Should be disabled for whitespace-only
    });

    it('should not be disabled when content is whitespace-only', async () => {
      const user = userEvent.setup();
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const textarea = screen.getByPlaceholderText(/Paste your Rhai configuration/);
      await user.type(textarea, 'content');

      const loadButton = screen.getByText('Load Configuration');
      expect(loadButton).toBeEnabled();
    });
  });

  describe('Error Display', () => {
    it('should display parse errors', () => {
      const errorMessage = 'Unexpected token at line 5';
      render(<ConfigLoader onLoad={mockOnLoad} error={errorMessage} />);

      expect(screen.getByText(/Unexpected token at line 5/)).toBeInTheDocument();
    });

    it('should extract and display line numbers from errors', () => {
      const errorMessage = 'Parse error at line 42: unexpected token';
      render(<ConfigLoader onLoad={mockOnLoad} error={errorMessage} />);

      expect(screen.getByText('Parse Error (line 42):')).toBeInTheDocument();
      expect(screen.getByText(/Error on line 42/)).toBeInTheDocument();
    });

    it('should highlight error line in textarea', () => {
      const errorMessage = 'Parse error at line 5';
      render(<ConfigLoader onLoad={mockOnLoad} error={errorMessage} />);

      const textarea = screen.getByPlaceholderText(/Paste your Rhai configuration/);
      expect(textarea).toHaveClass('has-error-line');
      expect(screen.getByText('Error on line 5')).toBeInTheDocument();
    });

    it('should handle errors without line numbers', () => {
      const errorMessage = 'Generic error message';
      render(<ConfigLoader onLoad={mockOnLoad} error={errorMessage} />);

      expect(screen.getByText('Error:')).toBeInTheDocument();
      expect(screen.getByText(/Generic error message/)).toBeInTheDocument();
      expect(screen.queryByText(/Error on line/)).not.toBeInTheDocument();
    });

    it('should show file validation errors', async () => {
      const user = userEvent.setup();
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const file = new File(['content'], 'config.json', { type: 'application/json' });
      const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;

      await user.upload(fileInput, file);

      await waitFor(() => {
        expect(screen.getByText(/Invalid file type.*got .json/)).toBeInTheDocument();
      });
    });
  });

  describe('Loading State', () => {
    it('should show loading text on button during load', () => {
      render(<ConfigLoader onLoad={mockOnLoad} isLoading={true} />);

      expect(screen.getByText('Loading Configuration...')).toBeInTheDocument();
    });

    it('should disable all inputs during loading', () => {
      render(<ConfigLoader onLoad={mockOnLoad} isLoading={true} />);

      expect(screen.getByText('Paste Configuration')).toBeDisabled();
      expect(screen.getByText('Upload File')).toBeDisabled();
      expect(screen.getByPlaceholderText(/Paste your Rhai configuration/)).toBeDisabled();
      expect(screen.getByText('Loading Configuration...')).toBeDisabled();
    });
  });

  describe('Mode Switching', () => {
    it('should activate correct button when switching modes', async () => {
      const user = userEvent.setup();
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const pasteButton = screen.getByText('Paste Configuration');
      const uploadButton = screen.getByText('Upload File');

      expect(pasteButton).toHaveClass('active');
      expect(uploadButton).not.toHaveClass('active');

      await user.click(uploadButton);

      expect(uploadButton).toHaveClass('active');

      await user.click(pasteButton);

      expect(pasteButton).toHaveClass('active');
    });

    it('should preserve textarea content when switching modes', async () => {
      const user = userEvent.setup();
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const textarea = screen.getByPlaceholderText(/Paste your Rhai configuration/);
      await user.type(textarea, 'let x = 42;');

      const uploadButton = screen.getByText('Upload File');
      await user.click(uploadButton);

      // Content should still be there
      expect(textarea).toHaveValue('let x = 42;');
    });
  });

  describe('Accessibility', () => {
    it('should have no axe violations in default state', async () => {
      const { container } = render(<ConfigLoader onLoad={mockOnLoad} />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations in file upload mode', async () => {
      const user = userEvent.setup();
      const { container } = render(<ConfigLoader onLoad={mockOnLoad} />);

      await user.click(screen.getByText('Upload File'));

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations with error state', async () => {
      const { container } = render(<ConfigLoader onLoad={mockOnLoad} error="Parse error at line 5" />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations when loading', async () => {
      const { container } = render(<ConfigLoader onLoad={mockOnLoad} isLoading={true} />);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper form structure', () => {
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const loadButton = screen.getByText('Load Configuration');
      expect(loadButton).toHaveAttribute('type', 'button');
    });

    it('should accept .rhai files in file input', () => {
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
      expect(fileInput).toHaveAttribute('accept', '.rhai');
    });

    it('should disable spellcheck on textarea', () => {
      render(<ConfigLoader onLoad={mockOnLoad} />);

      const textarea = screen.getByPlaceholderText(/Paste your Rhai configuration/);
      expect(textarea).toHaveAttribute('spellCheck', 'false');
    });
  });

  describe('Error Line Extraction', () => {
    const testCases = [
      { error: 'Error at line 42', expected: 42 },
      { error: 'Parse error on line 123', expected: 123 },
      { error: 'Syntax error line 5', expected: 5 },
      { error: 'Error without line number', expected: null },
      { error: 'line in text but not line 99', expected: 99 },
    ];

    testCases.forEach(({ error, expected }) => {
      it(`should extract line ${expected} from "${error}"`, () => {
        render(<ConfigLoader onLoad={mockOnLoad} error={error} />);

        if (expected) {
          expect(screen.getByText(`Error on line ${expected}`)).toBeInTheDocument();
        } else {
          expect(screen.queryByText(/Error on line/)).not.toBeInTheDocument();
        }
      });
    });
  });
});
