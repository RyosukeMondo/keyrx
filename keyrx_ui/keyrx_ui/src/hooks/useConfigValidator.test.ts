/**
 * Unit tests for useConfigValidator React hook.
 *
 * Tests cover:
 * - Debouncing behavior (validate only after 500ms idle)
 * - Validation state updates (isValidating flag)
 * - WASM unavailable fallback
 * - Cleanup on unmount (debounce timer cancellation)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useConfigValidator } from './useConfigValidator';
import type { ValidationResult } from '@/types/validation';

// Mock the validator module
vi.mock('@/utils/validator', () => ({
  validator: {
    validate: vi.fn(),
  },
}));

// Mock the WASM core module
vi.mock('@/wasm/core', () => ({
  wasmCore: {
    init: vi.fn(),
  },
}));

import { validator } from '@/utils/validator';
import { wasmCore } from '@/wasm/core';

describe('useConfigValidator', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.clearAllTimers();
    // Default: WASM available
    vi.mocked(wasmCore.init).mockResolvedValue(undefined);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('initialization', () => {
    it('should initialize with null validation result and not validating', () => {
      const { result } = renderHook(() => useConfigValidator());

      expect(result.current.validationResult).toBeNull();
      expect(result.current.isValidating).toBe(false);
    });

    it('should check WASM availability on mount', async () => {
      renderHook(() => useConfigValidator());

      await waitFor(() => {
        expect(wasmCore.init).toHaveBeenCalled();
      });
    });

    it('should set wasmAvailable to true when WASM initializes successfully', async () => {
      const { result } = renderHook(() => useConfigValidator());

      await waitFor(() => {
        expect(result.current.wasmAvailable).toBe(true);
      });
    });
  });

  describe('WASM unavailable handling', () => {
    it('should set wasmAvailable to false when WASM init fails', async () => {
      vi.mocked(wasmCore.init).mockRejectedValue(new Error('WASM load failed'));

      const { result } = renderHook(() => useConfigValidator());

      await waitFor(() => {
        expect(result.current.wasmAvailable).toBe(false);
      });
    });

    it('should set fallback error when WASM unavailable', async () => {
      vi.mocked(wasmCore.init).mockRejectedValue(new Error('WASM load failed'));

      const { result } = renderHook(() => useConfigValidator());

      await waitFor(() => {
        expect(result.current.validationResult).not.toBeNull();
        expect(result.current.validationResult?.errors).toHaveLength(1);
        expect(result.current.validationResult?.errors[0].code).toBe('WASM_UNAVAILABLE');
        expect(result.current.validationResult?.errors[0].message).toContain(
          'Validation unavailable'
        );
      });
    });
  });

  describe('debouncing behavior', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('should debounce validation by 500ms', async () => {
      const mockResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };
      vi.mocked(validator.validate).mockResolvedValue(mockResult);

      const { result } = renderHook(() => useConfigValidator());

      // Wait for WASM init
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      // Call validate multiple times within debounce window
      act(() => {
        result.current.validate('config1');
        result.current.validate('config2');
        result.current.validate('config3');
      });

      // Validator should not be called yet
      expect(validator.validate).not.toHaveBeenCalled();

      // Advance timers by 500ms and run all pending promises
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      // Validator should be called only once with the last config
      expect(validator.validate).toHaveBeenCalledTimes(1);
      expect(validator.validate).toHaveBeenCalledWith('config3', expect.any(Object));
    });

    it('should reset debounce timer on each call', async () => {
      const mockResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };
      vi.mocked(validator.validate).mockResolvedValue(mockResult);

      const { result } = renderHook(() => useConfigValidator());

      // Wait for WASM init
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      // First call
      act(() => {
        result.current.validate('config1');
      });

      // Advance time by 300ms (not enough to trigger)
      act(() => {
        vi.advanceTimersByTime(300);
      });

      // Second call should reset timer
      act(() => {
        result.current.validate('config2');
      });

      // Advance time by 300ms (total 600ms from first call, but only 300ms from second)
      act(() => {
        vi.advanceTimersByTime(300);
      });

      // Should not have called validator yet
      expect(validator.validate).not.toHaveBeenCalled();

      // Advance remaining 200ms and run all promises to complete 500ms from second call
      await act(async () => {
        vi.advanceTimersByTime(200);
        await vi.runAllTimersAsync();
      });

      // Now validator should be called with config2
      expect(validator.validate).toHaveBeenCalledTimes(1);
      expect(validator.validate).toHaveBeenCalledWith('config2', expect.any(Object));
    });

    it('should use custom debounce time when provided', async () => {
      const mockResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };
      vi.mocked(validator.validate).mockResolvedValue(mockResult);

      const { result } = renderHook(() => useConfigValidator(1000));

      // Wait for WASM init
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      act(() => {
        result.current.validate('config');
      });

      // 500ms should not trigger
      act(() => {
        vi.advanceTimersByTime(500);
      });
      expect(validator.validate).not.toHaveBeenCalled();

      // 1000ms total should trigger
      await act(async () => {
        vi.advanceTimersByTime(500);
        await vi.runAllTimersAsync();
      });

      expect(validator.validate).toHaveBeenCalledTimes(1);
    });
  });

  describe('validation state updates', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('should set isValidating to true during validation', async () => {
      let resolveValidation: (value: ValidationResult) => void;
      const validationPromise = new Promise<ValidationResult>((resolve) => {
        resolveValidation = resolve;
      });
      vi.mocked(validator.validate).mockReturnValue(validationPromise);

      const { result } = renderHook(() => useConfigValidator());

      // Wait for WASM init
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      act(() => {
        result.current.validate('config');
      });

      // Advance timers to trigger validation
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      // isValidating should be true
      expect(result.current.isValidating).toBe(true);

      // Resolve validation
      await act(async () => {
        resolveValidation!({
          errors: [],
          warnings: [],
          hints: [],
          timestamp: new Date().toISOString(),
        });
      });

      // isValidating should be false
      expect(result.current.isValidating).toBe(false);
    });

    it('should update validationResult when validation completes', async () => {
      const mockResult: ValidationResult = {
        errors: [
          {
            line: 5,
            column: 10,
            message: 'Syntax error',
            code: 'SYNTAX_ERROR',
          },
        ],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };
      vi.mocked(validator.validate).mockResolvedValue(mockResult);

      const { result } = renderHook(() => useConfigValidator());

      // Wait for WASM init
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      act(() => {
        result.current.validate('invalid config');
      });

      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(result.current.validationResult).toEqual(mockResult);
      expect(result.current.isValidating).toBe(false);
    });

    it('should handle validation errors gracefully', async () => {
      vi.mocked(validator.validate).mockRejectedValue(new Error('Validation failed'));

      const { result } = renderHook(() => useConfigValidator());

      // Wait for WASM init
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      act(() => {
        result.current.validate('config');
      });

      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(result.current.validationResult).not.toBeNull();
      expect(result.current.validationResult?.errors).toHaveLength(1);
      expect(result.current.validationResult?.errors[0].code).toBe('VALIDATION_ERROR');
      expect(result.current.validationResult?.errors[0].message).toBe('Validation failed');
      expect(result.current.isValidating).toBe(false);
    });
  });

  describe('clearValidation', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('should clear validation result', async () => {
      const mockResult: ValidationResult = {
        errors: [{ line: 1, column: 1, message: 'Error', code: 'ERROR' }],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };
      vi.mocked(validator.validate).mockResolvedValue(mockResult);

      const { result } = renderHook(() => useConfigValidator());

      // Wait for WASM init
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      // Perform validation
      act(() => {
        result.current.validate('config');
      });

      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(result.current.validationResult).not.toBeNull();

      // Clear validation
      act(() => {
        result.current.clearValidation();
      });

      expect(result.current.validationResult).toBeNull();
      expect(result.current.isValidating).toBe(false);
    });

    it('should cancel pending validation', async () => {
      vi.mocked(validator.validate).mockResolvedValue({
        errors: [],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      });

      const { result } = renderHook(() => useConfigValidator());

      // Wait for WASM init
      await act(async () => {
        await vi.runOnlyPendingTimersAsync();
      });

      // Start validation
      act(() => {
        result.current.validate('config');
      });

      // Clear before debounce timer completes
      act(() => {
        result.current.clearValidation();
      });

      // Advance timers
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      // Validator should not have been called
      expect(validator.validate).not.toHaveBeenCalled();
    });
  });

  describe('cleanup on unmount', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('should cancel debounce timer on unmount', async () => {
      vi.mocked(validator.validate).mockResolvedValue({
        errors: [],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      });

      const { result, unmount } = renderHook(() => useConfigValidator());

      // Wait for WASM init
      await act(async () => {
        await vi.runOnlyPendingTimersAsync();
      });

      // Start validation
      act(() => {
        result.current.validate('config');
      });

      // Unmount before debounce completes
      unmount();

      // Advance timers
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      // Validator should not have been called
      expect(validator.validate).not.toHaveBeenCalled();
    });

    it('should not update state after unmount', async () => {
      let resolveValidation: (value: ValidationResult) => void;
      const validationPromise = new Promise<ValidationResult>((resolve) => {
        resolveValidation = resolve;
      });
      vi.mocked(validator.validate).mockReturnValue(validationPromise);

      const { result, unmount } = renderHook(() => useConfigValidator());

      // Wait for WASM init
      await act(async () => {
        await vi.runOnlyPendingTimersAsync();
      });

      act(() => {
        result.current.validate('config');
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      // Unmount before validation completes
      unmount();

      // Resolve validation after unmount
      await act(async () => {
        resolveValidation!({
          errors: [],
          warnings: [],
          hints: [],
          timestamp: new Date().toISOString(),
        });
      });

      // No error should be thrown (state updates prevented)
      // This test mainly ensures no console warnings about state updates on unmounted component
    });
  });

  describe('validation options', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('should pass validation options to validator', async () => {
      vi.mocked(validator.validate).mockResolvedValue({
        errors: [],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      });

      const { result } = renderHook(() => useConfigValidator());

      // Wait for WASM init
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      const options = {
        enableLinting: true,
        maxErrors: 10,
        maxWarnings: 5,
      };

      act(() => {
        result.current.validate('config', options);
      });

      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(validator.validate).toHaveBeenCalledWith('config', options);
    });
  });
});
