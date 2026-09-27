import { useState, useCallback } from 'react';
import { logger } from '@/utils/logger';

const STORAGE_KEY_FAVORITES = 'keyrx_favorite_keys';

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
    logger.warn('favorite_keys_load_failed', { key, error: String(err) });
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
    logger.error('favorite_keys_save_failed', err instanceof Error ? err : undefined, {
      key,
    });
  }
}

export interface UseFavoriteKeysReturn {
  favoriteKeys: string[];
  toggleFavorite: (keyId: string) => void;
  isFavorite: (keyId: string) => boolean;
}

/**
 * Hook to manage favorite keys with localStorage persistence
 * Allows toggling keys as favorites and checking favorite status
 */
export function useFavoriteKeys(): UseFavoriteKeysReturn {
  const [favoriteKeys, setFavoriteKeys] = useState<string[]>(() =>
    loadFromStorage(STORAGE_KEY_FAVORITES)
  );

  const toggleFavorite = useCallback((keyId: string) => {
    setFavoriteKeys((prev) => {
      const isFavorite = prev.includes(keyId);
      const updated = isFavorite
        ? prev.filter((id) => id !== keyId)
        : [...prev, keyId];
      saveToStorage(STORAGE_KEY_FAVORITES, updated);
      return updated;
    });
  }, []);

  const isFavorite = useCallback(
    (keyId: string) => {
      return favoriteKeys.includes(keyId);
    },
    [favoriteKeys]
  );

  return {
    favoriteKeys,
    toggleFavorite,
    isFavorite,
  };
}
