/**
 * Unit tests for ErrorToast component
 */

import { render, screen, act, waitFor, renderHook } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi, beforeEach, afterEach, describe, it, expect } from 'vitest';
import { ErrorToast } from './ErrorToast';
import { useConfigBuilderStore } from '@/store/configBuilderStore';

// Mock timers for auto-dismiss testing
vi.useFakeTimers();

describe('ErrorToast', () => {
  beforeEach(() => {
    // Reset store before each test
    const { result } = renderHook(() => useConfigBuilderStore());
    act(() => {
      result.current.resetConfig();
    });
    vi.clearAllTimers();
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  describe('Rendering', () => {
    it('should not render when there is no error', () => {
      render(<ErrorToast />);
      expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    });

    it('should render when there is an error', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('Test error message');
      });

      render(<ErrorToast />);

      expect(screen.getByRole('alert')).toBeInTheDocument();
      expect(screen.getByText('Test error message')).toBeInTheDocument();
    });

    it('should have correct ARIA attributes', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('Test error');
      });

      render(<ErrorToast />);

      const alert = screen.getByRole('alert');
      expect(alert).toHaveAttribute('aria-live', 'assertive');
    });

    it('should display error icon', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('Test error');
      });

      render(<ErrorToast />);

      expect(screen.getByText('⚠️')).toBeInTheDocument();
    });
  });

  describe('Dismissal', () => {
    it('should dismiss error when close button is clicked', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('Test error');
      });

      render(<ErrorToast />);

      const dismissButton = screen.getByLabelText('Dismiss error');

      act(() => {
        dismissButton.click();
      });

      expect(result.current.lastError).toBeNull();
    });

    it('should have accessible dismiss button', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('Test error');
      });

      render(<ErrorToast />);

      const dismissButton = screen.getByLabelText('Dismiss error');
      expect(dismissButton).toBeInTheDocument();
      expect(dismissButton.tagName).toBe('BUTTON');
    });
  });

  describe('Auto-dismiss', () => {
    it('should auto-dismiss after 5 seconds', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('Test error');
      });

      render(<ErrorToast />);

      expect(screen.getByText('Test error')).toBeInTheDocument();

      // Fast-forward time by 5 seconds
      act(() => {
        vi.advanceTimersByTime(5000);
      });

      expect(result.current.lastError).toBeNull();
    });

    it('should cancel auto-dismiss timer when error is manually dismissed', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('Test error');
      });

      render(<ErrorToast />);

      const dismissButton = screen.getByLabelText('Dismiss error');

      act(() => {
        dismissButton.click();
      });

      // Advance time - should not cause any issues since timer was cleared
      act(() => {
        vi.advanceTimersByTime(5000);
      });

      expect(result.current.lastError).toBeNull();
    });

    it('should reset timer when error changes', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('First error');
      });

      const { rerender } = render(<ErrorToast />);

      // Advance time by 3 seconds
      act(() => {
        vi.advanceTimersByTime(3000);
      });

      // Change error - this should reset the timer
      act(() => {
        result.current.setError('Second error');
      });

      rerender(<ErrorToast />);

      // Advance by 3 more seconds (total 6, but timer was reset at 3)
      act(() => {
        vi.advanceTimersByTime(3000);
      });

      // Error should still be present (only 3 seconds since reset)
      expect(result.current.lastError).toBe('Second error');

      // Advance by 2 more seconds to complete the 5 second timer
      act(() => {
        vi.advanceTimersByTime(2000);
      });

      // Now error should be cleared
      expect(result.current.lastError).toBeNull();
    });
  });

  describe('Multiple errors', () => {
    it('should update displayed error when error changes', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('First error');
      });

      const { rerender } = render(<ErrorToast />);

      expect(screen.getByText('First error')).toBeInTheDocument();

      act(() => {
        result.current.setError('Second error');
      });

      rerender(<ErrorToast />);

      expect(screen.queryByText('First error')).not.toBeInTheDocument();
      expect(screen.getByText('Second error')).toBeInTheDocument();
    });
  });

  describe('Cleanup', () => {
    it('should clean up timer on unmount', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('Test error');
      });

      const { unmount } = render(<ErrorToast />);

      unmount();

      // Advance time - should not cause any issues
      act(() => {
        vi.advanceTimersByTime(5000);
      });

      // Error should still be in store (timer was cleaned up on unmount)
      expect(result.current.lastError).toBe('Test error');
    });
  });
});
