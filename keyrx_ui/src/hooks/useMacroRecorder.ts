/**
 * useMacroRecorder - React hook for macro recording functionality.
 *
 * This hook provides an interface to the daemon's macro recording API,
 * managing recording state, event retrieval, and error handling.
 */

import { useState, useCallback, useRef } from 'react';

/**
 * Macro event captured from keyboard input.
 */
export interface MacroEvent {
  /** The key event (press or release) */
  event: {
    code: number;
    value: number;
  };
  /** Relative timestamp in microseconds from recording start */
  relative_timestamp_us: number;
}

/**
 * Recording state for macro capture.
 */
type RecordingState = 'idle' | 'recording' | 'stopped' | 'error';

/**
 * State management for the macro recorder hook.
 */
interface MacroRecorderState {
  /** Current recording state */
  recordingState: RecordingState;
  /** Recorded events */
  events: MacroEvent[];
  /** Error message if operation failed */
  error: string | null;
  /** Whether data is being fetched */
  isLoading: boolean;
}

/**
 * Return value of useMacroRecorder hook.
 */
export interface UseMacroRecorderReturn {
  /** Current recorder state */
  state: MacroRecorderState;
  /** Start recording macro events */
  startRecording: () => Promise<void>;
  /** Stop recording macro events */
  stopRecording: () => Promise<void>;
  /** Fetch recorded events from daemon */
  fetchEvents: () => Promise<void>;
  /** Clear all recorded events */
  clearEvents: () => Promise<void>;
  /** Clear error message */
  clearError: () => void;
}

/**
 * Custom hook for macro recording functionality.
 *
 * Manages macro recording state, communicates with the daemon API,
 * and provides methods for recording control and event retrieval.
 *
 * @example
 * ```tsx
 * function MacroRecorder() {
 *   const { state, startRecording, stopRecording, fetchEvents } = useMacroRecorder();
 *
 *   const handleStart = async () => {
 *     await startRecording();
 *   };
 *
 *   return (
 *     <div>
 *       {state.recordingState === 'recording' && <div>Recording...</div>}
 *       <div>Events captured: {state.events.length}</div>
 *     </div>
 *   );
 * }
 * ```
 */
export function useMacroRecorder(): UseMacroRecorderReturn {
  // State management
  const [state, setState] = useState<MacroRecorderState>({
    recordingState: 'idle',
    events: [],
    error: null,
    isLoading: false,
  });

  // Track mounted state to prevent updates after unmount
  const isMountedRef = useRef(true);

  /**
   * Start recording macro events.
   */
  const startRecording = useCallback(async () => {
    try {
      setState((prev) => ({
        ...prev,
        isLoading: true,
        error: null,
      }));

      const response = await fetch('/api/macros/start-recording', {
        method: 'POST',
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error?.message || 'Failed to start recording');
      }

      if (!isMountedRef.current) return;

      setState((prev) => ({
        ...prev,
        recordingState: 'recording',
        events: [], // Clear previous events
        isLoading: false,
        error: null,
      }));
    } catch (err) {
      if (!isMountedRef.current) return;

      const message = err instanceof Error ? err.message : 'Failed to start recording';

      setState((prev) => ({
        ...prev,
        recordingState: 'error',
        isLoading: false,
        error: message,
      }));
    }
  }, []);

  /**
   * Stop recording macro events.
   */
  const stopRecording = useCallback(async () => {
    try {
      setState((prev) => ({
        ...prev,
        isLoading: true,
        error: null,
      }));

      const response = await fetch('/api/macros/stop-recording', {
        method: 'POST',
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error?.message || 'Failed to stop recording');
      }

      await response.json();

      if (!isMountedRef.current) return;

      setState((prev) => ({
        ...prev,
        recordingState: 'stopped',
        isLoading: false,
        error: null,
      }));

      // Automatically fetch events after stopping
      await fetchEvents();
    } catch (err) {
      if (!isMountedRef.current) return;

      const message = err instanceof Error ? err.message : 'Failed to stop recording';

      setState((prev) => ({
        ...prev,
        recordingState: 'error',
        isLoading: false,
        error: message,
      }));
    }
  }, []);

  /**
   * Fetch recorded events from daemon.
   */
  const fetchEvents = useCallback(async () => {
    try {
      setState((prev) => ({
        ...prev,
        isLoading: true,
        error: null,
      }));

      const response = await fetch('/api/macros/recorded-events');

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error?.message || 'Failed to fetch events');
      }

      const data = await response.json();

      if (!isMountedRef.current) return;

      setState((prev) => ({
        ...prev,
        events: data.events || [],
        recordingState: data.recording ? 'recording' : 'stopped',
        isLoading: false,
        error: null,
      }));
    } catch (err) {
      if (!isMountedRef.current) return;

      const message = err instanceof Error ? err.message : 'Failed to fetch events';

      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: message,
      }));
    }
  }, []);

  /**
   * Clear all recorded events.
   */
  const clearEvents = useCallback(async () => {
    try {
      setState((prev) => ({
        ...prev,
        isLoading: true,
        error: null,
      }));

      const response = await fetch('/api/macros/clear', {
        method: 'POST',
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error?.message || 'Failed to clear events');
      }

      if (!isMountedRef.current) return;

      setState((prev) => ({
        ...prev,
        events: [],
        recordingState: 'idle',
        isLoading: false,
        error: null,
      }));
    } catch (err) {
      if (!isMountedRef.current) return;

      const message = err instanceof Error ? err.message : 'Failed to clear events';

      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: message,
      }));
    }
  }, []);

  /**
   * Clear error message.
   */
  const clearError = useCallback(() => {
    setState((prev) => ({
      ...prev,
      error: null,
      recordingState: prev.recordingState === 'error' ? 'idle' : prev.recordingState,
    }));
  }, []);

  return {
    state,
    startRecording,
    stopRecording,
    fetchEvents,
    clearEvents,
    clearError,
  };
}
