/**
 * RhaiSyncEngine - Bidirectional synchronization between visual and code editors
 *
 * This component manages the synchronization between the visual configuration
 * editor (KeyboardVisualizer with key mappings) and the Monaco code editor
 * (Rhai script). It ensures both views stay in sync while preserving user
 * edits and handling errors gracefully.
 *
 * Key features:
 * - Bidirectional sync: visual ↔ code
 * - Debounced parsing of code changes (500ms)
 * - Immediate code generation from visual changes
 * - State machine: idle → parsing/generating → syncing → idle/error
 * - Graceful error handling with last valid state preservation
 * - LocalStorage persistence of unsaved changes
 *
 * @module RhaiSyncEngine
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import {
  parseRhaiScript,
  type RhaiAST,
  type ParseError,
} from '../utils/rhaiParser';
import { generateRhaiScript } from '../utils/rhaiCodeGen';

/**
 * Sync state machine states
 */
export type SyncState = 'idle' | 'parsing' | 'generating' | 'syncing' | 'error';

/**
 * Direction of the sync operation
 */
export type SyncDirection = 'visual-to-code' | 'code-to-visual' | 'none';

/**
 * Sync engine configuration options
 */
export interface RhaiSyncEngineOptions {
  /** Unique identifier for localStorage persistence (e.g., profileId) */
  storageKey: string;
  /** Debounce delay for code editor changes in milliseconds (default: 500) */
  debounceMs?: number;
  /** Whether to enable localStorage persistence (default: true) */
  enablePersistence?: boolean;
  /** Callback when sync state changes */
  onStateChange?: (state: SyncState) => void;
  /** Callback when sync error occurs */
  onError?: (error: ParseError, direction: SyncDirection) => void;
}

/**
 * Sync engine result
 */
export interface RhaiSyncEngineResult {
  /** Current sync state */
  state: SyncState;
  /** Current sync direction */
  direction: SyncDirection;
  /** Parse error if state is 'error' */
  error: ParseError | null;
  /** Last valid AST (preserved during errors) */
  lastValidAST: RhaiAST | null;
  /** Handle code editor changes */
  onCodeChange: (code: string) => void;
  /** Handle visual editor changes */
  onVisualChange: (ast: RhaiAST) => void;
  /** Get current code content */
  getCode: () => string;
  /** Get current AST */
  getAST: () => RhaiAST | null;
  /** Clear error state and restore last valid state */
  clearError: () => void;
  /** Force sync in a specific direction */
  forceSync: (direction: SyncDirection) => void;
}

/**
 * Custom hook for bidirectional Rhai sync engine
 *
 * @example
 * ```tsx
 * function ConfigEditor({ profileId }) {
 *   const {
 *     state,
 *     error,
 *     onCodeChange,
 *     onVisualChange,
 *     getCode,
 *     getAST
 *   } = useRhaiSyncEngine({
 *     storageKey: `profile-${profileId}`,
 *     onError: (err) => console.error('Sync error:', err)
 *   });
 *
 *   return (
 *     <>
 *       <MonacoEditor value={getCode()} onChange={onCodeChange} />
 *       <KeyboardVisualizer ast={getAST()} onChange={onVisualChange} />
 *       {state === 'error' && <ErrorBanner error={error} />}
 *     </>
 *   );
 * }
 * ```
 */
