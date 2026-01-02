import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useAutoSave } from '../../src/hooks/useAutoSave';

describe('useAutoSave', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  describe('Debouncing behavior', () => {
    it('should debounce multiple rapid changes into a single save', async () => {
      const saveFn = vi.fn().mockResolvedValue(undefined);
      const { rerender } = renderHook(
        ({ data }) => useAutoSave(data, { saveFn }),
        { initialProps: { data: 'initial' } }
      );

      // Make rapid changes
      rerender({ data: 'change1' });
      rerender({ data: 'change2' });
      rerender({ data: 'change3' });

      // saveFn should not be called yet (debouncing)
      expect(saveFn).not.toHaveBeenCalled();

      // Fast-forward past debounce delay (500ms default)
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      // Should only save once with the final value
      expect(saveFn).toHaveBeenCalledTimes(1);
      expect(saveFn).toHaveBeenCalledWith('change3');
    });

    it('should respect custom debounce delay', async () => {
      const saveFn = vi.fn().mockResolvedValue(undefined);
      const { rerender } = renderHook(
        ({ data }) => useAutoSave(data, { saveFn, debounceMs: 1000 }),
        { initialProps: { data: 'initial' } }
      );

      rerender({ data: 'changed' });

      // Should not save before custom delay
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });
      expect(saveFn).not.toHaveBeenCalled();

      // Should save after custom delay
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });
      expect(saveFn).toHaveBeenCalledTimes(1);
    });

    it('should reset debounce timer on each change', async () => {
      const saveFn = vi.fn().mockResolvedValue(undefined);
      const { rerender } = renderHook(
        ({ data }) => useAutoSave(data, { saveFn }),
        { initialProps: { data: 'initial' } }
      );

      rerender({ data: 'change1' });

      // Advance 400ms (not enough to trigger)
      await act(async () => {
        await vi.advanceTimersByTimeAsync(400);
      });

      // Another change resets the timer
      rerender({ data: 'change2' });

      // Advance another 400ms (800ms total, but timer was reset)
      await act(async () => {
        await vi.advanceTimersByTimeAsync(400);
      });
      expect(saveFn).not.toHaveBeenCalled();

      // Advance final 100ms to complete debounce
      await act(async () => {
        await vi.advanceTimersByTimeAsync(100);
      });
      expect(saveFn).toHaveBeenCalledWith('change2');
    });
  });

  describe('Retry logic', () => {
    it('should retry failed saves with exponential backoff', async () => {
      let callCount = 0;
      const saveFn = vi.fn().mockImplementation(() => {
        callCount++;
        if (callCount < 3) {
          return Promise.reject(new Error('Network error'));
        }
        return Promise.resolve();
      });

      const { result } = renderHook(() =>
        useAutoSave('data', { saveFn, retryDelayMs: 1000, maxRetries: 3 })
      );

      // Trigger initial save
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      // First attempt failed
      expect(saveFn).toHaveBeenCalledTimes(1);
      expect(result.current.isSaving).toBe(true);

      // First retry after 1000ms (2^0 * 1000)
      await act(async () => {
        await vi.advanceTimersByTimeAsync(1000);
      });
      expect(saveFn).toHaveBeenCalledTimes(2);

      // Second retry after 2000ms (2^1 * 1000)
      await act(async () => {
        await vi.advanceTimersByTimeAsync(2000);
      });

      // Should succeed on third attempt
      expect(saveFn).toHaveBeenCalledTimes(3);
      expect(result.current.isSaving).toBe(false);
      expect(result.current.error).toBeNull();
      expect(result.current.lastSavedAt).toBeInstanceOf(Date);
    });

    it('should not retry validation errors (4xx)', async () => {
      const saveFn = vi.fn().mockRejectedValue(new Error('400 Bad Request'));
      const { result } = renderHook(() =>
        useAutoSave('invalid-data', { saveFn })
      );

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(saveFn).toHaveBeenCalledTimes(1);
      expect(result.current.error).toBeTruthy();
      expect(result.current.error?.message).toContain('400');

      // Should not retry
      await act(async () => {
        await vi.advanceTimersByTimeAsync(10000);
      });
      expect(saveFn).toHaveBeenCalledTimes(1);
    });

    it('should stop retrying after maxRetries', async () => {
      const saveFn = vi.fn().mockRejectedValue(new Error('Server error'));
      const { result } = renderHook(() =>
        useAutoSave('data', { saveFn, retryDelayMs: 100, maxRetries: 2 })
      );

      // Initial attempt
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      // First retry
      await act(async () => {
        await vi.advanceTimersByTimeAsync(100);
      });

      // Second retry
      await act(async () => {
        await vi.advanceTimersByTimeAsync(200);
      });

      expect(saveFn).toHaveBeenCalledTimes(3); // Initial + 2 retries
      expect(result.current.error).toBeTruthy();
      expect(result.current.isSaving).toBe(false);

      // Should not retry again
      await act(async () => {
        await vi.advanceTimersByTimeAsync(10000);
      });
      expect(saveFn).toHaveBeenCalledTimes(3);
    });

    it('should reset retry count after successful save', async () => {
      let callCount = 0;
      const saveFn = vi.fn().mockImplementation(() => {
        callCount++;
        // Fail first attempt, succeed second
        if (callCount === 1) {
          return Promise.reject(new Error('Network error'));
        }
        return Promise.resolve();
      });

      const { result, rerender } = renderHook(
        ({ data }) => useAutoSave(data, { saveFn, retryDelayMs: 100 }),
        { initialProps: { data: 'data1' } }
      );

      // First save fails and retries
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500); // debounce
      });
      await act(async () => {
        await vi.advanceTimersByTimeAsync(100); // retry
      });

      expect(saveFn).toHaveBeenCalledTimes(2);
      expect(result.current.error).toBeNull();

      // Reset call count for next test
      callCount = 0;
      saveFn.mockClear();

      // Change data to trigger new save
      rerender({ data: 'data2' });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      // Should succeed on first try (retry count was reset)
      expect(saveFn).toHaveBeenCalledTimes(1);
      expect(result.current.error).toBeNull();
    });
  });

  describe('Cleanup on unmount', () => {
    it('should cancel pending debounced save on unmount', async () => {
      const saveFn = vi.fn().mockResolvedValue(undefined);
      const { rerender, unmount } = renderHook(
        ({ data }) => useAutoSave(data, { saveFn }),
        { initialProps: { data: 'initial' } }
      );

      rerender({ data: 'changed' });

      // Unmount before debounce completes
      unmount();

      // Advance time past debounce delay
      await act(async () => {
        await vi.advanceTimersByTimeAsync(1000);
      });

      // Save should not have been called
      expect(saveFn).not.toHaveBeenCalled();
    });

    it('should cancel pending retry on unmount', async () => {
      const saveFn = vi.fn().mockRejectedValue(new Error('Error'));
      const { unmount } = renderHook(() =>
        useAutoSave('data', { saveFn, retryDelayMs: 1000 })
      );

      // Trigger save that will fail
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(saveFn).toHaveBeenCalledTimes(1);

      // Unmount before retry
      unmount();
      saveFn.mockClear();

      // Advance past retry delay
      await act(async () => {
        await vi.advanceTimersByTimeAsync(2000);
      });

      // Retry should not have been attempted
      expect(saveFn).not.toHaveBeenCalled();
    });

    it('should not update state after unmount', async () => {
      let resolvePromise: () => void;
      const saveFn = vi.fn().mockImplementation(
        () =>
          new Promise<void>((resolve) => {
            resolvePromise = resolve;
          })
      );

      const { result, unmount } = renderHook(() =>
        useAutoSave('data', { saveFn })
      );

      // Start save
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(result.current.isSaving).toBe(true);

      // Unmount while save is in progress
      unmount();

      // Complete the save (should not cause errors)
      await act(async () => {
        resolvePromise!();
      });

      // No errors should occur (state updates are guarded by isMountedRef)
    });
  });

  describe('Success and error states', () => {
    it('should set isSaving to true during save', async () => {
      let resolvePromise: () => void;
      const saveFn = vi.fn().mockImplementation(
        () =>
          new Promise<void>((resolve) => {
            resolvePromise = resolve;
          })
      );

      const { result } = renderHook(() => useAutoSave('data', { saveFn }));

      expect(result.current.isSaving).toBe(false);

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      // Should be saving now
      expect(result.current.isSaving).toBe(true);

      await act(async () => {
        resolvePromise!();
      });

      expect(result.current.isSaving).toBe(false);
    });

    it('should set lastSavedAt on successful save', async () => {
      const saveFn = vi.fn().mockResolvedValue(undefined);
      const { result } = renderHook(() => useAutoSave('data', { saveFn }));

      expect(result.current.lastSavedAt).toBeNull();

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(result.current.lastSavedAt).toBeInstanceOf(Date);
    });

    it('should set error on failed save', async () => {
      const error = new Error('Save failed');
      const saveFn = vi.fn().mockRejectedValue(error);
      const { result } = renderHook(() =>
        useAutoSave('data', { saveFn, maxRetries: 0 })
      );

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(result.current.error).toBeTruthy();
      expect(result.current.error?.message).toBe('Save failed');
      expect(result.current.isSaving).toBe(false);
    });

    it('should clear error on clearError', async () => {
      const saveFn = vi.fn().mockRejectedValue(new Error('Error'));
      const { result } = renderHook(() =>
        useAutoSave('data', { saveFn, maxRetries: 0 })
      );

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(result.current.error).toBeTruthy();

      act(() => {
        result.current.clearError();
      });

      expect(result.current.error).toBeNull();
    });

    it('should clear error on next successful save', async () => {
      let shouldFail = true;
      const saveFn = vi.fn().mockImplementation(() => {
        if (shouldFail) {
          return Promise.reject(new Error('Error'));
        }
        return Promise.resolve();
      });

      const { result, rerender } = renderHook(
        ({ data }) => useAutoSave(data, { saveFn, maxRetries: 0 }),
        { initialProps: { data: 'data1' } }
      );

      // First save fails
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(result.current.error).toBeTruthy();

      // Change data and make next save succeed
      shouldFail = false;
      rerender({ data: 'data2' });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(result.current.error).toBeNull();
      expect(result.current.lastSavedAt).toBeInstanceOf(Date);
    });
  });

  describe('saveNow method', () => {
    it('should trigger immediate save without debouncing', async () => {
      const saveFn = vi.fn().mockResolvedValue(undefined);
      const { result } = renderHook(() => useAutoSave('data', { saveFn }));

      await act(async () => {
        result.current.saveNow();
      });

      // Should save immediately (no debounce delay needed)
      expect(saveFn).toHaveBeenCalledTimes(1);
    });

    it('should cancel pending debounced save when saveNow is called', async () => {
      const saveFn = vi.fn().mockResolvedValue(undefined);
      const { result, rerender } = renderHook(
        ({ data }) => useAutoSave(data, { saveFn }),
        { initialProps: { data: 'data1' } }
      );

      // Trigger debounced save
      rerender({ data: 'data2' });

      // Call saveNow before debounce completes
      await act(async () => {
        result.current.saveNow();
      });

      expect(saveFn).toHaveBeenCalledTimes(1);
      expect(saveFn).toHaveBeenCalledWith('data2');

      // Advance past debounce delay
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      // Should not save again (debounced save was canceled)
      expect(saveFn).toHaveBeenCalledTimes(1);
    });
  });

  describe('enabled option', () => {
    it('should not save when enabled is false', async () => {
      const saveFn = vi.fn().mockResolvedValue(undefined);
      const { rerender } = renderHook(
        ({ data }) => useAutoSave(data, { saveFn, enabled: false }),
        { initialProps: { data: 'data1' } }
      );

      rerender({ data: 'data2' });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(1000);
      });

      expect(saveFn).not.toHaveBeenCalled();
    });

    it('should not save with saveNow when enabled is false', async () => {
      const saveFn = vi.fn().mockResolvedValue(undefined);
      const { result } = renderHook(() =>
        useAutoSave('data', { saveFn, enabled: false })
      );

      await act(async () => {
        result.current.saveNow();
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(100);
      });

      expect(saveFn).not.toHaveBeenCalled();
    });

    it('should resume saving when enabled changes to true', async () => {
      const saveFn = vi.fn().mockResolvedValue(undefined);
      const { rerender } = renderHook(
        ({ enabled, data }) => useAutoSave(data, { saveFn, enabled }),
        { initialProps: { enabled: false, data: 'data1' } }
      );

      // Change data while disabled
      rerender({ enabled: false, data: 'data2' });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });
      expect(saveFn).not.toHaveBeenCalled();

      // Enable auto-save
      rerender({ enabled: true, data: 'data2' });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(saveFn).toHaveBeenCalledWith('data2');
    });
  });
});
