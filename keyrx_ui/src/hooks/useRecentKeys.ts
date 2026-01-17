import { useState, useCallback } from 'react';

const STORAGE_KEY_RECENT = 'keyrx_recent_keys';
const MAX_RECENT_KEYS = 10;

/**
 * Load array from localStorage with error handling
 */
function loadFromStorage(key: string): string[] {
  try {
    const stored = localStorage.getItem(key);
    if (stored) {
      const parsed = JSON.parse(stored);
      return Array.isArray(parsed) ? parsed : [];
    }
  } catch (err) {
    console.warn(`Failed to load ${key} from localStorage:`, err);
  }
  return [];
}

/**
 * Save array to localStorage with error handling
 */
function saveToStorage(key: string, data: string[]): void {
  try {
    localStorage.setItem(key, JSON.stringify(data));
  } catch (err) {
    console.error(`Failed to save ${key} to localStorage:`, err);
  }
}

export interface UseRecentKeysReturn {
  recentKeys: string[];
  addRecentKey: (keyId: string) => void;
  clearRecentKeys: () => void;
}

/**
 * Hook to manage recent keys with localStorage persistence
 * Maintains a FIFO queue of recent key IDs (max 10)
 * Most recently added key appears first
 */
export function useRecentKeys(): UseRecentKeysReturn {
  const [recentKeys, setRecentKeys] = useState<string[]>(() =>
    loadFromStorage(STORAGE_KEY_RECENT)
  );

  const addRecentKey = useCallback((keyId: string) => {
    setRecentKeys((prev) => {
      // Remove existing occurrence of this key
      const filtered = prev.filter((id) => id !== keyId);
      // Add to front, limit to max size
      const updated = [keyId, ...filtered].slice(0, MAX_RECENT_KEYS);
      saveToStorage(STORAGE_KEY_RECENT, updated);
      return updated;
    });
  }, []);

  const clearRecentKeys = useCallback(() => {
    setRecentKeys([]);
    saveToStorage(STORAGE_KEY_RECENT, []);
  }, []);

  return {
    recentKeys,
    addRecentKey,
    clearRecentKeys,
  };
}
