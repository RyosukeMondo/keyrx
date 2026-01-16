/**
 * Custom hook for managing code panel state with localStorage persistence
 *
 * Manages the collapsible code panel UI state including open/closed status
 * and panel height with persistence across sessions.
 *
 * @example
 * ```tsx
 * const { isOpen, height, toggleOpen, setHeight } = useCodePanel();
 *
 * return (
 *   <div>
 *     <button onClick={toggleOpen}>Toggle Code Panel</button>
 *     {isOpen && (
 *       <div style={{ height }}>
 *         <MonacoEditor />
 *       </div>
 *     )}
 *   </div>
 * );
 * ```
 */

import { useState, useCallback, useEffect } from 'react';

const STORAGE_KEY_HEIGHT = 'codePanel.height';
const DEFAULT_HEIGHT = 300;

interface UseCodePanelReturn {
  /** Whether the code panel is currently open */
  isOpen: boolean;
  /** Current height of the code panel in pixels */
  height: number;
  /** Toggle the open/closed state of the panel */
  toggleOpen: () => void;
  /** Set the height of the panel and persist to localStorage */
  setHeight: (height: number) => void;
}

/**
 * Hook for managing code panel state
 *
 * @returns Object containing panel state and control functions
 */
export function useCodePanel(): UseCodePanelReturn {
  // Initialize height from localStorage or use default
  const [height, setHeightState] = useState<number>(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY_HEIGHT);
      if (stored) {
        const parsed = parseInt(stored, 10);
        if (!isNaN(parsed) && parsed > 0) {
          return parsed;
        }
      }
    } catch (err) {
      console.error('Failed to load code panel height from localStorage:', err);
    }
    return DEFAULT_HEIGHT;
  });

  const [isOpen, setIsOpen] = useState<boolean>(false);

  // Persist height to localStorage whenever it changes
  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY_HEIGHT, height.toString());
    } catch (err) {
      console.error('Failed to persist code panel height:', err);
    }
  }, [height]);

  // Toggle open/closed state
  const toggleOpen = useCallback(() => {
    setIsOpen((prev) => !prev);
  }, []);

  // Set height with validation and persistence
  const setHeight = useCallback((newHeight: number) => {
    if (newHeight > 0) {
      setHeightState(newHeight);
    }
  }, []);

  return {
    isOpen,
    height,
    toggleOpen,
    setHeight,
  };
}
