import { useEffect, useRef, useState, useCallback } from 'react';

export interface UseAutoSaveOptions<T> {
  /** Function to save the data */
  saveFn: (data: T) => Promise<void>;
  /** Debounce delay in milliseconds (default: 500) */
  debounceMs?: number;
  /** Maximum retry attempts on failure (default: 3) */
  maxRetries?: number;
  /** Initial retry delay in milliseconds (default: 1000) */
  retryDelayMs?: number;
  /** Whether auto-save is enabled (default: true) */
  enabled?: boolean;
}

export interface UseAutoSaveResult {
  /** Whether a save operation is in progress */
  isSaving: boolean;
  /** Error from the last save attempt, if any */
  error: Error | null;
  /** Timestamp of the last successful save */
  lastSavedAt: Date | null;
  /** Trigger a save immediately without debouncing */
  saveNow: () => void;
  /** Clear the error state */
  clearError: () => void;
}

/**
 * Generic auto-save hook with debouncing and retry logic.
 *
 * @example
 * ```tsx
 * const { isSaving, error, lastSavedAt } = useAutoSave(
 *   layoutName,
 *   async (layout) => {
 *     await api.saveLayout(deviceId, layout);
 *   }
 * );
 * ```
 */
export function useAutoSave<T>(
  data: T,
  options: UseAutoSaveOptions<T>
): UseAutoSaveResult {
  const {
    saveFn,
    debounceMs = 500,
    maxRetries = 3,
    retryDelayMs = 1000,
    enabled = true,
  } = options;

  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [lastSavedAt, setLastSavedAt] = useState<Date | null>(null);

  // Use refs to track pending operations and prevent stale closures
  const debounceTimerRef = useRef<NodeJS.Timeout | null>(null);
  const retryTimerRef = useRef<NodeJS.Timeout | null>(null);
  const pendingDataRef = useRef<T>(data);
  const isMountedRef = useRef(true);
  const retryCountRef = useRef(0);

  // Update pending data ref when data changes
  useEffect(() => {
    pendingDataRef.current = data;
  }, [data]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      isMountedRef.current = false;
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
      if (retryTimerRef.current) {
        clearTimeout(retryTimerRef.current);
      }
    };
  }, []);

  // Save function with retry logic
  const executeSave = useCallback(async (dataToSave: T, retryCount: number = 0) => {
    if (!isMountedRef.current) return;

    try {
      setIsSaving(true);
      setError(null);

      await saveFn(dataToSave);

      if (!isMountedRef.current) return;

      setLastSavedAt(new Date());
      setIsSaving(false);
      retryCountRef.current = 0;
    } catch (err) {
      if (!isMountedRef.current) return;

      const error = err instanceof Error ? err : new Error(String(err));

      // Don't retry on validation errors (4xx status codes)
      const isValidationError =
        error.message.includes('400') ||
        error.message.includes('404') ||
        error.message.includes('validation');

      if (isValidationError || retryCount >= maxRetries) {
        setError(error);
        setIsSaving(false);
        retryCountRef.current = 0;
        return;
      }

      // Retry with exponential backoff
      const delay = retryDelayMs * Math.pow(2, retryCount);
      retryCountRef.current = retryCount + 1;

      retryTimerRef.current = setTimeout(() => {
        executeSave(dataToSave, retryCount + 1);
      }, delay);
    }
  }, [saveFn, maxRetries, retryDelayMs]);

  // Trigger save immediately without debouncing
  const saveNow = useCallback(() => {
    if (!enabled) return;

    // Clear any pending debounced save
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
      debounceTimerRef.current = null;
    }

    executeSave(pendingDataRef.current);
  }, [enabled, executeSave]);

  // Clear error state
  const clearError = useCallback(() => {
    setError(null);
  }, []);

  // Debounced auto-save effect
  useEffect(() => {
    if (!enabled) return;

    // Clear existing debounce timer
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }

    // Set new debounce timer
    debounceTimerRef.current = setTimeout(() => {
      executeSave(data);
    }, debounceMs);

    // Cleanup function
    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, [data, debounceMs, enabled, executeSave]);

  return {
    isSaving,
    error,
    lastSavedAt,
    saveNow,
    clearError,
  };
}