export function useRhaiSyncEngine(
  options: RhaiSyncEngineOptions
): RhaiSyncEngineResult {
  const {
    storageKey,
    debounceMs = 500,
    enablePersistence = true,
    onStateChange,
    onError,
  } = options;

  // State management
  const [state, setState] = useState<SyncState>('idle');
  const [direction, setDirection] = useState<SyncDirection>('none');
  const [error, setError] = useState<ParseError | null>(null);
  const [lastValidAST, setLastValidAST] = useState<RhaiAST | null>(null);

  // Refs to track current values and prevent stale closures
  const codeRef = useRef<string>('');
  const astRef = useRef<RhaiAST | null>(null);
  const debounceTimerRef = useRef<NodeJS.Timeout | null>(null);
  const parseTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const generateTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const isMountedRef = useRef(true);
  const syncLockRef = useRef(false); // Prevent infinite sync loops
  const abortControllerRef = useRef<AbortController | null>(null);

  // Update state and notify
  const updateState = useCallback(
    (newState: SyncState) => {
      setState(newState);
      onStateChange?.(newState);
    },
    [onStateChange]
  );

  // Load from localStorage on mount
  useEffect(() => {
    if (!enablePersistence) return;

    try {
      const saved = localStorage.getItem(`rhai-sync-${storageKey}`);
      if (saved) {
        const { code, timestamp } = JSON.parse(saved);
        // Only restore if saved within last 24 hours
        const age = Date.now() - timestamp;
        if (age < 24 * 60 * 60 * 1000) {
          codeRef.current = code;
          // Parse the restored code
          const result = parseRhaiScript(code);
          if (result.success && result.ast) {
            astRef.current = result.ast;
            // eslint-disable-next-line react-hooks/set-state-in-effect
            setLastValidAST(result.ast);
          }
        } else {
          // Clear old data
          localStorage.removeItem(`rhai-sync-${storageKey}`);
        }
      }
    } catch (err) {
      console.warn('Failed to restore from localStorage:', err);
    }
  }, [storageKey, enablePersistence]);

  // Save to localStorage
  const persistToStorage = useCallback(
    (code: string) => {
      if (!enablePersistence) return;

      try {
        localStorage.setItem(
          `rhai-sync-${storageKey}`,
          JSON.stringify({
            code,
            timestamp: Date.now(),
          })
        );
      } catch (err) {
        console.warn('Failed to save to localStorage:', err);
      }
    },
    [storageKey, enablePersistence]
  );

  // Cleanup on unmount
  useEffect(() => {
    // Create new AbortController for this mount
    abortControllerRef.current = new AbortController();

    return () => {
      isMountedRef.current = false;

      // Abort any ongoing async operations
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }

      // Clear all timeouts
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
        debounceTimerRef.current = null;
      }
      if (parseTimeoutRef.current) {
        clearTimeout(parseTimeoutRef.current);
        parseTimeoutRef.current = null;
      }
      if (generateTimeoutRef.current) {
        clearTimeout(generateTimeoutRef.current);
        generateTimeoutRef.current = null;
      }
    };
  }, []);

  // Parse code and update AST (code → visual)
  const parseCode = useCallback(
    (code: string) => {
      if (!isMountedRef.current || syncLockRef.current) return;

      syncLockRef.current = true;

      // Only update state if mounted
      if (isMountedRef.current) {
        updateState('parsing');
        setDirection('code-to-visual');
      }

      // Clear any existing parse timeout
      if (parseTimeoutRef.current) {
        clearTimeout(parseTimeoutRef.current);
      }

      // Small delay to allow UI to update
      parseTimeoutRef.current = setTimeout(() => {
        // Check if component is still mounted
        if (!isMountedRef.current) {
          syncLockRef.current = false;
          return;
        }

        try {
          const result = parseRhaiScript(code);

          if (result.success && result.ast) {
            // Success: update AST and last valid state
            astRef.current = result.ast;
            if (isMountedRef.current) {
              setLastValidAST(result.ast);
              setError(null);
              updateState('idle');
              setDirection('none');
            }
          } else if (result.error) {
            // Error: preserve last valid AST, show error
            if (isMountedRef.current) {
              setError(result.error);
              updateState('error');
              onError?.(result.error, 'code-to-visual');
            }
          }
        } catch (err) {
          // Handle unexpected errors from parseRhaiScript
          if (isMountedRef.current) {
            const unknownErr = err as { line?: number; column?: number };
            const error: ParseError = {
              line: unknownErr.line ?? 0,
              column: unknownErr.column ?? 0,
              message: err instanceof Error ? err.message : 'Parse failed',
              suggestion: 'Check your Rhai script syntax',
            };
            setError(error);
            updateState('error');
            onError?.(error, 'code-to-visual');
          }
        }

        syncLockRef.current = false;
        parseTimeoutRef.current = null;
      }, 10);
    },
    [updateState, onError]
  );

  // Generate code from AST (visual → code)
  const generateCode = useCallback(
    (ast: RhaiAST) => {
      if (!isMountedRef.current || syncLockRef.current) return;

      syncLockRef.current = true;

      // Only update state if mounted
      if (isMountedRef.current) {
        updateState('generating');
        setDirection('visual-to-code');
      }

      // Clear any existing generate timeout
      if (generateTimeoutRef.current) {
        clearTimeout(generateTimeoutRef.current);
      }

      // Small delay to allow UI to update
      generateTimeoutRef.current = setTimeout(() => {
        // Check if component is still mounted
        if (!isMountedRef.current) {
          syncLockRef.current = false;
          return;
        }

        try {
          const code = generateRhaiScript(ast);
          codeRef.current = code;
          astRef.current = ast;

          if (isMountedRef.current) {
            setLastValidAST(ast);
            persistToStorage(code);
            setError(null);
            updateState('idle');
            setDirection('none');
          }
        } catch (err) {
          if (isMountedRef.current) {
            const error: ParseError = {
              line: 0,
              column: 0,
              message:
                err instanceof Error ? err.message : 'Code generation failed',
              suggestion: 'Check visual editor state for invalid mappings',
            };
            setError(error);
            updateState('error');
            onError?.(error, 'visual-to-code');
          }
        }

        syncLockRef.current = false;
        generateTimeoutRef.current = null;
      }, 10);
    },
    [updateState, onError, persistToStorage]
  );

  // Handle code editor changes (debounced)
  const onCodeChange = useCallback(
    (code: string) => {
      if (!isMountedRef.current) return;

      codeRef.current = code;
      persistToStorage(code);

      // Clear existing debounce timer
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }

      // Set new debounce timer
      debounceTimerRef.current = setTimeout(() => {
        parseCode(code);
      }, debounceMs);
    },
    [debounceMs, parseCode, persistToStorage]
  );

  // Handle visual editor changes (immediate)
  const onVisualChange = useCallback(
    (ast: RhaiAST) => {
      if (!isMountedRef.current) return;
      generateCode(ast);
    },
    [generateCode]
  );

  // Get current code
  const getCode = useCallback(() => codeRef.current, []);

  // Get current AST
  const getAST = useCallback(() => astRef.current, []);

  // Clear error and restore last valid state
  const clearError = useCallback(() => {
    setError(null);
    updateState('idle');
    setDirection('none');
  }, [updateState]);

  // Force sync in a specific direction
  const forceSync = useCallback(
    (syncDirection: SyncDirection) => {
      if (syncDirection === 'code-to-visual') {
        parseCode(codeRef.current);
      } else if (syncDirection === 'visual-to-code' && astRef.current) {
        generateCode(astRef.current);
      }
    },
    [parseCode, generateCode]
  );

  return {
    state,
    direction,
    error,
    lastValidAST,
    onCodeChange,
    onVisualChange,
    getCode,
    getAST,
    clearError,
    forceSync,
  };
}

/**
 * RhaiSyncEngine component (wrapper for use in JSX)
 *
 * @example
 * ```tsx
 * <RhaiSyncEngine
 *   storageKey="profile-123"
 *   onStateChange={(state) => console.log('State:', state)}
 * >
 *   {({ onCodeChange, onVisualChange, getCode, getAST, error }) => (
 *     <>
 *       <MonacoEditor value={getCode()} onChange={onCodeChange} />
 *       <KeyboardVisualizer ast={getAST()} onChange={onVisualChange} />
 *       {error && <ErrorBanner error={error} />}
 *     </>
 *   )}
 * </RhaiSyncEngine>
 * ```
 */
export function RhaiSyncEngine({
  children,
  ...options
}: RhaiSyncEngineOptions & {
  children: (result: RhaiSyncEngineResult) => React.ReactNode;
}) {
  const result = useRhaiSyncEngine(options);
  return <>{children(result)}</>;
}
