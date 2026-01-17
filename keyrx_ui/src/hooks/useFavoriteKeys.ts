import { useState, useCallback } from 'react';

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
