import { useMemo } from 'react';
import type { AssignableKey, KeyCategory } from './assignableKeys';

export interface KeyPaletteFilterResult {
  /** Keys matching the current category and search query */
  filteredKeys: AssignableKey[];
  /** Filtered keys grouped by category, in category-key order */
  groupedKeys: Record<KeyCategory, AssignableKey[]>;
}

/**
 * Filters and groups the assignable key list by category and search query.
 */
export function useKeyPaletteFilter(
  allKeys: AssignableKey[],
  searchQuery: string,
  selectedCategory: KeyCategory | 'all'
): KeyPaletteFilterResult {
  const filteredKeys = useMemo(() => {
    let keys = allKeys;

    // Filter by category
    if (selectedCategory !== 'all') {
      keys = keys.filter((key) => key.category === selectedCategory);
    }

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      keys = keys.filter(
        (key) =>
          key.label.toLowerCase().includes(query) ||
          key.id.toLowerCase().includes(query) ||
          key.description?.toLowerCase().includes(query)
      );
    }

    return keys;
  }, [allKeys, searchQuery, selectedCategory]);

  const groupedKeys = useMemo(() => {
    const groups: Record<KeyCategory, AssignableKey[]> = {
      virtual: [],
      modifier: [],
      lock: [],
      layer: [],
      macro: [],
    };

    filteredKeys.forEach((key) => {
      groups[key.category].push(key);
    });

    return groups;
  }, [filteredKeys]);

  return { filteredKeys, groupedKeys };
}
