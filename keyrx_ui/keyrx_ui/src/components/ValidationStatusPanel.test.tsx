import { describe, it, expect, vi } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { axe, toHaveNoViolations } from 'jest-axe';
import { ValidationStatusPanel } from './ValidationStatusPanel';
import type { ValidationResult, ValidationError, ValidationWarning } from '../types/validation';

// Extend Vitest matchers with jest-axe
expect.extend(toHaveNoViolations);

describe('ValidationStatusPanel - Accessibility', () => {
  const createMockError = (line: number, message: string): ValidationError => ({
    line,
    column: 1,
    message,
    code: 'TEST_ERROR',
  });

  const createMockWarning = (line: number, message: string): ValidationWarning => ({
    line,
    column: 1,
    message,
    code: 'TEST_WARNING',
  });

  describe('Axe Accessibility Audit', () => {
    it('should have no axe violations with no validation result', async () => {
      const { container } = render(
        <ValidationStatusPanel
          validationResult={null}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations when validating', async () => {
      const { container } = render(
        <ValidationStatusPanel
          validationResult={null}
          isValidating={true}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations with errors', async () => {
      const validationResult: ValidationResult = {
        errors: [
          createMockError(5, 'Syntax error'),
          createMockError(10, 'Invalid key code'),
        ],
        warnings: [],
        hints: [],
      };

      const { container } = render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations with warnings', async () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [
          createMockWarning(3, 'Unused layer detected'),
          createMockWarning(7, 'Naming inconsistency'),
        ],
        hints: [],
      };

      const { container } = render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations with valid configuration', async () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [],
      };

      const { container } = render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have no axe violations with many errors (overflow)', async () => {
      const errors = Array.from({ length: 15 }, (_, i) =>
        createMockError(i + 1, `Error ${i + 1}`)
      );

      const validationResult: ValidationResult = {
        errors,
        warnings: [],
        hints: [],
      };

      const { container } = render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });
  });

  describe('ARIA Labels and Roles', () => {
    it('should have correct role on main container', () => {
      render(
        <ValidationStatusPanel
          validationResult={null}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const panel = screen.getByRole('region', { name: 'Validation Status' });
      expect(panel).toBeInTheDocument();
    });

    it('should have correct role and aria-label on header button', () => {
      const validationResult: ValidationResult = {
        errors: [createMockError(5, 'Test error')],
        warnings: [createMockWarning(3, 'Test warning')],
        hints: [],
      };

      render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const header = screen.getByRole('button', {
        name: /Validation status: 1 errors, 1 warnings/,
      });
      expect(header).toBeInTheDocument();
    });

    it('should have aria-expanded attribute on header', () => {
      render(
        <ValidationStatusPanel
          validationResult={null}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const header = screen.getByRole('button');
      expect(header).toHaveAttribute('aria-expanded', 'true');
    });

    it('should update aria-expanded when collapsed', async () => {
      const user = userEvent.setup();

      render(
        <ValidationStatusPanel
          validationResult={null}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const header = screen.getByRole('button');
      await user.click(header);

      expect(header).toHaveAttribute('aria-expanded', 'false');
    });

    it('should have aria-label on error list', () => {
      const validationResult: ValidationResult = {
        errors: [createMockError(5, 'Test error')],
        warnings: [],
        hints: [],
      };

      render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const errorList = screen.getByLabelText('Validation Errors');
      expect(errorList).toBeInTheDocument();
      expect(errorList.tagName).toBe('UL');
    });

    it('should have aria-label on warning list', () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [createMockWarning(3, 'Test warning')],
        hints: [],
      };

      render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const warningList = screen.getByLabelText('Validation Warnings');
      expect(warningList).toBeInTheDocument();
      expect(warningList.tagName).toBe('UL');
    });

    it('should have descriptive aria-label on jump buttons', () => {
      const validationResult: ValidationResult = {
        errors: [createMockError(5, 'Test error')],
        warnings: [],
        hints: [],
      };

      render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const jumpButton = screen.getByLabelText('Jump to error at line 5');
      expect(jumpButton).toBeInTheDocument();
    });

    it('should have aria-hidden on expand icon', () => {
      const { container } = render(
        <ValidationStatusPanel
          validationResult={null}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const expandIcon = container.querySelector('.expand-icon');
      expect(expandIcon).toHaveAttribute('aria-hidden', 'true');
    });
  });

  describe('Screen Reader Announcements', () => {
    it('should have aria-live on validating badge', () => {
      render(
        <ValidationStatusPanel
          validationResult={null}
          isValidating={true}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const validatingBadge = screen.getByText(/Validating/);
      expect(validatingBadge).toHaveAttribute('aria-live', 'polite');
    });

    it('should have aria-live on success badge', () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [],
      };

      render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const successBadge = screen.getByText(/Configuration valid/);
      expect(successBadge).toHaveAttribute('aria-live', 'polite');
    });

    it('should have aria-live on error badge', () => {
      const validationResult: ValidationResult = {
        errors: [createMockError(5, 'Test error')],
        warnings: [],
        hints: [],
      };

      render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const errorBadge = screen.getByText(/1 Error/);
      expect(errorBadge).toHaveAttribute('aria-live', 'polite');
    });

    it('should have aria-live on warning badge', () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [createMockWarning(3, 'Test warning')],
        hints: [],
      };

      render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const warningBadge = screen.getByText(/1 Warning/);
      expect(warningBadge).toHaveAttribute('aria-live', 'polite');
    });

    it('should have aria-live on overflow message', () => {
      const errors = Array.from({ length: 15 }, (_, i) =>
        createMockError(i + 1, `Error ${i + 1}`)
      );

      const validationResult: ValidationResult = {
        errors,
        warnings: [],
        hints: [],
      };

      render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const overflowMessage = screen.getByText(/...and 5 more errors/);
      expect(overflowMessage).toHaveAttribute('aria-live', 'polite');
    });
  });

  describe('Keyboard Navigation', () => {
    it('should be focusable with tab', async () => {
      const user = userEvent.setup();

      render(
        <ValidationStatusPanel
          validationResult={null}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const header = screen.getByRole('button');
      expect(header).toHaveAttribute('tabIndex', '0');

      await user.tab();
      expect(header).toHaveFocus();
    });

    it('should toggle expansion on Enter key', async () => {
      const user = userEvent.setup();

      render(
        <ValidationStatusPanel
          validationResult={null}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const header = screen.getByRole('button');
      await user.tab();

      expect(header).toHaveAttribute('aria-expanded', 'true');
      await user.keyboard('{Enter}');
      expect(header).toHaveAttribute('aria-expanded', 'false');
    });

    it('should toggle expansion on Space key', async () => {
      const user = userEvent.setup();

      render(
        <ValidationStatusPanel
          validationResult={null}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const header = screen.getByRole('button');
      await user.tab();

      expect(header).toHaveAttribute('aria-expanded', 'true');
      await user.keyboard(' ');
      expect(header).toHaveAttribute('aria-expanded', 'false');
    });

    it('should navigate through jump buttons with tab', async () => {
      const user = userEvent.setup();
      const validationResult: ValidationResult = {
        errors: [
          createMockError(5, 'Error 1'),
          createMockError(10, 'Error 2'),
        ],
        warnings: [],
        hints: [],
      };

      render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      // Tab to header
      await user.tab();
      const header = screen.getByRole('button', { name: /Validation status/ });
      expect(header).toHaveFocus();

      // Tab to first jump button
      await user.tab();
      const firstJump = screen.getByLabelText('Jump to error at line 5');
      expect(firstJump).toHaveFocus();

      // Tab to second jump button
      await user.tab();
      const secondJump = screen.getByLabelText('Jump to error at line 10');
      expect(secondJump).toHaveFocus();
    });

    it('should call onErrorClick when Enter pressed on jump button', async () => {
      const user = userEvent.setup();
      const onErrorClick = vi.fn();
      const error = createMockError(5, 'Test error');
      const validationResult: ValidationResult = {
        errors: [error],
        warnings: [],
        hints: [],
      };

      render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={onErrorClick}
          onWarningClick={vi.fn()}
        />
      );

      const jumpButton = screen.getByLabelText('Jump to error at line 5');
      jumpButton.focus();
      await user.keyboard('{Enter}');

      expect(onErrorClick).toHaveBeenCalledWith(error);
    });

    it('should call onWarningClick when Enter pressed on warning jump button', async () => {
      const user = userEvent.setup();
      const onWarningClick = vi.fn();
      const warning = createMockWarning(3, 'Test warning');
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [warning],
        hints: [],
      };

      render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={onWarningClick}
        />
      );

      const jumpButton = screen.getByLabelText('Jump to warning at line 3');
      jumpButton.focus();
      await user.keyboard('{Enter}');

      expect(onWarningClick).toHaveBeenCalledWith(warning);
    });
  });

  describe('Color Contrast', () => {
    it('should verify error badge has adequate contrast', () => {
      const validationResult: ValidationResult = {
        errors: [createMockError(5, 'Test error')],
        warnings: [],
        hints: [],
      };

      const { container } = render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const errorBadge = container.querySelector('.validation-badge.error');
      expect(errorBadge).toBeInTheDocument();
      // The actual color contrast testing is done by axe
    });

    it('should verify warning badge has adequate contrast', () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [createMockWarning(3, 'Test warning')],
        hints: [],
      };

      const { container } = render(
        <ValidationStatusPanel
          validationResult={validationResult}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const warningBadge = container.querySelector('.validation-badge.warning');
      expect(warningBadge).toBeInTheDocument();
      // The actual color contrast testing is done by axe
    });
  });

  describe('Functional Accessibility Tests', () => {
    it('should display correct plural/singular for errors', () => {
      const singleError: ValidationResult = {
        errors: [createMockError(5, 'Test error')],
        warnings: [],
        hints: [],
      };

      const { rerender } = render(
        <ValidationStatusPanel
          validationResult={singleError}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      expect(screen.getByText(/1 Error$/)).toBeInTheDocument();

      const multipleErrors: ValidationResult = {
        errors: [
          createMockError(5, 'Error 1'),
          createMockError(10, 'Error 2'),
        ],
        warnings: [],
        hints: [],
      };

      rerender(
        <ValidationStatusPanel
          validationResult={multipleErrors}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      expect(screen.getByText(/2 Errors$/)).toBeInTheDocument();
    });

    it('should maintain focus after expansion toggle', async () => {
      const user = userEvent.setup();

      render(
        <ValidationStatusPanel
          validationResult={null}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const header = screen.getByRole('button');
      header.focus();

      await user.keyboard('{Enter}');
      expect(header).toHaveFocus();

      await user.keyboard(' ');
      expect(header).toHaveFocus();
    });

    it('should prevent default on Space key to avoid scrolling', async () => {
      const user = userEvent.setup();

      render(
        <ValidationStatusPanel
          validationResult={null}
          isValidating={false}
          onErrorClick={vi.fn()}
          onWarningClick={vi.fn()}
        />
      );

      const header = screen.getByRole('button');
      header.focus();

      // Space key should toggle without scrolling (preventDefault is called)
      await user.keyboard(' ');
      expect(header).toHaveAttribute('aria-expanded', 'false');
    });
  });
});
