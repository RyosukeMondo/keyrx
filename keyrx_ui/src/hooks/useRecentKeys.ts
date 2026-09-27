import { useState, useCallback } from 'react';
import { logger } from '@/utils/logger';

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
    logger.warn('recent_keys_load_failed', { key, error: String(err) });
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
    logger.error('recent_keys_save_failed', err instanceof Error ? err : undefined, {
      key,
    });
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
